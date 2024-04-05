mod activity_node;
pub mod connection;
mod mutex_node;

pub use activity_node::ActivityNode;
use egui::{emath::TSTransform, Pos2};
pub use mutex_node::MutexNode;
use rand::{thread_rng, Rng};
use random_word::Lang;

use self::connection::Direction;

#[derive(
    PartialOrd, Ord, Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct ActivityNodeId(usize);
impl std::ops::Deref for ActivityNodeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for ActivityNodeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(
    PartialOrd, Ord, Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct MutexNodeId(usize);
impl std::ops::Deref for MutexNodeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for MutexNodeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(PartialEq, Eq)]
pub enum EditingMode {
    None,
    Delete,
}

impl Default for EditingMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy)]
enum AnyNode {
    Activity(ActivityNodeId),
    Mutex(MutexNodeId),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Graph {
    pub name: String,
    activity_nodes: indexmap::IndexMap<ActivityNodeId, ActivityNode>,
    mutex_nodes: std::collections::HashMap<MutexNodeId, MutexNode>,

    connections: std::collections::HashMap<
        ActivityNodeId,
        std::collections::HashMap<MutexNodeId, connection::Connection>,
    >,

    next_activity_id: ActivityNodeId,
    next_mutex_id: MutexNodeId,

    tick_progress: f32,

    pub ticks_per_second: f32,

    remaining_ticks_to_run: i32,

    #[serde(skip)]
    currently_connecting_from: Option<AnyNode>,

    #[serde(skip)]
    pub editing_mode: EditingMode,

    #[serde(skip)]
    autofit_rect: Option<egui::Rect>,
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            activity_nodes: self.activity_nodes.clone(),
            mutex_nodes: self.mutex_nodes.clone(),
            connections: self.connections.clone(),
            next_activity_id: self.next_activity_id,
            next_mutex_id: self.next_mutex_id,
            tick_progress: self.tick_progress,
            ticks_per_second: self.ticks_per_second,
            ..Default::default()
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            name: "Unnamed Graph".into(),
            activity_nodes: indexmap::IndexMap::new(),
            mutex_nodes: std::collections::HashMap::new(),
            connections: std::collections::HashMap::new(),
            next_activity_id: ActivityNodeId(0),
            next_mutex_id: MutexNodeId(0),
            tick_progress: 0.,
            ticks_per_second: 1.,
            remaining_ticks_to_run: 0,
            currently_connecting_from: None,
            editing_mode: EditingMode::None,
            autofit_rect: Some(egui::Rect::NAN),
        }
    }
}

// import/export
impl Graph {
    pub fn from_csv(text: &str) -> Result<Self, String> {
        const SEPERATOR: char = ';';
        let mut graph = Graph::default();

        for (line_number, line) in text.lines().enumerate() {
            let line_number = line_number + 1; // enumerate starts at 0

            // split returns at least 1 empty string -> subsequent values[0] are fine
            let values = line.split(SEPERATOR).map(|s| s.trim()).collect::<Vec<_>>();

            // match first value to determine type of line
            match values[0].to_lowercase().as_str() {
                "task" if values.len() >= 9 => {
                    let mut activity_node = ActivityNode::new(egui::Pos2 {
                        x: values[1].parse::<f32>().map_err(|_| {
                            format!("Error while parsing Position X in line: {}", line_number)
                        })?,
                        y: values[2].parse::<f32>().map_err(|_| {
                            format!("Error while parsing Position Y in line: {}", line_number)
                        })?,
                    });
                    let activity_id =
                        ActivityNodeId(values[3].parse::<usize>().map_err(|_| {
                            format!("Error while parsing ID in line: {}", line_number)
                        })?);
                    activity_node.task_name = values[4].to_string();
                    activity_node.activity_name = values[5].to_string();
                    activity_node.priority = values[6].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Priority in line: {}", line_number)
                    })?;
                    activity_node.duration = values[7].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Duration in line: {}", line_number)
                    })?;
                    activity_node.remaining_duration = values[8].parse::<u32>().map_err(|_| {
                        format!(
                            "Error while parsing Remaining Duration in line: {}",
                            line_number
                        )
                    })?;
                    graph.add_activiy_node_with_id(activity_node, activity_id);

                    values[9..]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .find_map(|x| match x.parse::<usize>() {
                            Ok(mutex_id) => {
                                graph.connect(
                                    activity_id,
                                    MutexNodeId(mutex_id),
                                    Direction::ActivityToMutex,
                                    false,
                                );
                                None
                            }
                            Err(_) => Some(Err(format!(
                                "Error while parsing Activity Connection in line: {}",
                                line_number
                            ))),
                        })
                        .unwrap_or(Ok(()))?;
                }

                "mutex" if values.len() >= 5 => {
                    let mut mutex_node = MutexNode::new(egui::Pos2 {
                        x: values[1].parse::<f32>().map_err(|_| {
                            format!("Error while parsing Position X in line: {}", line_number)
                        })?,
                        y: values[2].parse::<f32>().map_err(|_| {
                            format!("Error while parsing Position Y in line: {}", line_number)
                        })?,
                    });
                    let mutex_id =
                        MutexNodeId(values[3].parse::<usize>().map_err(|_| {
                            format!("Error while parsing ID in line: {}", line_number)
                        })?);
                    mutex_node.value = values[4].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Mutex Value in line: {}", line_number)
                    })?;
                    graph.add_mutex_node_with_id(mutex_node, mutex_id);

                    values[5..]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .find_map(|x| match x.parse::<usize>() {
                            Ok(activity_id) => {
                                graph.connect(
                                    ActivityNodeId(activity_id),
                                    mutex_id,
                                    Direction::MutexToActivity,
                                    false,
                                );
                                None
                            }
                            Err(_) => Some(Err(format!(
                                "Error while parsing Activity Connection in line: {}",
                                line_number
                            ))),
                        })
                        .unwrap_or(Ok(()))?;
                }
                _ => {} // skip line
            }
        }
        graph.update_connection_states();
        Ok(graph)
    }

    pub fn to_csv(&self) -> String {
        use std::collections::HashMap;
        let seperator = ";";

        let mut connection_activity_to_mutex: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut connection_mutex_to_activity: HashMap<usize, Vec<usize>> = HashMap::new();

        for (activity_id, activity_connections) in &self.connections {
            for (mutex_id, connection) in activity_connections {
                match connection.get_direction() {
                    Direction::ActivityToMutex => {
                        connection_activity_to_mutex
                            .entry(activity_id.0)
                            .or_default()
                            .push(mutex_id.0);
                    }
                    Direction::MutexToActivity => {
                        connection_mutex_to_activity
                            .entry(mutex_id.0)
                            .or_default()
                            .push(activity_id.0);
                    }
                    Direction::TwoWay => {
                        connection_activity_to_mutex
                            .entry(activity_id.0)
                            .or_default()
                            .push(mutex_id.0);
                        connection_mutex_to_activity
                            .entry(mutex_id.0)
                            .or_default()
                            .push(activity_id.0);
                    }
                }
            }
        }

        let mut csv = String::new();
        csv.push_str("Type;Position X;Position Y;ID;Parameters...\n");

        // add tasks
        csv.push_str("\"Task\";Position X;Position Y;ID;Task Name;Activity Name;Priority;Duration;Remaining Duration;Connected Mutex IDs...\n");
        for (activity_id, activity_node) in &self.activity_nodes {
            csv.push_str(&format!(
                "Task{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}\n",
                activity_node.pos.x,
                activity_node.pos.y,
                activity_id.0,
                activity_node.task_name,
                activity_node.activity_name,
                activity_node.priority,
                activity_node.duration,
                activity_node.remaining_duration,
                connection_activity_to_mutex
                    .get(&activity_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(seperator)
            ));
        }

        // add mutexes
        csv.push_str("\"Mutex\";Position X;Position Y;ID;Mutex Value;Connected Task IDs...\n");
        for (mutex_id, mutex_node) in &self.mutex_nodes {
            csv.push_str(&format!(
                "Mutex{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}\n",
                mutex_node.pos.x,
                mutex_node.pos.y,
                mutex_id.0,
                mutex_node.value,
                connection_mutex_to_activity
                    .get(&mutex_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(seperator)
            ));
        }

        csv
    }
}

// structure
impl Graph {
    pub fn add_activity_node(&mut self, activity_node: ActivityNode) -> ActivityNodeId {
        self.add_activiy_node_with_id(activity_node, self.next_activity_id)
    }
    pub fn add_activiy_node_with_id(
        &mut self,
        activity_node: ActivityNode,
        id: ActivityNodeId,
    ) -> ActivityNodeId {
        self.activity_nodes.insert(id, activity_node);
        *self.next_activity_id = usize::max(*self.next_mutex_id, *id + 1);
        id
    }

    pub fn add_mutex_node(&mut self, mutex_node: MutexNode) -> MutexNodeId {
        self.add_mutex_node_with_id(mutex_node, self.next_mutex_id)
    }
    pub fn add_mutex_node_with_id(
        &mut self,
        mutex_node: MutexNode,
        id: MutexNodeId,
    ) -> MutexNodeId {
        self.mutex_nodes.insert(id, mutex_node);
        *self.next_mutex_id = usize::max(*self.next_mutex_id, *id + 1);
        id
    }

    pub fn connect(
        &mut self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        direction: Direction,
        update_connections: bool,
    ) {
        let mut activity_connections = self.connections.remove(&activity_id).unwrap_or_default();
        let connection = match activity_connections.remove(&mutex_id) {
            Some(mut previous_connection) if previous_connection.get_direction() != direction => {
                let previous_direction = previous_connection.get_direction();
                previous_connection.set_direction(Direction::TwoWay);

                if update_connections {
                    if let (Some(activity_node), Some(mutex_node)) = (
                        self.activity_nodes.get(&activity_id),
                        self.mutex_nodes.get(&mutex_id),
                    ) {
                        match previous_direction {
                            Direction::MutexToActivity => {
                                previous_connection.tick_activity_to_mutex(activity_node);
                            }
                            Direction::ActivityToMutex => {
                                previous_connection
                                    .tick_mutex_to_activity(activity_node, mutex_node);
                            }
                            Direction::TwoWay => {}
                        }
                    }
                }

                previous_connection
            }
            Some(previous_connection) => previous_connection,
            None => {
                let mut connection = connection::Connection::new(direction);
                if update_connections {
                    if let (Some(activity_node), Some(mutex_node)) = (
                        self.activity_nodes.get(&activity_id),
                        self.mutex_nodes.get(&mutex_id),
                    ) {
                        connection.tick(activity_node, mutex_node);
                    }
                }
                connection
            }
        };
        activity_connections.insert(mutex_id, connection);
        self.connections.insert(activity_id, activity_connections);
    }

    pub fn disconnect(
        &mut self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        direction: Direction,
    ) {
        if let Some(mut activity_connections) = self.connections.remove(&activity_id) {
            if let Some(mut connection) = activity_connections.remove(&mutex_id) {
                if let Some(new_direction) = match (connection.get_direction(), direction) {
                    (Direction::MutexToActivity, Direction::ActivityToMutex) => {
                        Some(Direction::MutexToActivity)
                    }
                    (Direction::ActivityToMutex, Direction::MutexToActivity) => {
                        Some(Direction::ActivityToMutex)
                    }
                    (Direction::TwoWay, Direction::MutexToActivity) => {
                        Some(Direction::ActivityToMutex)
                    }
                    (Direction::TwoWay, Direction::ActivityToMutex) => {
                        Some(Direction::MutexToActivity)
                    }
                    _ => None,
                } {
                    connection.set_direction(new_direction);
                    activity_connections.insert(mutex_id, connection);
                }
            };
            self.connections.insert(activity_id, activity_connections);
        }
    }

    pub fn update_connection_states(&mut self) {
        self.do_per_connection(|connection, activity_node, mutex_node| {
            connection.tick(activity_node, mutex_node);
        });
    }

    pub fn is_connected(
        &self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        direction: Direction,
    ) -> bool {
        self.connections
            .get(&activity_id)
            .and_then(|activity_connections| activity_connections.get(&mutex_id))
            .map(|connection| {
                connection.get_direction() == direction
                    || connection.get_direction() == Direction::TwoWay
            })
            .unwrap_or(false)
    }

    pub fn toggle_connection(
        &mut self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        direction: Direction,
    ) {
        if self.is_connected(activity_id, mutex_id, direction) {
            self.disconnect(activity_id, mutex_id, direction);
        } else {
            self.connect(activity_id, mutex_id, direction, true);
        }
    }

    fn do_per_connection<F>(&mut self, mut action: F)
    where
        F: FnMut(&mut connection::Connection, &mut ActivityNode, &mut MutexNode),
    {
        self.connections
            .iter_mut()
            .for_each(|(activity_id, activity_connections)| {
                if let Some(activity_node) = self.activity_nodes.get_mut(activity_id) {
                    activity_connections
                        .iter_mut()
                        .for_each(|(mutex_id, connection)| {
                            if let Some(mutex_node) = self.mutex_nodes.get_mut(mutex_id) {
                                action(connection, activity_node, mutex_node);
                            }
                        });
                }
            });
    }

    fn new_random_activity(pos: Pos2) -> ActivityNode {
        let mut activity_node = ActivityNode::new(pos);
        activity_node.activity_name = "Activity".to_string();
        activity_node.task_name = random_word::gen_len(thread_rng().gen_range(4..=8), Lang::En)
            .map_or("Task".to_string(), |s| {
                // capitalize first character of the word
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first_char) => {
                        first_char.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            });
        activity_node.duration = 1;
        activity_node.priority = 0;
        activity_node
    }
}

// simulation
impl Graph {
    pub fn tick(&mut self, ui: &egui::Ui) {
        if self.remaining_ticks_to_run != 0 {
            let mut previous_tick_progress = self.tick_progress;
            self.tick_progress += ui.ctx().input(|i| i.stable_dt) * self.ticks_per_second;
            ui.ctx().request_repaint(); // keep the simulation running
            loop {
                if previous_tick_progress < 0.5 && self.tick_progress >= 0.5 {
                    self.tick_a();
                    self.do_per_connection(|c, a, m| {
                        c.tick(a, m);
                    });
                }
                if self.tick_progress >= 1. {
                    self.tick_b();
                    self.do_per_connection(|c, a, m| {
                        c.tick(a, m);
                    });

                    self.tick_progress -= 1.;
                    if self.remaining_ticks_to_run > 0 {
                        self.remaining_ticks_to_run -= 1;
                        if self.remaining_ticks_to_run == 0 {
                            self.tick_progress = 0.;
                        }
                    }

                    // make sure tick_a() is called
                    previous_tick_progress = 0.;
                } else {
                    break;
                }
            }
        }
    }

    fn tick_a(&mut self) {
        let base_seed = rand::random::<u64>();
        self.activity_nodes
            .sort_by(|&id_1, activity_node_1, &id_2, activity_node_2| {
                match activity_node_1.priority.cmp(&activity_node_2.priority) {
                    // randomize if priority is the same
                    std::cmp::Ordering::Equal => {
                        // random bool based on base_seed, constant for every node pair
                        let random_bool = (base_seed.wrapping_pow((*id_1 + *id_2) as u32) & 1) == 0;

                        // flip if id_1 < id_2, so cmp(a, b) is always the opposite of cmp(b, a)
                        // --> there is a consistent order between all nodes for a given base_seed
                        let node_1_is_greater = random_bool ^ (*id_1 < *id_2);

                        match node_1_is_greater {
                            true => std::cmp::Ordering::Greater,
                            false => std::cmp::Ordering::Less,
                        }
                    }
                    ordering => ordering,
                }
            });
        self.activity_nodes
            .iter_mut()
            .rev()
            .for_each(|(activity_id, activity_node)| {
                if activity_node.remaining_duration > 0 {
                    return;
                }

                let activity_connections = self.connections.get(activity_id);

                // check if prerequisites are met
                let prerequisites_missing =
                    activity_connections.map_or(false, |activity_connections| {
                        activity_connections
                            .iter()
                            .filter(|(_, connection)| {
                                connection.get_direction() != Direction::ActivityToMutex
                            })
                            .filter_map(|(mutex_id, _)| self.mutex_nodes.get(mutex_id))
                            .any(|mutex_node| mutex_node.value == 0)
                    });
                if prerequisites_missing {
                    return;
                }

                // start the node
                activity_node.remaining_duration = activity_node.duration;

                // decrement prerequisites
                if let Some(activity_connections) = activity_connections {
                    activity_connections
                        .iter()
                        .for_each(|(mutex_id, connection)| {
                            if connection.get_direction() != Direction::ActivityToMutex {
                                if let Some(mutex_node) = self.mutex_nodes.get_mut(mutex_id) {
                                    mutex_node.value -= 1;
                                }
                            }
                        });
                }
            });

        // return to predictable order for drawing the ui
        self.activity_nodes.sort_unstable_keys();
    }

    fn tick_b(&mut self) {
        for (activity_id, activity_node) in &mut self.activity_nodes {
            if activity_node.remaining_duration == 0 {
                continue;
            }
            activity_node.remaining_duration -= 1;

            if activity_node.remaining_duration == 0 {
                if let Some(activity_connections) = self.connections.get(activity_id) {
                    // increment all outputs
                    activity_connections
                        .iter()
                        .for_each(|(mutex_id, connection)| {
                            if connection.get_direction() != Direction::MutexToActivity {
                                if let Some(mutex_node) = self.mutex_nodes.get_mut(mutex_id) {
                                    mutex_node.value += 1;
                                }
                            }
                        })
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.remaining_ticks_to_run < 0
    }

    pub fn toggle_play_pause(&mut self) {
        self.remaining_ticks_to_run = match self.remaining_ticks_to_run {
            -1 => 1,
            _ => -1,
        }
    }

    pub fn queue_tick(&mut self) {
        if self.remaining_ticks_to_run >= 0 {
            self.set_remaining_ticks_to_run(self.remaining_ticks_to_run + 1);
        }
    }

    pub fn set_remaining_ticks_to_run(&mut self, ticks: i32) {
        self.remaining_ticks_to_run = ticks;
    }

    pub fn get_remaining_ticks_to_run(&self) -> i32 {
        self.remaining_ticks_to_run
    }
}

// ux
impl Graph {
    pub fn queue_autofit(&mut self) {
        self.autofit_rect = Some(egui::Rect::NAN);
    }

    pub fn interact(
        &mut self,
        ui: &mut egui::Ui,
        container_transform: &mut egui::emath::TSTransform,
        container_response: &egui::Response,
    ) {
        // autofit
        if container_response.triple_clicked() && self.editing_mode != EditingMode::Delete {
            self.queue_autofit();
        }

        if let Some(last_autofit_frame_rect) = self.autofit_rect {
            let untransformed_viewport_rect = container_response.rect;
            if last_autofit_frame_rect == untransformed_viewport_rect
                && untransformed_viewport_rect.width() >= 150.
                && untransformed_viewport_rect.height() >= 150.
            {
                self.autofit_rect = None;
            } else if untransformed_viewport_rect.is_positive() {
                self.autofit_rect = Some(untransformed_viewport_rect);
                let mut bounding_rect = egui::Rect::NOTHING;
                self.activity_nodes.iter().for_each(|(_, node)| {
                    let rect = egui::Rect::from_center_size(node.pos, egui::vec2(150., 100.));
                    bounding_rect = bounding_rect.union(rect);
                });
                self.mutex_nodes.iter().for_each(|(_, node)| {
                    let rect = egui::Rect::from_center_size(node.pos, egui::vec2(50., 50.));
                    bounding_rect = bounding_rect.union(rect);
                });

                if bounding_rect.is_positive() {
                    let scale_x = untransformed_viewport_rect.width() / bounding_rect.width();
                    let scale_y = untransformed_viewport_rect.height() / bounding_rect.height();
                    container_transform.scaling = scale_x.min(scale_y).min(1.8);
                    container_transform.translation = egui::Vec2::ZERO;
                    container_transform.translation =
                        untransformed_viewport_rect.center().to_vec2()
                            - (*container_transform * bounding_rect.center()).to_vec2();
                } else {
                    *container_transform = TSTransform::default();
                }
            }
        }

        // node interactions
        let mut node_left_clicked = None;
        let mut node_right_clicked = None;
        self.activity_nodes.iter_mut().for_each(|(id, node)| {
            if let Some(response) = node.interact(ui) {
                if response.clicked() {
                    node_left_clicked = Some(AnyNode::Activity(*id));
                }
                if response.secondary_clicked() {
                    node_right_clicked = Some(AnyNode::Activity(*id));
                }
            }
        });
        self.mutex_nodes.iter_mut().for_each(|(id, node)| {
            if let Some(response) = node.interact(ui) {
                if response.clicked() {
                    node_left_clicked = Some(AnyNode::Mutex(*id));
                }
                if response.secondary_clicked() {
                    node_right_clicked = Some(AnyNode::Mutex(*id));
                }
            }
        });
        if self.currently_connecting_from.is_none() {
            self.currently_connecting_from = node_right_clicked;
            node_right_clicked = None;
        }

        match self.editing_mode {
            EditingMode::Delete => {
                if container_response.secondary_clicked() {
                    self.editing_mode = EditingMode::None;
                    return;
                }
                if let Some(AnyNode::Activity(id)) = node_left_clicked {
                    self.activity_nodes.swap_remove(&id);
                    self.connections.remove(&id);
                }
                if let Some(AnyNode::Mutex(id)) = node_left_clicked {
                    self.mutex_nodes.remove(&id);
                    self.connections.iter_mut().for_each(|(_, connections)| {
                        connections.remove(&id);
                    });
                }
            }
            EditingMode::None => {
                // click existing node
                if let Some(new_from_node) = match (
                    self.currently_connecting_from,
                    node_left_clicked.or(node_right_clicked),
                ) {
                    (
                        Some(AnyNode::Activity(from_activity_id)),
                        Some(AnyNode::Mutex(to_mutex_id)),
                    ) => {
                        self.toggle_connection(
                            from_activity_id,
                            to_mutex_id,
                            Direction::ActivityToMutex,
                        );
                        Some(AnyNode::Mutex(to_mutex_id))
                    }
                    (
                        Some(AnyNode::Mutex(from_mutex_id)),
                        Some(AnyNode::Activity(to_activity_id)),
                    ) => {
                        self.toggle_connection(
                            to_activity_id,
                            from_mutex_id,
                            Direction::MutexToActivity,
                        );
                        Some(AnyNode::Activity(to_activity_id))
                    }
                    (
                        Some(AnyNode::Activity(from_activity_id)),
                        Some(AnyNode::Activity(to_activity_id)),
                    ) => {
                        if let (Some(from_activity), Some(to_activity)) = (
                            self.activity_nodes.get(&from_activity_id),
                            self.activity_nodes.get(&to_activity_id),
                        ) {
                            let mutex_pos = from_activity.pos / 2. + to_activity.pos.to_vec2() / 2.;
                            let mutex_id = self.add_mutex_node(MutexNode::new(mutex_pos));
                            self.connect(
                                from_activity_id,
                                mutex_id,
                                Direction::ActivityToMutex,
                                true,
                            );
                            self.connect(
                                to_activity_id,
                                mutex_id,
                                Direction::MutexToActivity,
                                true,
                            );
                        }
                        Some(AnyNode::Activity(to_activity_id))
                    }
                    (Some(AnyNode::Mutex(from_mutex_id)), Some(AnyNode::Mutex(to_mutex_id))) => {
                        if let (Some(from_mutex), Some(to_mutex)) = (
                            self.mutex_nodes.get(&from_mutex_id),
                            self.mutex_nodes.get(&to_mutex_id),
                        ) {
                            let activity_pos = from_mutex.pos / 2. + to_mutex.pos.to_vec2() / 2.;
                            let activity_id =
                                self.add_activity_node(Graph::new_random_activity(activity_pos));
                            self.connect(
                                activity_id,
                                from_mutex_id,
                                Direction::MutexToActivity,
                                true,
                            );
                            self.connect(
                                activity_id,
                                to_mutex_id,
                                Direction::ActivityToMutex,
                                true,
                            );
                        }
                        Some(AnyNode::Mutex(to_mutex_id))
                    }
                    _ => None,
                } {
                    self.currently_connecting_from = match node_left_clicked.is_some() {
                        true => None,
                        false => Some(new_from_node),
                    };
                }

                // right click empty space (create nodes)
                if container_response.secondary_clicked() {
                    if let Some(pos) = container_response.interact_pointer_pos() {
                        let pos = container_transform.inverse() * pos;
                        match self.currently_connecting_from {
                            Some(AnyNode::Mutex(mutex_id)) => {
                                let activity_id =
                                    self.add_activity_node(Graph::new_random_activity(pos));
                                self.connect(
                                    activity_id,
                                    mutex_id,
                                    Direction::MutexToActivity,
                                    true,
                                );
                                self.currently_connecting_from =
                                    Some(AnyNode::Activity(activity_id));
                            }
                            Some(AnyNode::Activity(activity_id)) => {
                                let mutex_id = self.add_mutex_node(MutexNode::new(pos));
                                self.connect(
                                    activity_id,
                                    mutex_id,
                                    Direction::ActivityToMutex,
                                    true,
                                );
                                self.currently_connecting_from = Some(AnyNode::Mutex(mutex_id));
                            }
                            None => {
                                self.add_activity_node(Graph::new_random_activity(pos));
                            }
                        }
                    }
                }

                // left click empty space (click away)
                if container_response.clicked() {
                    self.currently_connecting_from = None;
                }

                // draw connection preview
                if let Some(pointer_pos) = ui.input(|i| i.pointer.latest_pos()) {
                    match self.currently_connecting_from {
                        Some(AnyNode::Mutex(id)) => {
                            if let Some(node) = self.mutex_nodes.get(&id) {
                                connection::Connection::draw_arrow(
                                    ui,
                                    node.pos,
                                    container_transform.inverse() * pointer_pos,
                                    connection::Color::Default,
                                    connection::Color::Default,
                                    0.,
                                );
                            }
                        }
                        Some(AnyNode::Activity(id)) => {
                            if let Some(node) = self.activity_nodes.get(&id) {
                                connection::Connection::draw_arrow(
                                    ui,
                                    node.pos,
                                    container_transform.inverse() * pointer_pos,
                                    connection::Color::Default,
                                    connection::Color::Default,
                                    0.,
                                );
                            }
                        }
                        None => (),
                    };
                }
            }
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, container_transform: egui::emath::TSTransform) {
        ui.style_mut().spacing.interact_size = egui::Vec2::ZERO;
        ui.style_mut().spacing.button_padding = egui::Vec2::ZERO;
        ui.style_mut().interaction.multi_widget_text_select = false;

        // draw
        let tick_progress = self.tick_progress;
        self.do_per_connection(|c, a, m| c.draw(ui, a, m, tick_progress));
        self.mutex_nodes
            .iter_mut()
            .for_each(|n| n.1.draw(ui, container_transform));
        self.activity_nodes
            .iter_mut()
            .for_each(|(_, activity_node)| {
                activity_node.draw(ui, container_transform, tick_progress)
            });
    }
}

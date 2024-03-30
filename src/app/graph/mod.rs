mod activity_node;
pub mod connection;
mod mutex_node;

use std::{fs::File, io};

pub use activity_node::ActivityNode;
pub use mutex_node::MutexNode;

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

#[derive(Clone, Copy)]
enum AnyNode {
    Activity(ActivityNodeId),
    Mutex(MutexNodeId),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Graph {
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

    pub remaining_ticks_to_run: i32,

    #[serde(skip)]
    currently_connecting_from: Option<AnyNode>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            activity_nodes: indexmap::IndexMap::new(),
            mutex_nodes: std::collections::HashMap::new(),
            connections: std::collections::HashMap::new(),
            next_activity_id: ActivityNodeId(0),
            next_mutex_id: MutexNodeId(0),
            tick_progress: 0.,
            ticks_per_second: 1.,
            remaining_ticks_to_run: -1,
            currently_connecting_from: None,
        }
    }
}

// import/export
impl Graph {
    pub fn from_csv(lines: io::Lines<io::BufReader<File>>) -> Result<Self, Box<String>> {
        const SEPERATOR: char = ';';
        let mut graph = Graph::default();

        for (line_number, line) in lines.flatten().enumerate() {
            let line_number = line_number + 1; // enumerate starts at 0

            // split returns at least 1 empty string -> subsequent values[0] are fine
            let values = line.split(SEPERATOR).map(|s| s.trim()).collect::<Vec<_>>();

            // match first value to determine type of line
            match values[0].to_lowercase().as_str() {
                "task" if values.len() >= 6 => {
                    let activity_id =
                        ActivityNodeId(values[1].parse::<usize>().map_err(|_| {
                            format!("Error while parsing ID in line: {}", line_number)
                        })?);

                    let mut activity_node = ActivityNode::new(egui::Pos2 { x: 0., y: 0. });
                    activity_node.task_name = values[2].to_string();
                    activity_node.activity_name = values[3].to_string();
                    activity_node.duration = values[4].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Duration in line: {}", line_number)
                    })?;
                    activity_node.priority = values[5].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Priority in line: {}", line_number)
                    })?;
                    graph.add_activiy_node_with_id(activity_node, activity_id);

                    values[6..]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .find_map(|x| match x.parse::<usize>() {
                            Ok(mutex_id) => {
                                graph.connect(
                                    activity_id,
                                    MutexNodeId(mutex_id),
                                    Direction::ActivityToMutex,
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

                "mutex" if values.len() >= 3 => {
                    let mutex_id =
                        MutexNodeId(values[1].parse::<usize>().map_err(|_| {
                            format!("Error while parsing ID in line: {}", line_number)
                        })?);

                    let mut mutex_node = MutexNode::new(egui::Pos2 { x: 0., y: 0. });
                    mutex_node.value = values[2].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Value in line: {}", line_number)
                    })?;
                    graph.add_mutex_node_with_id(mutex_node, mutex_id);

                    values[3..]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .find_map(|x| match x.parse::<usize>() {
                            Ok(activity_id) => {
                                graph.connect(
                                    ActivityNodeId(activity_id),
                                    mutex_id,
                                    Direction::MutexToActivity,
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
        return Ok(graph);
    }

    pub fn to_csv(&self) -> String {
        use std::collections::HashMap;
        let seperator = ";";

        let mut connection_activity_to_mutex: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut connection_mutex_to_activity: HashMap<usize, Vec<usize>> = HashMap::new();

        for (activity_id, activity_connections) in &self.connections {
            for (mutex_id, connection) in activity_connections {
                match connection.direction {
                    Direction::ActivityToMutex => {
                        connection_activity_to_mutex
                            .entry(activity_id.0)
                            .or_insert_with(Vec::new)
                            .push(mutex_id.0);
                    }
                    Direction::MutexToActivity => {
                        connection_mutex_to_activity
                            .entry(mutex_id.0)
                            .or_insert_with(Vec::new)
                            .push(activity_id.0);
                    }
                    Direction::TwoWay => {
                        connection_activity_to_mutex
                            .entry(activity_id.0)
                            .or_insert_with(Vec::new)
                            .push(mutex_id.0);
                        connection_mutex_to_activity
                            .entry(mutex_id.0)
                            .or_insert_with(Vec::new)
                            .push(activity_id.0);
                    }
                }
            }
        }

        let mut csv = String::new();
        //add header
        csv.push_str("Type,ID,Parameters\n");

        // add tasks
        for (activity_id, activity_node) in &self.activity_nodes {
            csv.push_str(&format!(
                "Task{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}{seperator}{}\n",
                activity_id.0,
                activity_node.task_name,
                activity_node.activity_name,
                activity_node.duration,
                activity_node.priority,
                connection_activity_to_mutex
                    .get(&activity_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(seperator)
            ));
        }

        // add mutexes
        for (mutex_id, mutex_node) in &self.mutex_nodes {
            csv.push_str(&format!(
                "Mutex{seperator}{}{seperator}{}{seperator}{}\n",
                mutex_id.0,
                mutex_node.value,
                connection_mutex_to_activity
                    .get(&mutex_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(seperator)
            ));
        }

        return csv;
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
    ) {
        let mut activity_connections = self.connections.remove(&activity_id).unwrap_or_default();
        let connection = match activity_connections.remove(&mutex_id) {
            Some(mut previous_connection) if previous_connection.direction != direction => {
                previous_connection.direction = Direction::TwoWay;
                previous_connection
            }
            Some(previous_connection) => previous_connection,
            None => connection::Connection::new(direction),
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
                if let Some(new_direction) = match (connection.direction, direction) {
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
                    connection.direction = new_direction;
                    activity_connections.insert(mutex_id, connection);
                }
            };
            self.connections.insert(activity_id, activity_connections);
        }
    }

    pub fn is_connected(
        &self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        direction: Direction,
    ) -> bool {
        self.connections
            .get(&activity_id)
            .map(|activity_connections| activity_connections.get(&mutex_id))
            .flatten()
            .map(|connection| {
                connection.direction == direction || connection.direction == Direction::TwoWay
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
            self.connect(activity_id, mutex_id, direction);
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
}

// simulation
impl Graph {
    pub fn tick(&mut self, ui: &egui::Ui) {
        if self.remaining_ticks_to_run != 0 {
            let mut previous_tick_progress = self.tick_progress;
            self.tick_progress += ui.ctx().input(|i| i.stable_dt) * self.ticks_per_second;
            loop {
                if previous_tick_progress < 0.5 && self.tick_progress >= 0.5 {
                    self.tick_a();
                    self.do_per_connection(|c, a, m| c.tick(a, m));
                }
                if self.tick_progress >= 1. {
                    self.tick_b();
                    self.do_per_connection(|c, a, m| c.tick(a, m));

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

    pub fn queue_tick(&mut self) {
        if self.remaining_ticks_to_run >= 0 {
            self.remaining_ticks_to_run += 1;
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

    fn single_tick(&mut self) {
        self.tick_a();
        self.do_per_connection(|c, a, m| c.tick(a, m));
        self.tick_b();
        self.do_per_connection(|c, a, m| c.tick(a, m));
    }

    fn tick_a(&mut self) {
        self.activity_nodes
            .sort_unstable_by(|_, activity_node_1, _, activity_node_2| {
                match activity_node_1.priority.cmp(&activity_node_2.priority) {
                    // randomize if priority is the same
                    std::cmp::Ordering::Equal => match rand::random::<bool>() {
                        true => std::cmp::Ordering::Greater,
                        false => std::cmp::Ordering::Less,
                    },
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

                if let Some(activity_connections) = self.connections.get(&activity_id) {
                    // check if prerequisites are met
                    let prerequisites_missing = activity_connections
                        .iter()
                        .filter(|(_, connection)| {
                            connection.direction != Direction::ActivityToMutex
                        })
                        .filter_map(|(mutex_id, _)| self.mutex_nodes.get(mutex_id))
                        .find(|mutex_node| mutex_node.value <= 0)
                        .is_some();

                    if prerequisites_missing {
                        return;
                    }

                    // start the node
                    activity_node.remaining_duration = activity_node.duration;

                    // decrement prerequisites
                    activity_connections
                        .iter()
                        .for_each(|(mutex_id, connection)| {
                            if connection.direction != Direction::ActivityToMutex {
                                self.mutex_nodes
                                    .get_mut(mutex_id)
                                    .map(|mutex_node| mutex_node.value -= 1);
                            }
                        })
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
                if let Some(activity_connections) = self.connections.get(&activity_id) {
                    // increment all outputs
                    activity_connections
                        .iter()
                        .for_each(|(mutex_id, connection)| {
                            if connection.direction != Direction::MutexToActivity {
                                self.mutex_nodes
                                    .get_mut(mutex_id)
                                    .map(|mutex_node| mutex_node.value += 1);
                            }
                        })
                }
            }
        }
    }
}

// ux
impl Graph {
    pub fn interact(
        &mut self,
        ui: &mut egui::Ui,
        container_transform: egui::emath::TSTransform,
        container_response: &egui::Response,
    ) {
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

        // click existing node
        if let Some(new_from_node) = match (
            self.currently_connecting_from,
            node_left_clicked.or(node_right_clicked),
        ) {
            (Some(AnyNode::Activity(from_activity_id)), Some(AnyNode::Mutex(to_mutex_id))) => {
                self.toggle_connection(from_activity_id, to_mutex_id, Direction::ActivityToMutex);
                Some(AnyNode::Mutex(to_mutex_id))
            }
            (Some(AnyNode::Mutex(from_mutex_id)), Some(AnyNode::Activity(to_activity_id))) => {
                self.toggle_connection(to_activity_id, from_mutex_id, Direction::MutexToActivity);
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
                    self.connect(from_activity_id, mutex_id, Direction::ActivityToMutex);
                    self.connect(to_activity_id, mutex_id, Direction::MutexToActivity);
                }
                Some(AnyNode::Activity(to_activity_id))
            }
            (Some(AnyNode::Mutex(from_mutex_id)), Some(AnyNode::Mutex(to_mutex_id))) => {
                if let (Some(from_mutex), Some(to_mutex)) = (
                    self.mutex_nodes.get(&from_mutex_id),
                    self.mutex_nodes.get(&to_mutex_id),
                ) {
                    let activity_pos = from_mutex.pos / 2. + to_mutex.pos.to_vec2() / 2.;
                    let activity_id = self.add_activity_node(ActivityNode::new(activity_pos));
                    self.connect(activity_id, from_mutex_id, Direction::MutexToActivity);
                    self.connect(activity_id, to_mutex_id, Direction::ActivityToMutex);
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
                        let activity_id = self.add_activity_node(ActivityNode::new(pos));
                        self.connect(activity_id, mutex_id, Direction::MutexToActivity);
                        self.currently_connecting_from = Some(AnyNode::Activity(activity_id));
                    }
                    Some(AnyNode::Activity(activity_id)) => {
                        let mutex_id = self.add_mutex_node(MutexNode::new(pos));
                        self.connect(activity_id, mutex_id, Direction::ActivityToMutex);
                        self.currently_connecting_from = Some(AnyNode::Mutex(mutex_id));
                    }
                    None => {
                        self.add_activity_node(ActivityNode::new(pos));
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

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.interact_size = egui::Vec2::ZERO;
        ui.style_mut().spacing.button_padding = egui::Vec2::ZERO;
        ui.style_mut().interaction.multi_widget_text_select = false;

        // draw
        let tick_progress = self.tick_progress;
        self.do_per_connection(|c, a, m| c.draw(ui, a, m, tick_progress));
        self.mutex_nodes.iter_mut().for_each(|n| n.1.draw(ui));
        self.activity_nodes
            .iter_mut()
            .for_each(|(_, activity_node)| activity_node.draw(ui));
    }
}

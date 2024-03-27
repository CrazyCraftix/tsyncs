mod activity_node;
mod mutex_node;

use std::{fs::File, io};

pub use activity_node::ActivityNode;
pub use mutex_node::MutexNode;

#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityNodeId(usize);
#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutexNodeId(usize);

#[derive(PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionType {
    MutexToActivity,
    ActivityToMutex,
    TwoWay,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Graph {
    activity_nodes: std::collections::HashMap<ActivityNodeId, ActivityNode>,
    mutex_nodes: std::collections::HashMap<MutexNodeId, MutexNode>,

    connections: std::collections::HashMap<
        ActivityNodeId,
        std::collections::HashMap<MutexNodeId, ConnectionType>,
    >,

    next_activity_id: ActivityNodeId,
    next_mutex_id: MutexNodeId,

    time_since_last_tick_in_seconds: f32,
}

// structure
impl Graph {
    pub fn from_csv(lines: io::Lines<io::BufReader<File>>) -> Result<Self, Box<String>> {
        let seperator = ',';
        let graph = Graph::default();
        for (line_number, line) in lines.flatten().enumerate() {
            let mut values = line.split(seperator).collect::<Vec<&str>>();

            if values.len() < 5 {
                continue;
            }

            // match first value to determine type of line
            match values[0].to_lowercase().as_str() {
                "task" => {
                    let id = values[1]
                        .trim()
                        .parse::<usize>()
                        .map_err(|_| format!("Error while parsing ID in line: {}", line_number))?;
                    let task_name = values[2].to_string();
                    let activity_name = values[3].to_string();
                    let duration = values[3].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Duration in line: {}", line_number)
                    })?;
                    let priority = values[4].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Priority in line: {}", line_number)
                    })?;
                    let mutex_connections = values[5..]
                        .iter()
                        .map(|x| {
                            x.parse::<usize>().map_err(|_| {
                                format!(
                                    "Error while parsing Mutex Connection in line: {}",
                                    line_number
                                )
                            })
                        })
                        .collect::<Result<Vec<usize>, String>>()?;
                }
                "mutex" => {
                    let id = values[1].parse::<usize>().expect("Error while parsing ID");
                    let value = values[2].parse::<u32>().expect("Error while parsing Value");
                    let activity_connections = values[3..]
                        .iter()
                        .map(|x| {
                            x.parse::<u32>()
                                .expect("Error while parsing Activity Connection")
                        })
                        .collect::<Vec<u32>>();
                }
                _ => {
                    // skip line
                }
            }
        }
        Ok(Self::default())
    }

    pub fn to_csv(&self) -> String {
        use std::collections::HashMap;

        let mut connection_activity_to_mutex: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut connection_mutex_to_activity: HashMap<usize, Vec<usize>> = HashMap::new();

        for (activity_id, activity_connections) in &self.connections {
            for (mutex_id, connection_type) in activity_connections {
                match connection_type {
                    ConnectionType::ActivityToMutex => {
                        connection_activity_to_mutex
                            .entry(activity_id.0)
                            .or_insert_with(Vec::new)
                            .push(mutex_id.0);
                    }
                    ConnectionType::MutexToActivity => {
                        connection_mutex_to_activity
                            .entry(mutex_id.0)
                            .or_insert_with(Vec::new)
                            .push(activity_id.0);
                    }
                    ConnectionType::TwoWay => {
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
                "Task,{},{},{},{},{},{}\n",
                activity_id.0,
                activity_node.task_name,
                activity_node.activity_name,
                activity_node.duration,
                0, // priority
                connection_activity_to_mutex
                    .get(&activity_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(",")
            ));
        }

        // add mutexes
        for (mutex_id, mutex_node) in &self.mutex_nodes {
            csv.push_str(&format!(
                "Mutex,{},{},{}\n",
                mutex_id.0,
                mutex_node.value,
                connection_mutex_to_activity
                    .get(&mutex_id.0)
                    .map(|x| x.iter().map(|x| x.to_string()).collect::<Vec<String>>())
                    .unwrap_or_default()
                    .join(",")
            ));
        }

        return csv;
    }

    pub fn add_activiy_node(&mut self, activity_node: ActivityNode) -> ActivityNodeId {
        let id = self.next_activity_id;
        self.activity_nodes.insert(id, activity_node);
        self.next_activity_id = ActivityNodeId(id.0 + 1);
        id
    }

    pub fn add_mutex_node(&mut self, mutex_node: MutexNode) -> MutexNodeId {
        let id = self.next_mutex_id;
        self.mutex_nodes.insert(id, mutex_node);
        self.next_mutex_id = MutexNodeId(id.0 + 1);
        id
    }

    pub fn connect(
        &mut self,
        activity_id: ActivityNodeId,
        mutex_id: MutexNodeId,
        connection_type: ConnectionType,
    ) -> bool {
        if !self.mutex_nodes.contains_key(&mutex_id)
            || !self.activity_nodes.contains_key(&activity_id)
        {
            return false;
        }

        let mut activity_connections = self.connections.remove(&activity_id).unwrap_or_default();

        let new_connection_type = match activity_connections.remove(&mutex_id) {
            Some(previous_connection_type) if previous_connection_type != connection_type => {
                ConnectionType::TwoWay
            }
            Some(previous_connection_type) => previous_connection_type,
            None => connection_type,
        };

        activity_connections.insert(mutex_id, new_connection_type);
        self.connections.insert(activity_id, activity_connections);

        true
    }
}

// simulation
impl Graph {
    fn tick(&mut self) {
        for (activity_id, activity_node) in &mut self.activity_nodes {
            if activity_node.remaining_duration > 0 {
                continue;
            }

            if let Some(activity_connections) = self.connections.get(&activity_id) {
                // check if prerequisites are met
                let prerequisites_missing = activity_connections
                    .iter()
                    .filter(|(_, connection_type)| {
                        connection_type != &&ConnectionType::ActivityToMutex
                    })
                    .filter_map(|(mutex_id, _)| self.mutex_nodes.get(mutex_id))
                    .find(|mutex_node| mutex_node.value <= 0)
                    .is_some();

                if prerequisites_missing {
                    continue;
                }

                // start the node
                activity_node.remaining_duration = activity_node.duration;

                // decrement prerequisites
                activity_connections
                    .iter()
                    .for_each(|(mutex_id, connection_type)| {
                        if connection_type != &ConnectionType::ActivityToMutex {
                            self.mutex_nodes
                                .get_mut(mutex_id)
                                .map(|mutex_node| mutex_node.value -= 1);
                        }
                    })
            }
        }

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
                        .for_each(|(mutex_id, connection_type)| {
                            if connection_type != &ConnectionType::MutexToActivity {
                                self.mutex_nodes
                                    .get_mut(mutex_id)
                                    .map(|mutex_node| mutex_node.value += 1);
                            }
                        })
                }
            }
        }
    }

    fn resolve_connections<'a>(
        activity_connections: &'a std::collections::HashMap<MutexNodeId, ConnectionType>,
        mutex_nodes: &'a std::collections::HashMap<MutexNodeId, MutexNode>,
    ) -> Vec<(&'a MutexNode, &'a ConnectionType)> {
        activity_connections
            .iter()
            .filter_map(|(mutex_id, connection_type)| {
                mutex_nodes
                    .get(mutex_id)
                    .map(|mutex_node| (mutex_node, connection_type))
            })
            .collect::<Vec<_>>()
    }
}

// drawing
impl Graph {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.time_since_last_tick_in_seconds += ui.ctx().input(|i| i.stable_dt);
        if self.time_since_last_tick_in_seconds >= 1. {
            self.time_since_last_tick_in_seconds -= 1.;
            self.tick();
        }

        ui.style_mut().spacing.interact_size = egui::Vec2::ZERO;
        ui.style_mut().spacing.button_padding = egui::Vec2::ZERO;
        ui.style_mut().interaction.multi_widget_text_select = false;

        // interact
        for (_, node) in &mut self.activity_nodes {
            node.interact(ui);
        }

        for (_, node) in &mut self.mutex_nodes {
            node.interact(ui);
        }

        // draw
        self.connections
            .iter()
            .filter_map(|(activity_id, activity_connections)| {
                self.activity_nodes.get(activity_id).map(|activity_node| {
                    (
                        activity_node,
                        Self::resolve_connections(activity_connections, &self.mutex_nodes),
                    )
                })
            })
            .for_each(|(activity_node, resolved_activity_connections)| {
                Self::draw_connections(ui, activity_node, &resolved_activity_connections)
            });
        self.mutex_nodes.iter_mut().for_each(|n| n.1.draw(ui));
        self.activity_nodes
            .iter_mut()
            .for_each(|(_, activity_node)| activity_node.draw(ui));
    }

    fn draw_connections(
        ui: &mut egui::Ui,
        activity_node: &ActivityNode,
        connections: &Vec<(&MutexNode, &ConnectionType)>,
    ) {
        let colors = vec![egui::Color32::LIGHT_GRAY, egui::Color32::DARK_GRAY];

        connections
            .iter()
            .for_each(|(mutex_node, connection_type)| {
                let colors_mutex_to_activity = match mutex_node.value {
                    0 => vec![
                        egui::Color32::LIGHT_GRAY,
                        egui::Color32::DARK_RED,
                        egui::Color32::LIGHT_GRAY,
                        egui::Color32::RED,
                    ],
                    _ => vec![
                        egui::Color32::LIGHT_GRAY,
                        egui::Color32::DARK_GREEN,
                        egui::Color32::LIGHT_GRAY,
                        egui::Color32::GREEN,
                    ],
                };
                match connection_type {
                    ConnectionType::MutexToActivity => {
                        Self::draw_connection(
                            ui,
                            mutex_node.pos,
                            activity_node.pos,
                            &colors_mutex_to_activity,
                        );
                    }
                    ConnectionType::ActivityToMutex => {
                        Self::draw_connection(ui, activity_node.pos, mutex_node.pos, &colors);
                    }
                    ConnectionType::TwoWay => {
                        let offset = (activity_node.pos - mutex_node.pos).normalized().rot90() * 6.;
                        Self::draw_connection(
                            ui,
                            activity_node.pos + offset,
                            mutex_node.pos + offset,
                            &colors,
                        );
                        Self::draw_connection(
                            ui,
                            mutex_node.pos - offset,
                            activity_node.pos - offset,
                            &colors_mutex_to_activity,
                        );
                    }
                }
            });
    }

    fn draw_connection(
        ui: &mut egui::Ui,
        from_point: egui::Pos2,
        to_point: egui::Pos2,
        colors: &Vec<egui::Color32>,
    ) {
        const WIDTH: f32 = 7.;
        const ARROW_SPACING: f32 = 8.;
        const ARROW_DEPTH: f32 = 3.;
        const SCROLL_SPEED_IN_POINTS_PER_SECOND: f32 = 4.;

        ui.ctx().request_repaint();
        let time_offset = ui.input(|i| i.time) as f32 * SCROLL_SPEED_IN_POINTS_PER_SECOND
            % (ARROW_SPACING * colors.len() as f32);
        let color_offset = -(time_offset / ARROW_SPACING) as i32;

        let from_to_vector = to_point - from_point;
        let from_to_unit_vector = from_to_vector.normalized();
        let line_center_point =
            from_point + 0.5 * from_to_vector + (time_offset % ARROW_SPACING) * from_to_unit_vector;
        let half_arrow_count = (from_to_vector.length() / 2. / ARROW_SPACING) as i32;

        let arrow_tip_to_arrow_top_right =
            -ARROW_DEPTH * from_to_unit_vector + from_to_unit_vector.rot90() * (WIDTH / 2.);
        let arrow_tip_to_arrow_top_left =
            arrow_tip_to_arrow_top_right - from_to_unit_vector.rot90() * WIDTH;

        for i in ((-half_arrow_count + 1)..=half_arrow_count).rev() {
            let arrow_tip = line_center_point + i as f32 * ARROW_SPACING * from_to_unit_vector;
            let arrow_top_left = arrow_tip + arrow_tip_to_arrow_top_left;
            let arrow_top_right = arrow_tip + arrow_tip_to_arrow_top_right;
            let arrow_bottom_left = arrow_top_left - from_to_unit_vector * ARROW_SPACING;
            let arrow_bottom_right = arrow_top_right - from_to_unit_vector * ARROW_SPACING;
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    arrow_bottom_left,
                    arrow_top_left,
                    arrow_tip,
                    arrow_top_right,
                    arrow_bottom_right,
                ],
                colors[(i + color_offset).rem_euclid(colors.len() as i32) as usize],
                egui::Stroke::NONE,
            ));
        }
    }
}

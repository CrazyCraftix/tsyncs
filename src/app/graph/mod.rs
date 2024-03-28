mod activity_node;
pub mod connection;
mod mutex_node;

use std::{fs::File, io};

pub use activity_node::ActivityNode;
pub use mutex_node::MutexNode;

#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityNodeId(usize);
#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutexNodeId(usize);

const SECONDS_PER_TICK: f32 = 1.;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Graph {
    activity_nodes: std::collections::HashMap<ActivityNodeId, ActivityNode>,
    mutex_nodes: std::collections::HashMap<MutexNodeId, MutexNode>,

    connections: std::collections::HashMap<
        ActivityNodeId,
        std::collections::HashMap<MutexNodeId, connection::Connection>,
    >,

    next_activity_id: ActivityNodeId,
    next_mutex_id: MutexNodeId,

    tick_progress: f32,
}

// structure
impl Graph {
    pub fn from_csv(lines: io::Lines<io::BufReader<File>>) -> Result<Self, Box<String>> {
        let seperator = ';';
        let graph = Graph::default();
        for (line_number, line) in lines.flatten().enumerate() {
            let mut values = line.split(seperator).collect::<Vec<&str>>();

            if values.len() < 6 {
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
                    let duration = values[4].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Duration in line: {}", line_number)
                    })?;
                    let priority = values[5].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Priority in line: {}", line_number)
                    })?;
                    let mutex_connections = values[6..]
                        .iter()
                        .filter(|x| !x.is_empty())
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
                    let id = values[1]
                        .parse::<usize>()
                        .map_err(|_| format!("Error while parsing ID in line: {}", line_number))?;
                    let value = values[2].parse::<u32>().map_err(|_| {
                        format!("Error while parsing Value in line: {}", line_number)
                    })?;
                    let _activity_connections = values[3..]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .map(|x| {
                            x.parse::<u32>().map_err(|_| {
                                format!(
                                    "Error while parsing Activity Connection in line: {}",
                                    line_number
                                )
                            })
                        })
                        .collect::<Result<Vec<u32>, String>>()?;
                }
                _ => {
                    // skip line
                }
            }
        }
        return Ok(Self::default());
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
        direction: connection::Direction,
    ) -> bool {
        if !self.mutex_nodes.contains_key(&mutex_id)
            || !self.activity_nodes.contains_key(&activity_id)
        {
            return false;
        }

        let mut activity_connections = self.connections.remove(&activity_id).unwrap_or_default();

        let connection = match activity_connections.remove(&mutex_id) {
            Some(mut previous_connection) if previous_connection.direction != direction => {
                previous_connection.direction = connection::Direction::TwoWay;
                previous_connection
            }
            Some(previous_connection) => previous_connection,
            None => connection::Connection::new(direction),
        };

        activity_connections.insert(mutex_id, connection);
        self.connections.insert(activity_id, activity_connections);

        true
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
    fn tick(&mut self, ui: &egui::Ui) {
        let previous_tick_progress = self.tick_progress;
        self.tick_progress += ui.ctx().input(|i| i.stable_dt) / SECONDS_PER_TICK;
        if previous_tick_progress < 0.5 && self.tick_progress >= 0.5 {
            self.tick_a();
            self.do_per_connection(|c, a, m| c.tick(a, m));
        }
        if self.tick_progress >= 1. {
            self.tick_progress %= 1.;
            self.tick_b();
            self.do_per_connection(|c, a, m| c.tick(a, m));
        }
    }

    fn tick_a(&mut self) {
        for (activity_id, activity_node) in &mut self.activity_nodes {
            if activity_node.remaining_duration > 0 {
                continue;
            }

            if let Some(activity_connections) = self.connections.get(&activity_id) {
                // check if prerequisites are met
                let prerequisites_missing = activity_connections
                    .iter()
                    .filter(|(_, connection)| {
                        connection.direction != connection::Direction::ActivityToMutex
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
                    .for_each(|(mutex_id, connection)| {
                        if connection.direction != connection::Direction::ActivityToMutex {
                            self.mutex_nodes
                                .get_mut(mutex_id)
                                .map(|mutex_node| mutex_node.value -= 1);
                        }
                    })
            }
        }
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
                            if connection.direction != connection::Direction::MutexToActivity {
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

// drawing
impl Graph {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.tick(ui);

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
        let tick_progress = self.tick_progress;
        self.do_per_connection(|c, a, m| c.draw(ui, a, m, tick_progress));
        self.mutex_nodes.iter_mut().for_each(|n| n.1.draw(ui));
        self.activity_nodes
            .iter_mut()
            .for_each(|(_, activity_node)| activity_node.draw(ui));
    }
}

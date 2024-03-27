mod activity_node;
pub mod connection;
mod mutex_node;

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

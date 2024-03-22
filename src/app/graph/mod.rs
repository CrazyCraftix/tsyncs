mod activity_node;
mod mutex_node;

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

    connections: std::collections::HashMap<(ActivityNodeId, MutexNodeId), ConnectionType>,

    next_activity_node_id: ActivityNodeId,
    next_mutex_node_id: MutexNodeId,
}

impl Graph {
    fn resolve_connections<'a>(
        connections: &'a std::collections::HashMap<(ActivityNodeId, MutexNodeId), ConnectionType>,
        mutex_nodes: &'a std::collections::HashMap<MutexNodeId, MutexNode>,
        activity_node_id: ActivityNodeId,
    ) -> Vec<(&'a MutexNode, &'a ConnectionType)> {
        connections
            .iter()
            .filter_map(|((id, mutex_node_id), connection_type)| {
                if *id != activity_node_id {
                    return None;
                } else {
                    mutex_nodes
                        .get(mutex_node_id)
                        .map(|mutex_node| (mutex_node, connection_type))
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.activity_nodes
            .iter_mut()
            .for_each(|(&activity_node_id, activity_node)| {
                activity_node.draw(
                    ui,
                    &Self::resolve_connections(
                        &self.connections,
                        &self.mutex_nodes,
                        activity_node_id,
                    ),
                )
            });
        self.mutex_nodes.iter_mut().for_each(|n| n.1.draw(ui));
    }

    pub fn add_activiy_node(&mut self, activity_node: ActivityNode) -> ActivityNodeId {
        let id = self.next_activity_node_id;
        self.activity_nodes.insert(id, activity_node);
        self.next_activity_node_id = ActivityNodeId(id.0 + 1);
        id
    }

    pub fn add_mutex_node(&mut self, mutex_node: MutexNode) -> MutexNodeId {
        let id = self.next_mutex_node_id;
        self.mutex_nodes.insert(id, mutex_node);
        self.next_mutex_node_id = MutexNodeId(id.0 + 1);
        id
    }

    pub fn connect(
        &mut self,
        activity_node_id: ActivityNodeId,
        mutex_node_id: MutexNodeId,
        connection_type: ConnectionType,
    ) -> bool {
        if !self.mutex_nodes.contains_key(&mutex_node_id)
            || !self.activity_nodes.contains_key(&activity_node_id)
        {
            return false;
        }

        match self.connections.get_mut(&(activity_node_id, mutex_node_id)) {
            Some(existing_connection_type) => {
                if *existing_connection_type != connection_type {
                    *existing_connection_type = ConnectionType::TwoWay;
                }
            }
            None => {
                self.connections
                    .insert((activity_node_id, mutex_node_id), connection_type);
            }
        };
        true
    }
}

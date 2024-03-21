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

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ConnectedActivityNode {
    activity_node: ActivityNode,
    connections: std::collections::HashMap<MutexNodeId, ConnectionType>,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Graph {
    activity_nodes: std::collections::HashMap<ActivityNodeId, ConnectedActivityNode>,
    mutex_nodes: std::collections::HashMap<MutexNodeId, MutexNode>,
    next_activity_node_id: ActivityNodeId,
    next_mutex_node_id: MutexNodeId,
}

impl Graph {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.activity_nodes.iter_mut().for_each(|n| {
            n.1.activity_node.draw(
                ui,
                &n.1.connections
                    .iter()
                    .filter_map(|connection| {
                        if let Some(mutex_node) = self.mutex_nodes.get(connection.0) {
                            Some((mutex_node, connection.1))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        });
        self.mutex_nodes.iter_mut().for_each(|n| n.1.draw(ui));
    }

    pub fn add_activiy_node(&mut self, activity_node: ActivityNode) -> ActivityNodeId {
        let id = self.next_activity_node_id;
        self.activity_nodes.insert(
            id,
            ConnectedActivityNode {
                activity_node,
                ..Default::default()
            },
        );
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
        self.mutex_nodes.contains_key(&mutex_node_id)
            && self.activity_nodes.get_mut(&activity_node_id).map_or(
                false,
                |connected_activity_node| {
                    if let Some(existing_connection_type) =
                        connected_activity_node.connections.get_mut(&mutex_node_id)
                    {
                        if *existing_connection_type != connection_type {
                            *existing_connection_type = ConnectionType::TwoWay;
                        }
                    } else {
                        connected_activity_node
                            .connections
                            .insert(mutex_node_id, connection_type);
                    }
                    true
                },
            )
    }
}

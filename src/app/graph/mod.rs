mod activity_node;
mod mutex_node;

pub use activity_node::ActivityNode;
pub use mutex_node::MutexNode;

#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityNodeId(usize);

#[derive(Default, Hash, Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutexNodeId(usize);

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct ConnectedActivityNode {
    activity_node: ActivityNode,
    inputs: Vec<MutexNodeId>,
    outputs: Vec<MutexNodeId>,
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
                n.1.inputs
                    .iter()
                    .filter_map(|mutex_id| self.mutex_nodes.get(mutex_id))
                    .collect(),
                n.1.outputs
                    .iter()
                    .filter_map(|mutex_id| self.mutex_nodes.get(mutex_id))
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

    pub fn connect_mutex_to_activity(
        &mut self,
        mutex_node_id: MutexNodeId,
        activity_node_id: ActivityNodeId,
    ) -> bool {
        self.mutex_nodes.get(&mutex_node_id).is_some()
            && self
                .activity_nodes
                .get_mut(&activity_node_id)
                .map_or(false, |a| {
                    a.inputs.push(mutex_node_id);
                    true
                })
    }

    pub fn connect_activity_to_mutex(
        &mut self,
        activity_node_id: ActivityNodeId,
        mutex_node_id: MutexNodeId,
    ) -> bool {
        self.mutex_nodes.get(&mutex_node_id).is_some()
            && self
                .activity_nodes
                .get_mut(&activity_node_id)
                .map_or(false, |a| {
                    a.inputs.push(mutex_node_id);
                    true
                })
    }
}

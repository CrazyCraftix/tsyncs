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

enum ConnectionColor {
    Default,
    Active,
    Blocking,
}
impl Into<Vec<egui::Color32>> for ConnectionColor {
    fn into(self) -> Vec<egui::Color32> {
        match self {
            ConnectionColor::Default => vec![egui::Color32::LIGHT_GRAY, egui::Color32::DARK_GRAY],
            ConnectionColor::Active => vec![
                egui::Color32::LIGHT_GRAY,
                egui::Color32::DARK_GREEN,
                egui::Color32::LIGHT_GRAY,
                egui::Color32::GREEN,
            ],
            ConnectionColor::Blocking => vec![
                egui::Color32::LIGHT_GRAY,
                egui::Color32::DARK_RED,
                egui::Color32::LIGHT_GRAY,
                egui::Color32::RED,
            ],
        }
    }
}

const SECONDS_PER_TICK: f32 = 3.;

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

// simulation
impl Graph {
    fn tick(&mut self, ui: &egui::Ui) {
        let previous_tick_progress = self.tick_progress;
        self.tick_progress += ui.ctx().input(|i| i.stable_dt) / SECONDS_PER_TICK;
        if previous_tick_progress < 0.5 && self.tick_progress >= 0.5 {
            self.tick_a();
        } else if self.tick_progress >= 1. {
            self.tick_progress -= 1.;
            self.mutex_nodes
                .iter_mut()
                .for_each(|n| n.1.update_previous_value());
            self.tick_b();
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
                    .filter(|(_, connection_type)| {
                        connection_type != &&ConnectionType::ActivityToMutex
                    })
                    .filter_map(|(mutex_id, _)| self.mutex_nodes.get(mutex_id))
                    .find(|mutex_node| mutex_node.get_value() <= 0)
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
                                .map(|mutex_node| mutex_node.decrement_value());
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
                        .for_each(|(mutex_id, connection_type)| {
                            if connection_type != &ConnectionType::MutexToActivity {
                                self.mutex_nodes
                                    .get_mut(mutex_id)
                                    .map(|mutex_node| mutex_node.increment_value());
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
                Self::draw_connections(
                    ui,
                    activity_node,
                    &resolved_activity_connections,
                    self.tick_progress,
                )
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
        tick_progress: f32,
    ) {
        connections
            .iter()
            .for_each(|(mutex_node, connection_type)| {
                let (
                    mutex_to_activity_progress,
                    mutex_to_activity_color_1,
                    mutex_to_activity_color_2,
                ) = match (mutex_node.get_previous_value(), mutex_node.get_value()) {
                    (0, a) if a > 0 && tick_progress < 0.5 => (
                        tick_progress * 2.,
                        ConnectionColor::Active,
                        ConnectionColor::Default,
                    ),
                    (a, 0) if a > 0 && tick_progress >= 0.5 => (
                        (tick_progress - 0.5) * 2.,
                        ConnectionColor::Default,
                        ConnectionColor::Active,
                    ),
                    (_, a) if a > 0 => (0., ConnectionColor::Active, ConnectionColor::Active),
                    //_ => (0., ConnectionColor::Blocking, ConnectionColor::Blocking),
                    _ => (0., ConnectionColor::Default, ConnectionColor::Default),
                };
                let (
                    activity_to_mutex_progress,
                    activity_to_mutex_color_1,
                    activity_to_mutex_color_2,
                ) = match (
                    activity_node.remaining_duration,
                    mutex_node.get_previous_value(),
                    mutex_node.get_value(),
                ) {
                    (1, _, _) if tick_progress >= 0.5 => (
                        ((tick_progress - 0.5) * 2.),
                        ConnectionColor::Active,
                        ConnectionColor::Default,
                    ),
                    (0, a, b) if b == a + 1 && tick_progress < 0.5 => (
                        (tick_progress * 2.),
                        ConnectionColor::Default,
                        ConnectionColor::Active,
                    ),
                    _ => (1., ConnectionColor::Default, ConnectionColor::Default),
                };

                match connection_type {
                    ConnectionType::ActivityToMutex => {
                        Self::draw_connection(
                            ui,
                            activity_node.pos,
                            mutex_node.pos,
                            activity_to_mutex_color_1,
                            activity_to_mutex_color_2,
                            activity_to_mutex_progress,
                        );
                    }
                    ConnectionType::MutexToActivity => {
                        Self::draw_connection(
                            ui,
                            mutex_node.pos,
                            activity_node.pos,
                            mutex_to_activity_color_1,
                            mutex_to_activity_color_2,
                            mutex_to_activity_progress,
                        );
                    }
                    ConnectionType::TwoWay => {
                        let offset = (activity_node.pos - mutex_node.pos).normalized().rot90() * 6.;
                        Self::draw_connection(
                            ui,
                            activity_node.pos + offset,
                            mutex_node.pos + offset,
                            activity_to_mutex_color_1,
                            activity_to_mutex_color_2,
                            activity_to_mutex_progress,
                        );
                        Self::draw_connection(
                            ui,
                            mutex_node.pos - offset,
                            activity_node.pos - offset,
                            mutex_to_activity_color_1,
                            mutex_to_activity_color_2,
                            mutex_to_activity_progress,
                        );
                    }
                }
            });
    }

    fn draw_connection(
        ui: &mut egui::Ui,
        from_point: egui::Pos2,
        to_point: egui::Pos2,
        color_1: ConnectionColor,
        color_2: ConnectionColor,
        color_progress: f32,
    ) {
        let color_1: Vec<egui::Color32> = color_1.into();
        let color_2: Vec<egui::Color32> = color_2.into();

        const WIDTH: f32 = 7.;
        const ARROW_SPACING: f32 = 8.;
        const ARROW_DEPTH: f32 = 3.;
        const SCROLL_SPEED_IN_POINTS_PER_SECOND: f32 = 4.;

        ui.ctx().request_repaint();
        let time_offset = ui.input(|i| i.time) as f32 * SCROLL_SPEED_IN_POINTS_PER_SECOND
            % (ARROW_SPACING * color_1.len().max(color_2.len()) as f32);
        let color_offset = -(time_offset / ARROW_SPACING) as i32;

        let from_to_vector = to_point - from_point;
        let from_to_unit_vector = from_to_vector.normalized();
        let line_center_point =
            from_point + 0.5 * from_to_vector + (time_offset % ARROW_SPACING) * from_to_unit_vector;
        let from_to_vector_length = from_to_vector.length();
        let half_arrow_count = (from_to_vector_length / 2. / ARROW_SPACING) as i32;

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

            let progress = (arrow_tip - from_point).length() / from_to_vector_length;

            let colors = match progress {
                p if p < color_progress => &color_1,
                _ => &color_2,
            };

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

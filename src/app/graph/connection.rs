#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    MutexToActivity,
    ActivityToMutex,
    TwoWay,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum MutexToActivityState {
    Uncharged,
    Charging,
    Charged,
    Forwarding,
    Uncharging,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum ActivityToMutexState {
    Uncharged,
    Charging,
    Forwarding,
}

pub enum Color {
    Default,
    Active,
}
impl Into<Vec<egui::Color32>> for Color {
    fn into(self) -> Vec<egui::Color32> {
        match self {
            Color::Default => vec![
                egui::Color32::GRAY,
                egui::Color32::GRAY,
                egui::Color32::GRAY,
                egui::Color32::GRAY,
                egui::Color32::GRAY,
                egui::Color32::DARK_GRAY,
            ],
            Color::Active => vec![
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_GREEN,
                egui::Color32::GREEN,
                egui::Color32::GREEN,
                egui::Color32::GREEN,
                egui::Color32::GREEN,
                egui::Color32::GREEN,
                egui::Color32::GREEN,
            ],
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Connection {
    pub direction: Direction,

    activity_to_mutex_state: ActivityToMutexState,
    mutex_to_activity_state: MutexToActivityState,
}

impl Connection {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            activity_to_mutex_state: ActivityToMutexState::Uncharged,
            mutex_to_activity_state: MutexToActivityState::Uncharged,
        }
    }

    pub fn tick(&mut self, activity_node: &super::ActivityNode, mutex_node: &super::MutexNode) {
        if self.direction == Direction::MutexToActivity || self.direction == Direction::TwoWay {
            self.mutex_to_activity_state = match (
                &self.mutex_to_activity_state,
                mutex_node.value,
                activity_node.duration == activity_node.remaining_duration,
            ) {
                (MutexToActivityState::Charging, 0, true) => MutexToActivityState::Forwarding,
                (MutexToActivityState::Charged, 0, true) => MutexToActivityState::Forwarding,

                (MutexToActivityState::Charging, 0, _) => MutexToActivityState::Uncharging,
                (MutexToActivityState::Charged, 0, _) => MutexToActivityState::Uncharging,
                (_, 0, _) => MutexToActivityState::Uncharged,

                (MutexToActivityState::Charging, _, _) => MutexToActivityState::Charged,
                (MutexToActivityState::Charged, _, _) => MutexToActivityState::Charged,
                _ => MutexToActivityState::Charging,
            };
        }
        if self.direction == Direction::ActivityToMutex || self.direction == Direction::TwoWay {
            self.activity_to_mutex_state = match (
                &self.activity_to_mutex_state,
                activity_node.remaining_duration,
            ) {
                (_, 1) => ActivityToMutexState::Charging,
                (ActivityToMutexState::Uncharged, _) => ActivityToMutexState::Uncharged,
                (ActivityToMutexState::Charging, _) => ActivityToMutexState::Forwarding,
                (ActivityToMutexState::Forwarding, _) => ActivityToMutexState::Uncharged,
            }
        }
    }

    pub fn draw(
        &mut self,
        ui: &egui::Ui,
        activity_node: &super::ActivityNode,
        mutex_node: &super::MutexNode,
        tick_progress: f32,
    ) {
        let (activity_to_mutex_progress, activity_to_mutex_color_1, activity_to_mutex_color_2) =
            match self.activity_to_mutex_state {
                ActivityToMutexState::Uncharged => (0., Color::Default, Color::Default),
                ActivityToMutexState::Charging => (
                    (tick_progress - 0.5) * 2. * 2. - 1.,
                    Color::Active,
                    Color::Default,
                ),
                ActivityToMutexState::Forwarding => {
                    (tick_progress * 2. * 2. - 1., Color::Default, Color::Active)
                }
            };
        let (mutex_to_activity_progress, mutex_to_activity_color_1, mutex_to_activity_color_2) =
            match self.mutex_to_activity_state {
                MutexToActivityState::Uncharged => (0., Color::Default, Color::Default),
                MutexToActivityState::Charging => {
                    (tick_progress * 2. * 2. - 1., Color::Active, Color::Default)
                }
                MutexToActivityState::Charged => (0., Color::Active, Color::Active),
                MutexToActivityState::Forwarding => (
                    ((tick_progress - 0.5) * 2. * 2.) - 1.,
                    Color::Default,
                    Color::Active,
                ),
                MutexToActivityState::Uncharging => (
                    1. - (tick_progress - 0.5) * 2. * 2.,
                    Color::Active,
                    Color::Default,
                ),
            };

        match self.direction {
            Direction::ActivityToMutex => {
                Self::draw_arrow(
                    ui,
                    activity_node.pos,
                    mutex_node.pos,
                    activity_to_mutex_color_1,
                    activity_to_mutex_color_2,
                    activity_to_mutex_progress,
                );
            }
            Direction::MutexToActivity => {
                Self::draw_arrow(
                    ui,
                    mutex_node.pos,
                    activity_node.pos,
                    mutex_to_activity_color_1,
                    mutex_to_activity_color_2,
                    mutex_to_activity_progress,
                );
            }
            Direction::TwoWay => {
                let offset = (activity_node.pos - mutex_node.pos).normalized().rot90() * 6.;
                Self::draw_arrow(
                    ui,
                    activity_node.pos + offset,
                    mutex_node.pos + offset,
                    activity_to_mutex_color_1,
                    activity_to_mutex_color_2,
                    activity_to_mutex_progress,
                );
                Self::draw_arrow(
                    ui,
                    mutex_node.pos - offset,
                    activity_node.pos - offset,
                    mutex_to_activity_color_1,
                    mutex_to_activity_color_2,
                    mutex_to_activity_progress,
                );
            }
        }
    }

    pub fn draw_arrow(
        ui: &egui::Ui,
        from_point: egui::Pos2,
        to_point: egui::Pos2,
        color_1: Color,
        color_2: Color,
        color_progress: f32,
    ) {
        let color_1: Vec<egui::Color32> = color_1.into();
        let color_2: Vec<egui::Color32> = color_2.into();

        const WIDTH: f32 = 7.;
        const ARROW_SPACING: f32 = 3.;
        const ARROW_DEPTH: f32 = 3.;
        const SCROLL_SPEED_IN_POINTS_PER_SECOND: f32 = 4.;

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

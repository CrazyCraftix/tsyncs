#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ActivityNode {
    pos: egui::Pos2,
    task_name: String,
    activity_name: String,
    duration: String,
}

impl ActivityNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        connections: &Vec<(&super::MutexNode, &super::ConnectionType)>,
    ) {
        let style = ui.ctx().style().visuals.widgets.inactive;

        let text_field_width = 100.;

        let task_name_height = 20.;
        let activity_name_height = 18.;
        let duration_height = 15.;

        let task_name_font = egui::FontId::monospace(15.);
        let activity_name_font = egui::FontId::monospace(12.5);
        let duration_font = egui::FontId::proportional(12.5);

        let outer_padding = egui::vec2(6., 4.);
        let outer_size_without_padding =
            egui::vec2(text_field_width, task_name_height + activity_name_height);
        let outer_size = outer_size_without_padding + 2. * outer_padding;
        let outer_rect = egui::Rect::from_center_size(self.pos, outer_size);
        let outer_rounding = 10.;
        let circle_position = outer_rect.right_top() + outer_padding * egui::vec2(-1., 0.5);
        let circle_radius = outer_size.y / 3.;
        let circle_hitbox =
            egui::Rect::from_center_size(circle_position, egui::Vec2::splat(2. * circle_radius));

        // for debugging
        let frame = false;

        let mut ui = ui.child_ui(ui.max_rect(), *ui.layout());
        ui.set_enabled(!ui.ctx().input(|i| i.pointer.secondary_down()));

        self.draw_connections(&mut ui, connections);

        ui.painter()
            .rect_filled(outer_rect, outer_rounding, style.bg_fill);

        let response_outer = ui.allocate_rect(outer_rect, egui::Sense::click_and_drag());
        let response_circle = ui.allocate_rect(circle_hitbox, egui::Sense::click_and_drag());

        let response_task_name = ui.put(
            egui::Rect::from_center_size(
                self.pos
                    - egui::vec2(
                        (circle_radius + outer_padding.x / 2.) / 2.,
                        (outer_size_without_padding.y - task_name_height) / 2.,
                    ),
                egui::vec2(
                    text_field_width - circle_radius - outer_padding.x / 2.,
                    task_name_height,
                ),
            ),
            egui::TextEdit::singleline(&mut self.task_name)
                .margin(egui::Margin::ZERO)
                .frame(frame)
                .vertical_align(egui::Align::Center)
                .font(task_name_font),
        );
        let response_activity_name = ui.put(
            egui::Rect::from_center_size(
                self.pos
                    + egui::vec2(
                        0.,
                        (outer_size_without_padding.y - activity_name_height) / 2.,
                    ),
                egui::vec2(text_field_width, activity_name_height),
            ),
            egui::TextEdit::singleline(&mut self.activity_name)
                .margin(egui::Margin::ZERO)
                .frame(frame)
                .vertical_align(egui::Align::Center)
                .font(activity_name_font),
        );

        ui.painter()
            .rect_stroke(outer_rect, outer_rounding, style.fg_stroke);
        ui.painter()
            .circle_filled(circle_position, circle_radius, style.bg_fill);
        ui.painter()
            .circle_stroke(circle_position, circle_radius, style.fg_stroke);

        let response_duration = ui.put(
            egui::Rect::from_center_size(circle_position, egui::Vec2::splat(duration_height)),
            egui::TextEdit::singleline(&mut self.duration)
                .margin(egui::Margin::ZERO)
                .frame(frame)
                .vertical_align(egui::Align::Center)
                .horizontal_align(egui::Align::Center)
                .font(duration_font),
        );

        if !ui.ctx().input(|i| i.pointer.secondary_down()) {
            let response_union = response_outer
                | response_circle
                | response_task_name.clone()
                | response_activity_name.clone()
                | response_duration.clone();
            if response_union.dragged() {
                self.pos += response_union.drag_delta();
                response_task_name.surrender_focus();
                response_activity_name.surrender_focus();
                response_duration.surrender_focus();
            }
        }
    }

    fn draw_connections(
        &self,
        ui: &mut egui::Ui,
        connections: &Vec<(&super::MutexNode, &super::ConnectionType)>,
    ) {
        //let colors = vec![
        //    egui::Color32::BLUE,
        //    egui::Color32::DARK_BLUE,
        //    egui::Color32::LIGHT_BLUE,
        //];
        //let colors = vec![
        //    egui::Color32::GREEN,
        //    egui::Color32::DARK_GREEN,
        //    egui::Color32::LIGHT_GREEN,
        //];
        //let colors = vec![
        //    egui::Color32::GOLD,
        //    egui::Color32::YELLOW,
        //    egui::Color32::LIGHT_YELLOW,
        //];
        let colors = vec![egui::Color32::DARK_GRAY, egui::Color32::LIGHT_GRAY];
        //let colors = vec![egui::Color32::DARK_RED, egui::Color32::LIGHT_GRAY];
        //let colors = vec![
        //    egui::Color32::DARK_RED,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::RED,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::DARK_RED,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::DARK_BLUE,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::BLUE,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::DARK_BLUE,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::DARK_GREEN,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::GREEN,
        //    egui::Color32::LIGHT_GRAY,
        //    egui::Color32::DARK_GREEN,
        //    egui::Color32::LIGHT_GRAY,
        //];

        connections
            .iter()
            .for_each(|(mutex_node, connection_type)| match connection_type {
                super::ConnectionType::MutexToActivity => {
                    Self::draw_connection(ui, mutex_node.pos, self.pos, &colors);
                }
                super::ConnectionType::ActivityToMutex => {
                    Self::draw_connection(ui, self.pos, mutex_node.pos, &colors);
                }
                super::ConnectionType::TwoWay => {
                    let offset = (self.pos - mutex_node.pos).normalized().rot90() * 6.;
                    Self::draw_connection(ui, self.pos + offset, mutex_node.pos + offset, &colors);
                    Self::draw_connection(ui, mutex_node.pos - offset, self.pos - offset, &colors);
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

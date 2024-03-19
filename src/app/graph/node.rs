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
    pub fn draw(&mut self, ui: &mut egui::Ui) {
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
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct MutexNode {
    pos: egui::Pos2,
    count: String,
}

impl MutexNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let style = ui.ctx().style().visuals.widgets.inactive;

        let mut ui = ui.child_ui(ui.max_rect(), *ui.layout());
        ui.set_enabled(!ui.ctx().input(|i| i.pointer.secondary_down()));

        let outer_rect = egui::Rect::from_center_size(self.pos, egui::Vec2::splat(30.));

        ui.painter().rect_filled(outer_rect, 0., style.bg_fill);
        ui.painter().rect_stroke(outer_rect, 0., style.fg_stroke);
        let response_outer = ui.allocate_rect(outer_rect, egui::Sense::click_and_drag());

        let response_count = ui.put(
            outer_rect,
            egui::TextEdit::singleline(&mut self.count)
                .margin(egui::Margin::ZERO)
                .frame(false)
                .vertical_align(egui::Align::Center)
                .horizontal_align(egui::Align::Center),
        );

        let response_union = response_outer | response_count;

        if !ui.ctx().input(|i| i.pointer.secondary_down()) {
            if response_union.dragged() {
                self.pos += response_union.drag_delta();
            }
        }
    }
}

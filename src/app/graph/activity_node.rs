#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ActivityNode {
    pub pos: egui::Pos2,
    pub task_name: String,
    pub activity_name: String,
    pub duration: u32,
    pub remaining_duration: u32,

    response_outer_id: Option<egui::Id>,
    response_circle_id: Option<egui::Id>,
    response_task_name_id: Option<egui::Id>,
    response_activity_name_id: Option<egui::Id>,
    response_duration_id: Option<egui::Id>,
}

impl ActivityNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn interact(&mut self, ui: &egui::Ui) {
        if let (
            Some(Some(response_outer)),
            Some(Some(response_circle)),
            Some(Some(response_task_name)),
            Some(Some(response_activity_name)),
            Some(Some(response_duration)),
        ) = (
            self.response_outer_id
                .map(|response_outer_id| ui.ctx().read_response(response_outer_id)),
            self.response_circle_id
                .map(|response_circle_id| ui.ctx().read_response(response_circle_id)),
            self.response_task_name_id
                .map(|response_task_name_id| ui.ctx().read_response(response_task_name_id)),
            self.response_activity_name_id
                .map(|response_activity_name_id| ui.ctx().read_response(response_activity_name_id)),
            self.response_duration_id
                .map(|response_duration_id| ui.ctx().read_response(response_duration_id)),
        ) {
            if !ui.ctx().input(|i| i.pointer.secondary_down()) {
                let response_union = response_outer
                    | response_circle
                    | response_task_name.clone()
                    | response_activity_name.clone();
                if response_union.dragged() || response_union.drag_stopped() {
                    self.pos += response_union.drag_delta();
                    response_task_name.surrender_focus();
                    response_activity_name.surrender_focus();
                }
            }
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let style = ui.style().visuals.widgets.inactive;

        let outline_stoke = match self.remaining_duration {
            0 => egui::Stroke::new(2., egui::Color32::RED),
            _ => egui::Stroke::new(2., egui::Color32::GREEN),
        };

        let text_field_width = 100.;

        let task_name_height = 20.;
        let activity_name_height = 18.;
        let duration_height = 15.;

        let task_name_font = egui::FontId::monospace(15.);
        let activity_name_font = egui::FontId::monospace(12.5);

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
        //ui.set_enabled(!ui.ctx().input(|i| i.pointer.secondary_down()));

        ui.painter()
            .rect_filled(outer_rect, outer_rounding, style.bg_fill);

        let response_outer = ui.allocate_rect(outer_rect, egui::Sense::click_and_drag());
        self.response_outer_id = Some(response_outer.id);

        let response_circle = ui.allocate_rect(circle_hitbox, egui::Sense::click_and_drag());
        self.response_circle_id = Some(response_circle.id);

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
        self.response_task_name_id = Some(response_task_name.id);

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
        self.response_activity_name_id = Some(response_activity_name.id);

        ui.painter()
            .rect_stroke(outer_rect, outer_rounding, outline_stoke);
        ui.painter()
            .circle_filled(circle_position, circle_radius, style.bg_fill);
        ui.painter()
            .circle_stroke(circle_position, circle_radius, style.fg_stroke);

        let response_duration = ui.put(
            egui::Rect::from_center_size(circle_position, egui::Vec2::splat(duration_height)),
            //egui::TextEdit::singleline(&mut self.duration)
            //    .margin(egui::Margin::ZERO)
            //    .frame(frame)
            //    .vertical_align(egui::Align::Center)
            //    .horizontal_align(egui::Align::Center)
            //    .font(duration_font),
            egui::DragValue::new(&mut self.duration)
                .update_while_editing(false)
                .speed(0.05)
                .clamp_range(1..=std::u32::MAX),
        );
        let response_remaining_duration = ui.put(
            egui::Rect::from_center_size(circle_position+egui::vec2(0., 1.5*duration_height), egui::Vec2::splat(duration_height)),
            egui::DragValue::new(&mut self.remaining_duration)
                .update_while_editing(false)
                .speed(0.05)
        );
        self.response_duration_id = Some(response_duration.id);
    }
}
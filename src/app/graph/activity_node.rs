#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ActivityNode {
    pub pos: egui::Pos2,
    pub task_name: String,
    pub activity_name: String,
    pub priority: u32,
    pub duration: u32,
    pub remaining_duration: u32,

    #[serde(skip)]
    response_outer_id: Option<egui::Id>,
    #[serde(skip)]
    response_circle_id: Option<egui::Id>,
    #[serde(skip)]
    response_task_name_id: Option<egui::Id>,
    #[serde(skip)]
    response_activity_name_id: Option<egui::Id>,
}

impl ActivityNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn interact(&mut self, ui: &egui::Ui) -> Option<egui::Response> {
        if let (
            Some(Some(response_outer)),
            Some(Some(response_circle)),
            Some(Some(response_task_name)),
            Some(Some(response_activity_name)),
        ) = (
            self.response_outer_id
                .map(|response_outer_id| ui.ctx().read_response(response_outer_id)),
            self.response_circle_id
                .map(|response_circle_id| ui.ctx().read_response(response_circle_id)),
            self.response_task_name_id
                .map(|response_task_name_id| ui.ctx().read_response(response_task_name_id)),
            self.response_activity_name_id
                .map(|response_activity_name_id| ui.ctx().read_response(response_activity_name_id)),
        ) {
            let response_union = response_outer
                | response_circle
                | response_task_name.clone()
                | response_activity_name.clone();
            if !ui.ctx().input(|i| i.pointer.secondary_down()) {
                if response_union.dragged() || response_union.drag_stopped() {
                    self.pos += response_union.drag_delta();
                    response_task_name.surrender_focus();
                    response_activity_name.surrender_focus();
                }
            }
            Some(response_union)
        } else {
            None
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        container_transform: egui::emath::TSTransform,
        tick_progress: f32,
    ) {
        const MAX_THREE_DIGIT_NUMBER: u32 = 999;
        let style = ui.style().visuals.widgets.inactive;

        let outline_stoke = match self.remaining_duration {
            0 => egui::Stroke::new(2., egui::Color32::RED),
            _ => egui::Stroke::new(2.5, egui::Color32::GREEN),
        };

        let text_field_width = 100.;

        let task_name_height = 20.;
        let activity_name_height = 18.;
        let textinput_height = 15.;

        let task_name_font = egui::FontId::proportional(18.);
        let activity_name_font = egui::FontId::proportional(15.5);

        let outer_padding = egui::vec2(6., 4.);
        let outer_size_without_padding =
            egui::vec2(text_field_width, task_name_height + activity_name_height);
        let outer_size = outer_size_without_padding + 2. * outer_padding;
        let outer_rect = egui::Rect::from_center_size(self.pos, outer_size);
        let outer_rounding = 10.;
        let priority_rect = egui::Rect::from_two_pos(
            outer_rect.right_bottom(),
            outer_rect.right_bottom() - egui::vec2(textinput_height * 2., textinput_height * 1.29),
        );

        let circle_position = egui::pos2(priority_rect.center_top().x, outer_rect.right_top().y);
        let circle_radius = outer_size.y / 2.5;
        let circle_hitbox =
            egui::Rect::from_center_size(circle_position, egui::Vec2::splat(2. * circle_radius));

        let priority_rounding = egui::Rounding {
            nw: outer_rounding,
            ne: 0.,
            sw: 0.,
            se: outer_rounding,
        };

        // for debugging
        let frame = false;
        ui.child_ui(ui.max_rect(), *ui.layout());

        ui.painter()
            .rect_filled(outer_rect, outer_rounding, style.bg_fill);

        if self.remaining_duration > 0 {
            let mut progress_rect = outer_rect;
            progress_rect.set_width(
                (1. - (self.remaining_duration as f32 - tick_progress)
                    / (self.duration as f32 - 0.5))
                    * outer_rect.width(),
            );
            progress_rect.set_left(outer_rect.left());
            ui.painter().rect_filled(
                progress_rect,
                outer_rounding,
                egui::Color32::from_rgba_unmultiplied(0, 255, 0, 5),
            );
        }

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

        ui.painter().rect_filled(
            priority_rect,
            priority_rounding,
        style.bg_fill  );
        ui.painter().rect_stroke(
            priority_rect,
            priority_rounding,
            style.fg_stroke,
        );
        ui.painter()
            .rect_stroke(outer_rect, outer_rounding, outline_stoke);
        ui.painter()
            .circle_filled(circle_position, circle_radius, style.bg_fill);
        ui.painter()
            .circle_stroke(circle_position, circle_radius, style.fg_stroke);

        // Priority
        let speed = container_transform.scaling * 0.05;
        ui.put(
            egui::Rect::from_center_size(
                priority_rect.center(),
                egui::Vec2::splat(textinput_height),
            ),
            egui::DragValue::new(&mut self.priority)
                .update_while_editing(false)
                .speed(speed)
                .clamp_range(0..=MAX_THREE_DIGIT_NUMBER),
        );

        // Line between remaining duration and duration
        ui.painter().line_segment(
            [
                circle_position + egui::vec2(-circle_radius * 0.7, 0.),
                circle_position + egui::vec2(circle_radius * 0.7, 0.),
            ],
            egui::Stroke::new(0.5, egui::Color32::GRAY),
        );

        // Duration
        ui.put(
            egui::Rect::from_center_size(
                circle_position + egui::vec2(0., 0.53 * textinput_height),
                egui::Vec2::splat(textinput_height),
            ),
            egui::DragValue::new(&mut self.duration)
                .update_while_editing(false)
                .speed(speed)
                .clamp_range(1..=MAX_THREE_DIGIT_NUMBER),
        );

        // Remaining Duration
        ui.put(
            egui::Rect::from_center_size(
                circle_position + egui::vec2(0., -0.53 * textinput_height),
                egui::Vec2::splat(textinput_height),
            ),
            egui::DragValue::new(&mut self.remaining_duration)
                .update_while_editing(false)
                .speed(speed)
                .clamp_range(0..=MAX_THREE_DIGIT_NUMBER),
        );
    }
}

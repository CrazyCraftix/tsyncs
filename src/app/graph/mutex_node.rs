#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct MutexNode {
    pub pos: egui::Pos2,
    pub value: u32,

    previous_value: u32,
    response_outer_id: Option<egui::Id>,
    response_value_id: Option<egui::Id>,
}

impl MutexNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn interact(&mut self, ui: &egui::Ui) {
        if let (Some(Some(response_outer)), Some(Some(response_value))) = (
            self.response_outer_id
                .map(|response_outer_id| ui.ctx().read_response(response_outer_id)),
            self.response_value_id
                .map(|response_value_id| ui.ctx().read_response(response_value_id)),
        ) {
            if !ui.ctx().input(|i| i.pointer.secondary_down()) {
                if response_outer.dragged() || response_outer.drag_stopped() {
                    self.pos += response_outer.drag_delta();
                    response_value.surrender_focus();
                }
            }
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let style = ui.ctx().style().visuals.widgets.inactive;

        let mut ui = ui.child_ui(ui.max_rect(), *ui.layout());
        //ui.set_enabled(!ui.ctx().input(|i| i.pointer.secondary_down()));

        let outer_rect = egui::Rect::from_center_size(self.pos, egui::Vec2::splat(30.));

        ui.painter().rect_filled(outer_rect, 0., style.bg_fill);
        ui.painter().rect_stroke(outer_rect, 0., style.fg_stroke);
        let response_outer = ui.allocate_rect(outer_rect, egui::Sense::click_and_drag());
        self.response_outer_id = Some(response_outer.id);

        let response_value = ui.put(
            egui::Rect::from_center_size(self.pos, egui::Vec2::splat(15.)),
            egui::DragValue::new(&mut self.value)
                .update_while_editing(false)
                .speed(0.05),
        );
        self.response_value_id = Some(response_value.id);
    }
}

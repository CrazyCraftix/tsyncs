#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct MutexNode {
    pub pos: egui::Pos2,
    pub count: String,
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

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct MutexNode {
    pub pos: egui::Pos2,
    pub value: u32,

    #[serde(skip)]
    response_outer_id: Option<egui::Id>,
    #[serde(skip)]
    response_value_id: Option<egui::Id>,
}

impl Clone for MutexNode {
    fn clone(&self) -> Self {
        Self {
            pos: self.pos,
            value: self.value,
            response_outer_id: None,
            response_value_id: None,
        }
    }
}

impl MutexNode {
    pub fn new(pos: egui::Pos2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    pub fn interact(&mut self, ui: &egui::Ui) -> Option<egui::Response> {
        if let (Some(Some(response_outer)), Some(Some(response_value))) = (
            self.response_outer_id
                .map(|response_outer_id| ui.ctx().read_response(response_outer_id)),
            self.response_value_id
                .map(|response_value_id| ui.ctx().read_response(response_value_id)),
        ) {
            if !ui.ctx().input(|i| i.pointer.secondary_down())
                && (response_outer.dragged() || response_outer.drag_stopped())
            {
                self.pos += response_outer.drag_delta();
                response_value.surrender_focus();
            }

            Some(response_outer | response_value)
        } else {
            None
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, container_transform: egui::emath::TSTransform) {
        let style = ui.ctx().style().visuals.widgets.inactive;

        let mut ui = ui.child_ui(ui.max_rect(), *ui.layout());
        //ui.set_enabled(!ui.ctx().input(|i| i.pointer.secondary_down()));

        let outer_rect = egui::Rect::from_center_size(self.pos, egui::Vec2::splat(30.));

        let mut stroke = style.fg_stroke;
        if self.value != 0 {
            stroke.color = egui::Color32::GREEN;
            stroke.width = 1.5;
        }
        ui.painter().rect_filled(outer_rect, 0., style.bg_fill);
        ui.painter().rect_stroke(outer_rect, 0., stroke);
        let response_outer = ui.allocate_rect(outer_rect, egui::Sense::click_and_drag());
        self.response_outer_id = Some(response_outer.id);

        let response_value = ui.put(
            egui::Rect::from_center_size(self.pos, egui::Vec2::splat(15.)),
            egui::DragValue::new(&mut self.value)
                .update_while_editing(false)
                .speed(container_transform.scaling * 0.05),
        );
        self.response_value_id = Some(response_value.id);
    }
}

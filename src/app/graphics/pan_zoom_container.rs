use egui::{emath::TSTransform, Id, LayerId, Pos2, Vec2};

#[derive(Clone, PartialEq)]
#[must_use = "You should call .show()"]
pub struct PanZoomContainer {
    id_source: Id,
    min_size: Vec2,
}

#[allow(dead_code)]
impl PanZoomContainer {
    /// create a new [`PanZoomContainer`]
    pub fn new() -> Self {
        Self {
            id_source: Id::NULL,
            min_size: Vec2::INFINITY,
        }
    }

    /// specify an id source, default is `Id::NULL`
    /// the final id will become `ui.id().with(id_source)`
    /// ids must be unique, see [`Id`]
    pub fn id_source(mut self, id_source: impl Into<Id>) -> Self {
        self.id_source = id_source.into();
        self
    }

    /// specify the minimum size
    /// this will be capped at `ui.available_size_before_wrap()`
    /// default is `Vec2::INFINITY`, so effectively the cap
    /// the actual size may be bigger e.g. in a justified layout, see [`egui::Ui::allocate_space()`]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
}

impl PanZoomContainer {
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui, TSTransform, &egui::Response) -> R,
    ) -> egui::InnerResponse<R> {
        let id = ui.id().with(self.id_source);

        // allocate space and check for interactions
        let available_size = ui.available_size_before_wrap();
        let (_, rect) = ui.allocate_space(available_size.min(self.min_size));
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        // update zomm and pan
        let mut state = PanZoomContainerState::load(ui.ctx(), id);
        state.handle_zoom_pan(&response);
        state.store(ui.ctx(), id);

        // draw on a transformed layer inside a child ui, decoupled from the surrounding ui
        // this seems to be the cleanest way to get this to work
        let mut ui = ui.child_ui(ui.max_rect(), *ui.layout());
        let inner_response = ui
            .with_layer_id(LayerId::new(egui::Order::Middle, id), |ui| {
                ui.set_clip_rect(state.transform.inverse() * rect);
                ui.ctx().set_transform_layer(ui.layer_id(), state.transform);
                add_contents(ui, state.transform, &response)
            })
            .inner;

        egui::InnerResponse {
            inner: inner_response,
            response,
        }
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
struct PanZoomContainerState {
    transform: TSTransform,
    last_center: Pos2,
}

impl PanZoomContainerState {
    fn load(context: &egui::Context, id: Id) -> Self {
        context
            .data_mut(|data| data.get_persisted(id))
            .unwrap_or_default()
    }

    fn store(&self, context: &egui::Context, id: Id) {
        context.data_mut(|data| {
            data.insert_persisted(id, self.clone());
        });
    }

    fn handle_zoom_pan(&mut self, response: &egui::Response) {
        let mouse_position = response.ctx.input(|i| i.pointer.latest_pos());
        if mouse_position.map_or(true, |pos| response.interact_rect.contains(pos)) {
            // zoom
            let zoom_delta = response.ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1. {
                let screen_space_zoom_anchor_position = response
                    .ctx
                    .input(|i| i.pointer.latest_pos())
                    .unwrap_or_default();

                let transformed_space_zoom_anchor_position =
                    self.transform.inverse() * screen_space_zoom_anchor_position;
                self.transform.scaling *= zoom_delta;
                let new_screen_space_zoom_anchor_position =
                    self.transform * transformed_space_zoom_anchor_position;

                self.transform.translation +=
                    screen_space_zoom_anchor_position - new_screen_space_zoom_anchor_position;
            }

            // scroll
            let scroll_delta = response.ctx.input(|i| i.smooth_scroll_delta);
            self.transform.translation += scroll_delta;
        }

        // pan
        if !response.ctx.input(|i| i.pointer.secondary_down()) {
            self.transform.translation += response.drag_delta();
        }

        // reset
        if response.double_clicked() {
            self.transform = Default::default();
            self.last_center = Default::default();
        }

        // anchor the content in the center
        let center = response.rect.center();
        self.transform.translation += center - self.last_center;
        self.last_center = center;
    }
}

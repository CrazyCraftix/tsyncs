mod graph;
mod graphics;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    graph: Vec<graph::Node>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            graph: vec![
                graph::Node::new(egui::pos2(0., 0.)),
                graph::Node::new(egui::pos2(100., 100.)),
            ],
        }
    }
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        //creation_context.egui_ctx.set_zoom_factor(2.);

        // load previous app state, if it exists
        if let Some(storage) = creation_context.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        // load default app state
        Default::default()
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // save app state
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // menu bar
        egui::TopBottomPanel::top("top_panel")
            .min_height(0.)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(0.)
            .show(ctx, |ui| {
                egui::warn_if_debug_build(ui);
            });

        // main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                graphics::PanZoomContainer::new()
                    .id_source("abc")
                    .show(ui, |ui| {
                        for node in &mut self.graph {
                            node.draw(ui);
                        }
                    });
            });
        });
    }
}

mod graph;
mod graphics;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    graph: graph::Graph,
}

impl Default for App {
    fn default() -> Self {
        let mut graph = graph::Graph::default();
        let activity_1 = graph.add_activiy_node(graph::ActivityNode::new(egui::pos2(0., 0.)));
        let activity_2 = graph.add_activiy_node(graph::ActivityNode::new(egui::pos2(100., 100.)));
        let mutex_1 = graph.add_mutex_node(graph::MutexNode::new(egui::pos2(50., 50.)));
        graph.connect(activity_1, mutex_1, graph::ConnectionType::TwoWay);
        graph.connect(activity_2, mutex_1, graph::ConnectionType::MutexToActivity);
        Self { graph }
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
                graphics::PanZoomContainer::new().show(ui, |ui| {
                    self.graph.draw(ui);
                });
            });
        });
    }
}

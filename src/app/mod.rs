use std::{
    fmt::format,
    io::{BufRead as _, Write as _},
};

use egui::Label;

use self::graph::Graph;

mod graph;
mod graphics;

//use native_dialog::{FileDialog, MessageDialog, MessageType};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    graph: Graph,
}

impl Default for App {
    fn default() -> Self {
        let mut a2 = graph::ActivityNode::new(egui::pos2(300., 100.));
        a2.task_name = "Task 2".into();
        a2.activity_name = "Activity 2".into();
        a2.duration = 3;

        let mut a1 = graph::ActivityNode::new(egui::pos2(150., 250.));
        a1.task_name = "Task 1".into();
        a1.activity_name = "Activity 1".into();
        a1.duration = 3;

        let mut a5b = graph::ActivityNode::new(egui::pos2(150., 400.));
        a5b.task_name = "Task 5".into();
        a5b.activity_name = "Activity 5b".into();
        a5b.duration = 1;

        let mut a5a = graph::ActivityNode::new(egui::pos2(450., 400.));
        a5a.task_name = "Task 5".into();
        a5a.activity_name = "Activity 5a".into();
        a5a.duration = 2;

        let mut a3 = graph::ActivityNode::new(egui::pos2(450., 250.));
        a3.task_name = "Task 3".into();
        a3.activity_name = "Activity 3".into();
        a3.duration = 2;

        let mut a4 = graph::ActivityNode::new(egui::pos2(600., 100.));
        a4.task_name = "Task 4".into();
        a4.activity_name = "Activity 4".into();
        a4.duration = 3;

        let mut a6 = graph::ActivityNode::new(egui::pos2(750., 250.));
        a6.task_name = "Task 6".into();
        a6.activity_name = "Activity 6".into();
        a6.duration = 3;

        let m24 = graph::MutexNode::new((a2.pos + a4.pos.to_vec2()) / 2.);
        let m12 = graph::MutexNode::new((a1.pos + a2.pos.to_vec2()) / 2.);
        let mut m234 = graph::MutexNode::new((a2.pos + a3.pos.to_vec2() + a4.pos.to_vec2()) / 3.);
        m234.value = 1;
        let m46 = graph::MutexNode::new((a4.pos + a6.pos.to_vec2()) / 2.);
        let m13 = graph::MutexNode::new((a1.pos + a3.pos.to_vec2()) / 2.);
        let m36 = graph::MutexNode::new((a3.pos + a6.pos.to_vec2()) / 2.);
        let m65a = graph::MutexNode::new((a6.pos + a5a.pos.to_vec2()) / 2.);
        let mut m5b1 = graph::MutexNode::new((a5b.pos + a1.pos.to_vec2()) / 2.);
        m5b1.value = 1;
        let mut m5b5a =
            graph::MutexNode::new((a5b.pos + a5a.pos.to_vec2()) / 2. + egui::vec2(0., 20.));
        m5b5a.value = 1;
        let m5a5b = graph::MutexNode::new((a5b.pos + a5a.pos.to_vec2()) / 2. - egui::vec2(0., 20.));

        let mut graph = Graph::default();
        let a2 = graph.add_activity_node(a2);
        let a1 = graph.add_activity_node(a1);
        let a5b = graph.add_activity_node(a5b);
        let a5a = graph.add_activity_node(a5a);
        let a3 = graph.add_activity_node(a3);
        let a4 = graph.add_activity_node(a4);
        let a6 = graph.add_activity_node(a6);

        let m24 = graph.add_mutex_node(m24);
        let m12 = graph.add_mutex_node(m12);
        let m234 = graph.add_mutex_node(m234);
        let m46 = graph.add_mutex_node(m46);
        let m13 = graph.add_mutex_node(m13);
        let m36 = graph.add_mutex_node(m36);
        let m65a = graph.add_mutex_node(m65a);
        let m5b1 = graph.add_mutex_node(m5b1);
        let m5b5a = graph.add_mutex_node(m5b5a);
        let m5a5b = graph.add_mutex_node(m5a5b);

        graph.connect(a2, m24, graph::connection::Direction::ActivityToMutex);
        graph.connect(a4, m24, graph::connection::Direction::MutexToActivity);

        graph.connect(a1, m12, graph::connection::Direction::ActivityToMutex);
        graph.connect(a2, m12, graph::connection::Direction::MutexToActivity);

        graph.connect(a2, m234, graph::connection::Direction::TwoWay);
        graph.connect(a3, m234, graph::connection::Direction::TwoWay);
        graph.connect(a4, m234, graph::connection::Direction::TwoWay);

        graph.connect(a4, m46, graph::connection::Direction::ActivityToMutex);
        graph.connect(a6, m46, graph::connection::Direction::MutexToActivity);

        graph.connect(a1, m13, graph::connection::Direction::ActivityToMutex);
        graph.connect(a3, m13, graph::connection::Direction::MutexToActivity);

        graph.connect(a3, m36, graph::connection::Direction::ActivityToMutex);
        graph.connect(a6, m36, graph::connection::Direction::MutexToActivity);

        graph.connect(a6, m65a, graph::connection::Direction::ActivityToMutex);
        graph.connect(a5a, m65a, graph::connection::Direction::MutexToActivity);

        graph.connect(a5b, m5b1, graph::connection::Direction::ActivityToMutex);
        graph.connect(a1, m5b1, graph::connection::Direction::MutexToActivity);

        graph.connect(a5b, m5b5a, graph::connection::Direction::ActivityToMutex);
        graph.connect(a5a, m5b5a, graph::connection::Direction::MutexToActivity);

        graph.connect(a5a, m5a5b, graph::connection::Direction::ActivityToMutex);
        graph.connect(a5b, m5a5b, graph::connection::Direction::MutexToActivity);

        Self { graph }
    }
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        creation_context.egui_ctx.set_visuals(egui::Visuals::dark());

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
                    egui::menu::menu_button(ui, "File", |ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Open Graph...").clicked() {
                            let path_result = native_dialog::FileDialog::new()
                                .set_location(&dirs::home_dir().unwrap())
                                .add_filter("Comma Separated Values", &["csv"])
                                .add_filter("All files", &["*"])
                                .show_open_single_file();

                            match path_result {
                                Ok(Some(path_buffer)) => {
                                    let filename = path_buffer.to_str().unwrap();
                                    let lines = std::io::BufReader::new(
                                        std::fs::File::open(filename).unwrap(),
                                    )
                                    .lines();

                                    match Graph::from_csv(lines) {
                                        Ok(graph) => {
                                            self.graph = graph;
                                        }
                                        Err(e) => {
                                            native_dialog::MessageDialog::new()
                                                .set_type(native_dialog::MessageType::Error)
                                                .set_title("Parser Error")
                                                .set_text(&format!("{}", e))
                                                .show_alert()
                                                .unwrap();
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    native_dialog::MessageDialog::new()
                                        .set_type(native_dialog::MessageType::Error)
                                        .set_title("Error")
                                        .set_text(&format!("Error: {}", e))
                                        .show_alert()
                                        .unwrap();
                                }
                            }
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Save Graph As...").clicked() {
                            let path_result = native_dialog::FileDialog::new()
                                .set_location(&dirs::home_dir().unwrap())
                                .add_filter("Comma Separated Values", &["csv"])
                                .add_filter("All files", &["*"])
                                .show_save_single_file();

                            match path_result {
                                Ok(Some(path_buffer)) => {
                                    let filename = path_buffer.to_str().unwrap();
                                    let csv = self.graph.to_csv();
                                    match std::fs::File::create(filename) {
                                        Ok(mut file) => {
                                            file.write(csv.as_bytes()).unwrap();
                                        }
                                        Err(e) => {
                                            native_dialog::MessageDialog::new()
                                                .set_type(native_dialog::MessageType::Error)
                                                .set_title("Error")
                                                .set_text(&format!("{}", e))
                                                .show_alert()
                                                .unwrap();
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    native_dialog::MessageDialog::new()
                                        .set_type(native_dialog::MessageType::Error)
                                        .set_title("Error")
                                        .set_text(&format!("Error: {}", e))
                                        .show_alert()
                                        .unwrap();
                                }
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        if ui.button("Download Graph").clicked() {
                            // download file
                        }

                        #[cfg(target_arch = "wasm32")]
                        if ui.button("Upload Graph").clicked() {
                            // upload file
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(25.)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    egui::warn_if_debug_build(ui);
                    ui.style_mut().spacing.slider_width = 175.;
                    ui.add(
                        egui::widgets::Slider::new(&mut self.graph.ticks_per_second, 0.1..=50.0)
                            .text("ticks per second")
                            .logarithmic(true)
                            .max_decimals(2),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(format!(
                                    "{}",
                                    match self.graph.is_running() {
                                        true => "⏸",
                                        false => "▶",
                                    }
                                ))
                                .min_size(egui::vec2(25., 0.)),
                            )
                            .clicked()
                        {
                            self.graph.toggle_play_pause();
                        };
                        if !self.graph.is_running() {
                            let range = match self.graph.remaining_ticks_to_run {
                                0 => 0..=1000,
                                _ => 1..=1000,
                            };
                            if ui.button("Single Step").clicked() {
                                self.graph.queue_tick();
                            }
                            ui.separator();
                            ui.label("ticks remaining");
                            ui.add(
                                egui::DragValue::new(&mut self.graph.remaining_ticks_to_run)
                                    .update_while_editing(false)
                                    .speed(0.1)
                                    .clamp_range(range)
                                    .max_decimals(0),
                            );
                        }
                    });
                });
            });

        // main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let mut transform = Default::default();
                graphics::PanZoomContainer::new()
                    .show(ui, |ui, container_transform, container_response| {
                        transform = container_transform;
                        self.graph.tick(ui);
                        self.graph.interact(ui, container_transform, container_response);
                        self.graph.draw(ui);
                    });
            });
        });
    }
}

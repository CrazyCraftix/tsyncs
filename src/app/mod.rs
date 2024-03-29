use std::{
    fmt::format,
    io::{BufRead as _, Write as _},
};

use std::sync::mpsc::{channel, Receiver, Sender};

use self::graph::Graph;
use std::future;

mod graph;
mod graphics;

//use native_dialog::{FileDialog, MessageDialog, MessageType};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    graph: Graph,

    #[serde(skip)]
    text_channel: (Sender<String>, Receiver<String>),
    #[serde(skip)]
    file_buffer: String,
}

impl Default for App {
    fn default() -> Self {
        let mut a2 = graph::ActivityNode::new(egui::pos2(300., 100.));
        a2.task_name = "Task 2".into();
        a2.activity_name = "Activiy 2".into();
        a2.duration = 3;

        let mut a1 = graph::ActivityNode::new(egui::pos2(150., 250.));
        a1.task_name = "Task 1".into();
        a1.activity_name = "Activiy 1".into();
        a1.duration = 3;

        let mut a5b = graph::ActivityNode::new(egui::pos2(150., 400.));
        a5b.task_name = "Task 5".into();
        a5b.activity_name = "Activiy 5b".into();
        a5b.duration = 1;

        let mut a5a = graph::ActivityNode::new(egui::pos2(450., 400.));
        a5a.task_name = "Task 5".into();
        a5a.activity_name = "Activiy 5a".into();
        a5a.duration = 2;

        let mut a3 = graph::ActivityNode::new(egui::pos2(450., 250.));
        a3.task_name = "Task 3".into();
        a3.activity_name = "Activiy 3".into();
        a3.duration = 2;

        let mut a4 = graph::ActivityNode::new(egui::pos2(600., 100.));
        a4.task_name = "Task 4".into();
        a4.activity_name = "Activiy 4".into();
        a4.duration = 3;

        let mut a6 = graph::ActivityNode::new(egui::pos2(750., 250.));
        a6.task_name = "Task 6".into();
        a6.activity_name = "Activiy 6".into();
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
        let a2 = graph.add_activiy_node(a2);
        let a1 = graph.add_activiy_node(a1);
        let a5b = graph.add_activiy_node(a5b);
        let a5a = graph.add_activiy_node(a5a);
        let a3 = graph.add_activiy_node(a3);
        let a4 = graph.add_activiy_node(a4);
        let a6 = graph.add_activiy_node(a6);

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

        Self {
            graph,
            text_channel: channel(),
            file_buffer: Default::default(),
        }
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
        if let Ok(text) = self.text_channel.1.try_recv() {
            self.file_buffer = text;
        }

        if !self.file_buffer.is_empty() {
            match Graph::from_csv(&self.file_buffer) {
                Ok(graph) => {
                    self.graph = graph;
                }
                Err(e) => {
                    rfd::MessageDialog::new()
                        .set_title("Error")
                        .set_description(&format!("Failed to import graph: {}", e))
                        .set_level(rfd::MessageLevel::Error)
                        .show();
                }
            }
            self.file_buffer.clear();
        }

        egui::TopBottomPanel::top("top_panel")
            .min_height(0.)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::menu::menu_button(ui, "File", |ui| {
                        if ui.button("⬅ Import Graph").clicked() {
                            let sender = self.text_channel.0.clone();
                            let task = rfd::AsyncFileDialog::new().add_filter("Comma Seperated Values", &["csv"]).add_filter("All Files", &["*"]).pick_file();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    let text = file.read().await;
                                    let _ = sender.send(String::from_utf8_lossy(&text).to_string());
                                }
                            });
                        }

                        if ui.button("➡ Export Graph").clicked() {
                            let task = rfd::AsyncFileDialog::new().add_filter("Comma Seperated Values", &["csv"]).add_filter("All Files", &["*"]).save_file();
                            let contents = self.graph.to_csv();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    _ = file.write(contents.as_bytes()).await;
                                }
                            });
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
                graphics::PanZoomContainer::new().show(ui, |ui| {
                    self.graph.draw(ui);
                });
            });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: future::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

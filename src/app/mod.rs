use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

mod graph;
mod graphics;

//use native_dialog::{FileDialog, MessageDialog, MessageType};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    graph: graph::Graph,
}

impl Default for App {
    fn default() -> Self {
        let mut a2 = graph::ActivityNode::new(egui::pos2(300., 100.));
        a2.task_name = "Task 2".into();
        a2.activity_name = "Activiy 2".into();

        let mut a1 = graph::ActivityNode::new(egui::pos2(150., 250.));
        a1.task_name = "Task 1".into();
        a1.activity_name = "Activiy 1".into();

        let mut a5b = graph::ActivityNode::new(egui::pos2(150., 400.));
        a5b.task_name = "Task 5".into();
        a5b.activity_name = "Activiy 5b".into();

        let mut a5a = graph::ActivityNode::new(egui::pos2(450., 400.));
        a5a.task_name = "Task 5".into();
        a5a.activity_name = "Activiy 5a".into();

        let mut a3 = graph::ActivityNode::new(egui::pos2(450., 250.));
        a3.task_name = "Task 3".into();
        a3.activity_name = "Activiy 3".into();

        let mut a4 = graph::ActivityNode::new(egui::pos2(600., 100.));
        a4.task_name = "Task 4".into();
        a4.activity_name = "Activiy 4".into();

        let mut a6 = graph::ActivityNode::new(egui::pos2(750., 250.));
        a6.task_name = "Task 6".into();
        a6.activity_name = "Activiy 6".into();

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

        let mut graph = graph::Graph::default();
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

        graph.connect(a2, m24, graph::ConnectionType::ActivityToMutex);
        graph.connect(a4, m24, graph::ConnectionType::MutexToActivity);

        graph.connect(a1, m12, graph::ConnectionType::ActivityToMutex);
        graph.connect(a2, m12, graph::ConnectionType::MutexToActivity);

        graph.connect(a2, m234, graph::ConnectionType::TwoWay);
        graph.connect(a3, m234, graph::ConnectionType::TwoWay);
        graph.connect(a4, m234, graph::ConnectionType::TwoWay);

        graph.connect(a4, m46, graph::ConnectionType::ActivityToMutex);
        graph.connect(a6, m46, graph::ConnectionType::MutexToActivity);

        graph.connect(a1, m13, graph::ConnectionType::ActivityToMutex);
        graph.connect(a3, m13, graph::ConnectionType::MutexToActivity);

        graph.connect(a3, m36, graph::ConnectionType::ActivityToMutex);
        graph.connect(a6, m36, graph::ConnectionType::MutexToActivity);

        graph.connect(a6, m65a, graph::ConnectionType::ActivityToMutex);
        graph.connect(a5a, m65a, graph::ConnectionType::MutexToActivity);

        graph.connect(a5b, m5b1, graph::ConnectionType::ActivityToMutex);
        graph.connect(a1, m5b1, graph::ConnectionType::MutexToActivity);

        graph.connect(a5b, m5b5a, graph::ConnectionType::ActivityToMutex);
        graph.connect(a5a, m5b5a, graph::ConnectionType::MutexToActivity);

        graph.connect(a5a, m5a5b, graph::ConnectionType::ActivityToMutex);
        graph.connect(a5b, m5a5b, graph::ConnectionType::MutexToActivity);

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
                    egui::menu::menu_button(ui, "File", |ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Open File...").clicked() {
                            let path_result = native_dialog::FileDialog::new()
                                .set_location("~/Desktop")
                                .add_filter("Comma Seperated Values", &["csv"])
                                .add_filter("All files", &["*"])
                                .show_open_single_file();

                            match path_result {
                                Ok(Some(pathBuffer)) => {
                                    let filename = pathBuffer.to_str().unwrap();
                                    let lines = read_lines(filename).unwrap();
                                    _ = parse_csv(lines);
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
                                .set_location("~/Desktop")
                                .add_filter("Comma Seperated Values", &["csv"])
                                .add_filter("All files", &["*"])
                                .show_save_single_file();

                            match path_result {
                                Ok(Some(pathBuffer)) => {
                                    let filename = pathBuffer.to_str().unwrap();
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

fn parse_csv(lines : io::Lines<io::BufReader<File>>) -> Result<graph::Graph, Box<String>> {
    let seperator = ',';
    for (line_number, line) in lines.flatten().enumerate() {
        let mut values = line.split(seperator).collect::<Vec<&str>>();

        if values.len() < 5 {
            continue;
        }

        // match first value to determine type of line
        match values[0].to_lowercase().as_str() {
            "task" => {
                let id = values[1].trim().parse::<i32>().map_err(|_| format!("Error while parsing ID in line: {}", line_number))?;
                let task_name = values[2].to_string();
                let activity_name = values[3].to_string();
                let duration = values[3].parse::<i32>().map_err(|_| format!("Error while parsing Duration in line: {}", line_number))?;
                let priority = values[4].parse::<i32>().map_err(|_| format!("Error while parsing Priority in line: {}", line_number))?;
                let mutex_connections = values[5..].iter().map(|x| x.parse::<i32>().map_err(|_| format!("Error while parsing Mutex Connection in line: {}", line_number))).collect::<Result<Vec<i32>, String>>()?;
            }
            "mutex" => {
                let id = values[1].parse::<i32>().expect("Error while parsing ID");
                let value = values[2].parse::<i32>().expect("Error while parsing Value");
            }
            _ => {
                // skip line
            }
        }
    }
    Ok(graph::Graph::default())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

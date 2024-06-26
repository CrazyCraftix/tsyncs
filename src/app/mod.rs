use std::sync::mpsc::{channel, Receiver, Sender};

use egui::{Layout, Pos2};

use self::graph::Graph;
use std::future;

mod graph;
mod graphics;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    active_graph: Graph,
    stored_graphs: Vec<Graph>,
    scaling_in_percent: f32,

    show_about_dialog: bool,
    show_simulation_controls: bool,
    pin_menu_bar: bool,

    #[serde(skip)]
    text_channel: (Sender<String>, Receiver<String>),
    #[serde(skip)]
    file_buffer: String,
    #[serde(skip)]
    import_state: ImportState,

    #[serde(skip)]
    seconds_until_hiding_menu_bar: f32,
}

#[derive(PartialEq)]
enum ImportState {
    Free,
    Csv,
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

        graph.connect(a2, m24, graph::connection::Direction::ActivityToMutex, true);
        graph.connect(a4, m24, graph::connection::Direction::MutexToActivity, true);

        graph.connect(a1, m12, graph::connection::Direction::ActivityToMutex, true);
        graph.connect(a2, m12, graph::connection::Direction::MutexToActivity, true);

        graph.connect(a2, m234, graph::connection::Direction::TwoWay, true);
        graph.connect(a3, m234, graph::connection::Direction::TwoWay, true);
        graph.connect(a4, m234, graph::connection::Direction::TwoWay, true);

        graph.connect(a4, m46, graph::connection::Direction::ActivityToMutex, true);
        graph.connect(a6, m46, graph::connection::Direction::MutexToActivity, true);

        graph.connect(a1, m13, graph::connection::Direction::ActivityToMutex, true);
        graph.connect(a3, m13, graph::connection::Direction::MutexToActivity, true);

        graph.connect(a3, m36, graph::connection::Direction::ActivityToMutex, true);
        graph.connect(a6, m36, graph::connection::Direction::MutexToActivity, true);

        graph.connect(
            a6,
            m65a,
            graph::connection::Direction::ActivityToMutex,
            true,
        );
        graph.connect(
            a5a,
            m65a,
            graph::connection::Direction::MutexToActivity,
            true,
        );

        graph.connect(
            a5b,
            m5b1,
            graph::connection::Direction::ActivityToMutex,
            true,
        );
        graph.connect(
            a1,
            m5b1,
            graph::connection::Direction::MutexToActivity,
            true,
        );

        graph.connect(
            a5b,
            m5b5a,
            graph::connection::Direction::ActivityToMutex,
            true,
        );
        graph.connect(
            a5a,
            m5b5a,
            graph::connection::Direction::MutexToActivity,
            true,
        );

        graph.connect(
            a5a,
            m5a5b,
            graph::connection::Direction::ActivityToMutex,
            true,
        );
        graph.connect(
            a5b,
            m5a5b,
            graph::connection::Direction::MutexToActivity,
            true,
        );

        graph.name = "Example Graph".to_string();
        graph.toggle_play_pause();

        Self {
            stored_graphs: vec![graph.clone()],
            active_graph: graph,
            show_about_dialog: true,
            show_simulation_controls: true,
            pin_menu_bar: true,
            scaling_in_percent: 100.,
            text_channel: channel(),
            file_buffer: Default::default(),
            import_state: ImportState::Free,
            seconds_until_hiding_menu_bar: 0.,
        }
    }
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        creation_context.egui_ctx.set_visuals(egui::Visuals::dark());
        setup_custom_fonts(&creation_context.egui_ctx);

        // load previous app state, if it exists
        // create default otherwise
        let app: Self = match creation_context.storage {
            Some(storage) => eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            None => Default::default(),
        };

        egui_extras::install_image_loaders(&creation_context.egui_ctx);

        creation_context
            .egui_ctx
            .set_zoom_factor(1.5 * app.scaling_in_percent / 100.);

        app
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "sharetech".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/ShareTech.ttf")),
    );

    fonts.font_data.insert(
        "sharetechmono".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/ShareTechMono.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sharetech".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("sharetechmono".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // save app state
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const LOGO_IMAGESORUCE: egui::ImageSource<'static> =
            egui::include_image!("../../assets/Logo.png");
        if let Ok(text) = self.text_channel.1.try_recv() {
            self.file_buffer = text;
        }

        if !self.file_buffer.is_empty() && self.import_state != ImportState::Free {
            if self.import_state == ImportState::Csv {
                match Graph::from_csv(&self.file_buffer) {
                    Ok(graph) => {
                        self.active_graph = graph;
                    }
                    Err(e) => {
                        rfd::MessageDialog::new()
                            .set_title("Parser Error")
                            .set_description(format!("Failed to import graph: {}", e))
                            .set_level(rfd::MessageLevel::Error)
                            .show();
                    }
                }
            }
            self.file_buffer.clear();
        }

        if self.pin_menu_bar || ctx.pointer_interact_pos().map_or(false, |pos| pos.y < 25.) {
            self.seconds_until_hiding_menu_bar = 2.;
        } else if self.seconds_until_hiding_menu_bar > 0. {
            self.seconds_until_hiding_menu_bar -= ctx.input(|i| i.unstable_dt);
        }
        let show_menu_bar = self.pin_menu_bar || self.seconds_until_hiding_menu_bar > 0.;
        egui::TopBottomPanel::top("top_panel")
            .min_height(0.)
            .show_animated(ctx, show_menu_bar, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::menu::menu_button(ui, "File", |ui| {
                        if ui.button("📄 New Graph").clicked() {
                            ui.close_menu();
                            self.active_graph = Graph::default();
                        }

                        ui.separator();

                        if ui.button("💾 Save Graph").clicked() {
                            self.stored_graphs
                                .append(&mut vec![self.active_graph.clone()]);
                        }

                        ui.menu_button("📂 Load Graph", |ui| {
                            egui::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                                if self.stored_graphs.is_empty() {
                                    ui.label("nothing to load");
                                    return;
                                }

                                ui.spacing_mut().item_spacing.x = 3.;
                                let mut graph_to_delete = None;
                                for i in (0..self.stored_graphs.len()).rev() {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut self.stored_graphs[i].name,
                                            )
                                            .desired_width(100.),
                                        );
                                        if ui.button("🗑").clicked() {
                                            graph_to_delete = Some(i);
                                        }
                                        if ui.button("⬆").clicked()
                                            && i + 1 < self.stored_graphs.len()
                                        {
                                            self.stored_graphs.swap(i, i + 1);
                                        }
                                        if ui.button("⬇").clicked() && i > 0 {
                                            self.stored_graphs.swap(i - 1, i);
                                        }
                                        if ui.button("➡").clicked() {
                                            self.active_graph = self.stored_graphs[i].clone();
                                            ui.close_menu();
                                        }
                                    });
                                }
                                if let Some(i) = graph_to_delete {
                                    self.stored_graphs.remove(i);
                                }
                            });
                        });

                        ui.separator();

                        if ui.button("⬅ Export Graph").clicked() {
                            ui.close_menu();
                            let task = rfd::AsyncFileDialog::new()
                                .add_filter("Comma Seperated Values", &["csv"])
                                .add_filter("All Files", &["*"])
                                .set_file_name(format!("{}.csv", self.active_graph.name))
                                .save_file();
                            let contents = self.active_graph.to_csv();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    println!("{}", file.file_name());
                                    _ = file.write(contents.as_bytes()).await;
                                }
                            });
                        }

                        if ui.button("➡ Import Graph").clicked() {
                            ui.close_menu();
                            let sender = self.text_channel.0.clone();
                            let task = rfd::AsyncFileDialog::new()
                                .add_filter("Comma Seperated Values", &["csv"])
                                .add_filter("All Files", &["*"])
                                .pick_file();
                            execute(async move {
                                let file = task.await;
                                if let Some(file) = file {
                                    let text = file.read().await;
                                    let _ = sender.send(String::from_utf8_lossy(&text).to_string());
                                }
                            });
                            self.import_state = ImportState::Csv;
                        }
                    });
                    egui::menu::menu_button(ui, "Edit", |ui| {
                        if ui.button("🗑 Delete Mode").clicked() {
                            ui.close_menu();
                            self.active_graph.editing_mode = graph::EditingMode::Delete;
                        }
                    });
                    egui::menu::menu_button(ui, "View", |ui| {
                        if ui.button("[  ] Autofit Graph").clicked() {
                            self.active_graph.queue_autofit();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.pin_menu_bar, " Pin Menu Bar");
                        ui.checkbox(&mut self.show_simulation_controls, " Simulation Controls");
                        ui.checkbox(&mut self.show_about_dialog, " ℹ About");
                    });
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let previous_scaling = self.scaling_in_percent;
                        if ui.button("+").clicked() {
                            self.scaling_in_percent += 10.;
                            if self.scaling_in_percent > 300. {
                                self.scaling_in_percent = 300.;
                            }
                        }
                        let response = ui.add(
                            egui::DragValue::new(&mut self.scaling_in_percent)
                                .fixed_decimals(0)
                                .clamp_range(50.0..=300.0)
                                .suffix("%".to_owned())
                                .update_while_editing(false),
                        );
                        if response.double_clicked() {
                            self.scaling_in_percent = 100.;
                            response.surrender_focus();
                        };
                        if ui.button("-").clicked() {
                            self.scaling_in_percent -= 10.;
                            if self.scaling_in_percent < 50. {
                                self.scaling_in_percent = 50.;
                            }
                        };

                        if self.scaling_in_percent != previous_scaling {
                            ui.ctx()
                                .set_zoom_factor(1.5 * self.scaling_in_percent / 100.);
                        }

                        if ui
                            .label(
                                egui::RichText::new(match self.active_graph.editing_mode {
                                    graph::EditingMode::None => "",
                                    graph::EditingMode::Delete => {
                                        "Delete Mode active! Click here to exit."
                                    }
                                })
                                .color(egui::Color32::YELLOW),
                            )
                            .clicked()
                        {
                            self.active_graph.editing_mode = graph::EditingMode::None;
                        }
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.active_graph.name)
                                    .horizontal_align(egui::Align::Center)
                                    .frame(false),
                            );
                        });
                    });
                });
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(25.)
            .show_animated(ctx, self.show_simulation_controls, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.style_mut().spacing.slider_width = 175.;
                    let response = ui.add(
                        egui::widgets::Slider::new(
                            &mut self.active_graph.ticks_per_second,
                            0.1..=50.0,
                        )
                        .text("ticks per second")
                        .logarithmic(true)
                        .max_decimals(2),
                    );
                    if response.double_clicked() {
                        self.active_graph.ticks_per_second = 1.0;
                        response.surrender_focus();
                    };

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    match self.active_graph.is_running() {
                                        true => "⏸",
                                        false => "▶",
                                    }
                                    .to_string(),
                                )
                                .min_size(egui::vec2(25., 0.)),
                            )
                            .clicked()
                        {
                            self.active_graph.toggle_play_pause();
                        };
                        if !self.active_graph.is_running() {
                            let mut remaining_ticks =
                                self.active_graph.get_remaining_ticks_to_run();
                            let range = match remaining_ticks {
                                0 => 0..=1000,
                                _ => 1..=1000,
                            };
                            if ui.button("Single Step").clicked() {
                                self.active_graph.queue_tick();
                            }
                            ui.separator();
                            ui.label("ticks remaining");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut remaining_ticks)
                                        .update_while_editing(false)
                                        .speed(0.1)
                                        .clamp_range(range)
                                        .max_decimals(0),
                                )
                                .changed()
                            {
                                self.active_graph
                                    .set_remaining_ticks_to_run(remaining_ticks);
                            };
                        }
                    });
                });
            });

        egui::SidePanel::left("about_panel")
            .resizable(true)
            .default_width(350.)
            .min_width(200.)
            .max_width(500.)
            .show_animated(ctx, self.show_about_dialog, |ui| {
                ui.spacing_mut().item_spacing.x = 0.;
                ui.spacing_mut().item_spacing.y = 10.;

                // pinned close button
                let right_top = ui.available_rect_before_wrap().right_top();
                let rect = egui::Rect::from_points(&[right_top, right_top + egui::vec2(-20., 20.)]);
                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "❌", egui::FontId::proportional(15.), egui::Color32::GRAY);
                if ui.input(|i| i.pointer.primary_clicked()) && ui.rect_contains_pointer(rect) {
                    self.show_about_dialog = false;
                }

                egui::scroll_area::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.vertical_centered(|ui|ui.add(egui::Image::new(LOGO_IMAGESORUCE).tint(egui::Color32::LIGHT_GRAY).max_height(125.)));
                    ui.heading("Task Synchronization Simulator");

                    ui.label("A simple tool for simulating the execution of interdependent tasks.");

                    ui.label("Tasks block until all inputs are > 0, then all inputs are decremented and the task starts running. When a task finishes, all outputs are incremented. Using these simple rules, it is possible to model complex systems, including synchronization mechanisms like semaphores and mutexes.");

                    ui.horizontal_wrapped(|ui| {
                        ui.label("View on ");
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "GitHub",
                            "https://github.com/CrazyCraftix/tsyncs",
                        ).open_in_new_tab(true));
                        ui.label(" for more information and documentation.");
                    });

                    #[cfg(not(target_arch = "wasm32"))]
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Also, try the ");
                        ui.add(egui::Hyperlink::from_label_and_url("web version", "https://tsyncs.de").open_in_new_tab(true));
                        ui.label("!");
                    });

                    ui.separator();

                    ui.horizontal_wrapped(|ui| {
                        ui.label("This project was created as part of the course 'Echtzeitsysteme' at the ");
                        ui.add(egui::Hyperlink::from_label_and_url("DHBW Stuttgart", "https://www.dhbw-stuttgart.de/").open_in_new_tab(true));
                        ui.label(".");
                    });

                    ui.horizontal_wrapped(|ui| {
                        ui.label("Made with ♥ by ");
                        ui.add(egui::Hyperlink::from_label_and_url("Nicolai Bergmann", "https://github.com/CrazyCraftix").open_in_new_tab(true));
                        ui.label(" and ");
                        ui.add(egui::Hyperlink::from_label_and_url("Mark Orlando Zeller", "https://the-maze.net").open_in_new_tab(true));
                        ui.label(".\nPowered by ");
                        ui.add(egui::Hyperlink::from_label_and_url("egui", "https://github.com/emilk/egui").open_in_new_tab(true));
                        ui.label(" and ");
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/crates/eframe",
                        ).open_in_new_tab(true));
                        ui.label(".");
                    });
                });
            });

        // main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                graphics::PanZoomContainer::new().show(
                    ui,
                    |ui, container_transform, container_response| {
                        let image = egui::Image::new(LOGO_IMAGESORUCE);
                        let image_size = egui::vec2(120., 60.);
                        image
                            .shrink_to_fit()
                            .tint(egui::Color32::DARK_GRAY)
                            .paint_at(
                                ui,
                                egui::Rect::from_center_size(
                                    //container_transform.inverse()
                                    container_transform.inverse()
                                        * Pos2::new(
                                            container_response.rect.right() - image_size.x * 0.7,
                                            container_response.rect.bottom() - image_size.y * 0.5,
                                        ),
                                    image_size / container_transform.scaling,
                                ),
                            );

                        self.active_graph.tick(ui);
                        // skip first frame because interaction results don't exist yet
                        if ui.ctx().frame_nr() != 0 {
                            self.active_graph
                                .interact(ui, container_transform, container_response);
                        }
                        self.active_graph.draw(ui, *container_transform);
                    },
                );
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

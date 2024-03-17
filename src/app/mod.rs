mod graphics;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {}

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
        let draw_function = |ui: &mut egui::Ui| {
            if ui
                .put(
                    Rect::from_min_size(Pos2::ZERO, Vec2::new(200., 20.)),
                    egui::Button::new("this is a test button"),
                )
                .clicked()
            {
                log::error!("button was clicked!");
            };
            ui.put(
                Rect::from_min_size(Pos2::new(0., 25.), Vec2::new(200., 20.)),
                egui::Label::new("this is a test label"),
            );
            static mut STR: String = String::new();
            ui.put(
                Rect::from_min_size(Pos2::new(0., 50.), Vec2::new(200., 20.)),
                egui::TextEdit::singleline(unsafe { &mut STR }).desired_width(120.),
            );

            use egui::epaint::*;
            let painter = ui.painter();
            painter.rect(
                Rect::from_min_size(Pos2::new(800., 600.), Vec2::splat(300.)),
                0.,
                Color32::DEBUG_COLOR,
                Stroke::NONE,
            );
            painter.circle_filled(pos2(0.0, -10.0), 1.0, Rgba::from_gray(20.));
            painter.circle_filled(pos2(10.0, -10.0), 1.0, Color32::DARK_RED);
            painter.circle_filled(pos2(100.0, 100.0), 10.0, Color32::DARK_RED);
            painter.add(QuadraticBezierShape::from_points_stroke(
                [pos2(0.0, 0.0), pos2(5.0, 3.0), pos2(10.0, 0.0)],
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::YELLOW),
            ));
            painter.add(TextShape::new(
                pos2(800., 600.),
                ui.ctx().fonts(|f| {
                    f.layout_no_wrap(
                        "test txt".to_string(),
                        FontId::new(20., FontFamily::Monospace),
                        Color32::GREEN,
                    )
                }),
                Color32::RED,
            ));
            painter.text(
                pos2(600., 600.),
                egui::Align2::LEFT_TOP,
                "test txt",
                FontId::new(20., FontFamily::Monospace),
                Color32::GREEN,
            );
        };

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
                    .show(ui, draw_function);
            });
        });

        egui::Window::new("w1").show(ctx, |ui| {
            graphics::PanZoomContainer::new()
                .id_source("abc")
                .show(ui, draw_function);
        });
    }
}

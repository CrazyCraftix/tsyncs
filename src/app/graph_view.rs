use std::{sync::Arc, borrow::BorrowMut};

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct Transform {
    translation: egui::Vec2,
    scaling: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: egui::vec2(0., 0.),
            scaling: 1.,
        }
    }
}

impl Transform {
    fn apply(&self, vec: &egui::Vec2) -> egui::Vec2 {
        (*vec + self.translation) * self.scaling
    }
    fn apply_inverse(&self, vec: &egui::Vec2) -> egui::Vec2 {
        *vec / self.scaling - self.translation
    }
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GraphView {
    transform: Transform,
}

impl GraphView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let (id, rect) = ui.allocate_space(ui.available_size());
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        // panning
        if response.dragged() {
            self.transform.translation += response.drag_delta() / self.transform.scaling;
        }

        // zooming
        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if zoom_delta != 1. {
            let screen_space_zoom_anchor_position = ui.ctx().input(|i| i.pointer.latest_pos()).unwrap_or_default();

            let graph_space_zoom_anchor_position = self
                .transform
                .apply_inverse(&screen_space_zoom_anchor_position.to_vec2());

            self.transform.scaling *= zoom_delta;

            let new_screen_space_zoom_anchor_position = self.transform.apply(&graph_space_zoom_anchor_position);

            self.transform.translation += (screen_space_zoom_anchor_position.to_vec2()
                - new_screen_space_zoom_anchor_position)
                / self.transform.scaling;
        }

        // scrolling
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
        self.transform.translation += scroll_delta / self.transform.scaling;

        // reset on double click
        if response.double_clicked() {
            self.transform = Default::default();
        }

        {
            use egui::epaint::*;

            let mut shapes: Vec<Shape> = vec![];
            // Smiley face.
            shapes.push(CircleShape::filled(pos2(0.0, -10.0), 1.0, Rgba::from_gray(20.)).into());
            shapes.push(CircleShape::filled(pos2(10.0, -10.0), 1.0, Color32::YELLOW).into());
            shapes.push(
                QuadraticBezierShape::from_points_stroke(
                    [pos2(0.0, 0.0), pos2(5.0, 3.0), pos2(10.0, 0.0)],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, Color32::YELLOW),
                )
                .into(),
            );
            shapes.push(
                TextShape::new(
                    pos2(600., 600.),
                    ui.ctx().fonts(|f| {
                        f.layout_no_wrap(
                            "test abc".to_string(),
                            FontId::new(20., FontFamily::Monospace),
                            Color32::GREEN,
                        )
                    }),
                    Color32::RED,
                )
                .into(),
            );

            for shape in &mut shapes {
                shape.translate(self.transform.translation);
                self.scale_shape(shape);
            }

            let painter = ui.painter();
            painter.add(shapes);
        }
    }

    fn scale_shape(&mut self, shape: &mut egui::epaint::Shape) {
        use egui::epaint::*;
        match shape {
            Shape::Noop => {}
            Shape::Vec(shapes) => {
                for shape in shapes {
                    self.scale_shape(shape);
                }
            }
            Shape::Circle(CircleShape {
                center,
                radius,
                stroke,
                ..
            }) => {
                *center = *center * self.transform.scaling;
                *radius *= self.transform.scaling;
                stroke.width *= self.transform.scaling;
            }
            Shape::LineSegment { points, stroke } => {
                for p in points {
                    *p = *p * self.transform.scaling;
                }
                stroke.width *= self.transform.scaling;
            }
            Shape::Path(PathShape { points, stroke, .. }) => {
                for p in points {
                    *p = *p * self.transform.scaling;
                }
                stroke.width *= self.transform.scaling;
            }
            Shape::Rect(RectShape {
                rect,
                rounding,
                stroke,
                ..
            }) => {
                *rect = *rect * self.transform.scaling;
                rounding.nw *= self.transform.scaling;
                rounding.ne *= self.transform.scaling;
                rounding.sw *= self.transform.scaling;
                rounding.se *= self.transform.scaling;
                stroke.width *= self.transform.scaling;
            }
            Shape::Text(TextShape {
                pos,
                galley,
                underline,
                ..
            }) => {
                *pos = *pos * self.transform.scaling;
                //for section in &galley.job.sections {
                    //section.format.underline.width *= self.transform.scaling;
                    //section.format.extra_letter_spacing
                    //section.format.strikethrough
                    //section.format.line_height
                    //section.format.font_id.size
                //}
                underline.width *= self.transform.scaling;
            }
            Shape::Mesh(mesh) => {
                for Vertex { pos, .. } in &mut mesh.vertices {
                    *pos = *pos * self.transform.scaling;
                }
            }
            Shape::QuadraticBezier(QuadraticBezierShape { points, stroke, .. }) => {
                for p in points {
                    *p = *p * self.transform.scaling;
                }
                stroke.width *= self.transform.scaling;
            }
            Shape::CubicBezier(CubicBezierShape { points, stroke, .. }) => {
                for p in points {
                    *p = *p * self.transform.scaling;
                }
                stroke.width *= self.transform.scaling;
            }
            Shape::Callback(shape) => {
                shape.rect = shape.rect * self.transform.scaling;
            }
        }
    }
}

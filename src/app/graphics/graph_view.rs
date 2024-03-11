use crate::app::graphics::Drawer;
use egui::{layers::ShapeIdx, Align2, Color32, FontId, Pos2, Rect, Stroke};

#[derive(Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Transform {
    translation: egui::Vec2,
    scaling: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: egui::Vec2::ZERO,
            scaling: 1.,
        }
    }
}

impl Transform {
    fn apply(&self, vec: egui::Pos2) -> egui::Pos2 {
        (vec + self.translation) * self.scaling
    }
    fn apply_inverse(&self, vec: egui::Pos2) -> egui::Pos2 {
        vec / self.scaling - self.translation
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct GraphView {
    transform: Transform,
}

impl egui::Widget for GraphView {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        self.transform = ui.data_mut(|data| {
            data.get_persisted::<Transform>(egui::Id::NULL)
                .unwrap_or_default()
        });

        // panning
        if response.dragged() {
            self.transform.translation += response.drag_delta() / self.transform.scaling;
        }

        // zooming
        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
        if zoom_delta != 1. {
            let screen_space_zoom_anchor_position = ui
                .ctx()
                .input(|i| i.pointer.latest_pos())
                .unwrap_or_default();

            let graph_space_zoom_anchor_position = self
                .transform
                .apply_inverse(screen_space_zoom_anchor_position);

            self.transform.scaling *= zoom_delta;

            let new_screen_space_zoom_anchor_position =
                self.transform.apply(graph_space_zoom_anchor_position);

            self.transform.translation += (screen_space_zoom_anchor_position
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
            let painter = TransformablePainter::new(&painter, self.transform);
            painter.circle_filled(pos2(0.0, -10.0), 1.0, Rgba::from_gray(20.));
            painter.circle_filled(pos2(10.0, -10.0), 1.0, Color32::DARK_RED);
            //shapes.push(
            //    QuadraticBezierShape::from_points_stroke(
            //        [pos2(0.0, 0.0), pos2(5.0, 3.0), pos2(10.0, 0.0)],
            //        false,
            //        Color32::TRANSPARENT,
            //        Stroke::new(1.0, Color32::YELLOW),
            //    )
            //    .into(),
            //);
            //shapes.push(
            //    TextShape::new(
            //        pos2(600., 600.),
            //        ui.ctx().fonts(|f| {
            //            f.layout_no_wrap(
            //                "test txt".to_string(),
            //                FontId::new(20., FontFamily::Monospace),
            //                Color32::GREEN,
            //            )
            //        }),
            //        Color32::RED,
            //    )
            //    .into(),
            //);
            painter.text(
                pos2(600., 600.),
                Align2::LEFT_TOP,
                "test txt",
                FontId::new(20., FontFamily::Monospace),
                Color32::GREEN,
            );
        }

        ui.data_mut(|data| {
            data.insert_persisted(egui::Id::NULL, self.transform);
        });

        response
    }
}

impl GraphView {
    pub fn new() -> Self {
        Self::default()
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

/// wrapper around [egui::Painter]
/// translates and scales all positions and sizes before drawing
struct TransformablePainter<'a> {
    painter: &'a egui::Painter,
    transform: Transform,
}

impl TransformablePainter<'_> {
    fn new<'a>(painter: &'a egui::Painter, transform: Transform) -> TransformablePainter<'a> {
        TransformablePainter { painter, transform }
    }
    fn transform_pos2(&self, pos2: Pos2) -> Pos2 {
        self.transform.apply(pos2)
    }
    fn transform_f32(&self, f32: f32) -> f32 {
        self.transform.scaling * f32
    }
    fn transform_stroke(&self, stroke: impl Into<Stroke>) -> Stroke {
        let mut stroke = stroke.into();
        stroke.width *= self.transform.scaling;
        stroke
    }
    fn transform_font_id(&self, mut font_id: FontId) -> FontId {
        font_id.size *= self.transform.scaling;
        font_id
    }
}

impl super::Drawer for TransformablePainter<'_> {
    fn circle(
        &self,
        center: Pos2,
        radius: f32,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx {
        self.painter.circle(
            self.transform_pos2(center),
            self.transform_f32(radius),
            fill_color,
            self.transform_stroke(stroke),
        )
    }
    fn circle_filled(
        &self,
        center: Pos2,
        radius: f32,
        fill_color: impl Into<Color32>,
    ) -> egui::layers::ShapeIdx {
        self.painter.circle_filled(
            self.transform_pos2(center),
            self.transform_f32(radius),
            fill_color,
        )
    }
    fn circle_stroke(
        &self,
        center: Pos2,
        radius: f32,
        stroke: impl Into<Stroke>,
    ) -> egui::layers::ShapeIdx {
        self.painter.circle_stroke(
            self.transform_pos2(center),
            self.transform_f32(radius),
            self.transform_stroke(stroke),
        )
    }

    fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) -> Rect {
        self.painter.text(
            self.transform_pos2(pos),
            anchor,
            text,
            self.transform_font_id(font_id),
            text_color,
        )
    }
}

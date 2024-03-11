use egui::{layers::ShapeIdx, Align2, Color32, FontId, Pos2, Rect, Stroke};
pub use graph_view::*;

mod graph_view;

pub trait Drawer {
    fn circle(
        &self,
        center: Pos2,
        radius: f32,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) -> ShapeIdx;
    fn circle_filled(&self, center: Pos2, radius: f32, fill_color: impl Into<Color32>) -> ShapeIdx;
    fn circle_stroke(&self, center: Pos2, radius: f32, stroke: impl Into<Stroke>) -> ShapeIdx;
    fn text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) -> Rect;
}

pub trait Drawable {
    fn draw(drawer: impl Drawer);
}

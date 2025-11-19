use glam::{Vec2, Vec4};

pub struct OverlayText {
    pub text: String,
    pub position: Vec2,
    pub color: Vec4,
    pub size: f32,
}

pub enum Shape {
    Rect {
        pos: Vec2,
        size: Vec2,
        color: Vec4,
    },
    Circle {
        pos: Vec2,
        radius: f32,
        color: Vec4,
    },
    Line {
        a: Vec2,
        b: Vec2,
        width: f32,
        color: Vec4,
    },
}

pub struct OverlayShape {
    pub shape: Shape,
}

pub struct OverlayImage {
    pub pos: Vec2,
    pub size: Vec2,
    pub texture_id: u32,
}

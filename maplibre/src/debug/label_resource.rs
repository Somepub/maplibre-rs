use std::sync::RwLock;

#[derive(Default)]
pub struct LabelResource {
    pub labels: RwLock<Vec<Label>>,
}

#[derive(Clone)]
pub struct Label {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

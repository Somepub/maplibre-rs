use crate::debug::text_renderer::TextRenderer;
use std::sync::RwLock;

pub struct TextRendererResource {
    pub renderer: RwLock<Option<TextRenderer>>,
}

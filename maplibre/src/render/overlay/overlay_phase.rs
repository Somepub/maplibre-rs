use crate::render::render_phase::{Draw, PhaseItem};

pub enum OverlayKind {
    Text,
    //Shape,
    //Image,
}

pub struct OverlayItem {
    pub kind: OverlayKind,
    pub z: u32,
}

impl PhaseItem for OverlayItem {
    type SortKey = u32;

    fn sort_key(&self) -> Self::SortKey {
        self.sort
    }

    fn draw_function(&self) -> &dyn Draw<Self> {
        self.draw_fn.as_ref()
    }
}

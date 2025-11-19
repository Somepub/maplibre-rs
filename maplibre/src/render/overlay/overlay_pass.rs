use crate::{
    render::{
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext},
        overlay::overlay_phase::OverlayItem,
        render_phase::RenderPhase,
        RenderResources,
    },
    tcs::world::World,
};

pub struct OverlayPassNode;

impl Node for OverlayPassNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_ctx: &mut RenderContext,
        _state: &RenderResources,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(output_view) = graph.get_input_texture_view(0) else {
            return Ok(());
        };
        let mut pass = render_ctx.begin_tracked_pass("overlay_pass", &output_view);

        if let Some(mut phase) = world.resources.get_mut::<RenderPhase<OverlayItem>>() {
            phase.sort();

            for item in phase.into_iter() {
                item.draw_function().draw(&mut pass, world, item);
            }

            phase.clear();
        }

        Ok(())
    }
}

use std::ops::Deref;
use wgpu::StoreOp;

use crate::{
    debug::{text_resource::TextRendererResource, TileDebugItem},
    render::{
        eventually::Eventually::Initialized,
        graph::{Node, NodeRunError, RenderContext, RenderGraphContext, SlotInfo},
        render_phase::RenderPhase,
        resource::TrackedRenderPass,
        RenderResources,
    },
    tcs::world::World,
};

pub struct DebugPassNode;

impl DebugPassNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for DebugPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        resources: &RenderResources,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Initialized(render_target) = &resources.render_target else {
            return Ok(());
        };

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: render_target.deref(),
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: StoreOp::Store,
            },
            resolve_target: None,
        };

        let render_pass =
            render_context
                .command_encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("debug_pass"),
                    color_attachments: &[Some(color_attachment)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        let mut tracked_pass = TrackedRenderPass::new(render_pass);

        if let Some(items) = world.resources.get::<RenderPhase<TileDebugItem>>() {
            for item in items {
                item.draw_function.draw(&mut tracked_pass, world, item);
            }
        }

        if let Some(text_res) = world.resources.get::<TextRendererResource>() {
            let guard = text_res.renderer.read().unwrap();
            if let Some(renderer) = &*guard {
                renderer.draw(&mut tracked_pass);
            }
        }

        Ok(())
    }
}

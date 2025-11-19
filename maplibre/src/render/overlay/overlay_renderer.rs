use super::{overlay_phase::OverlayItem, overlay_text};
use crate::{
    render::{graph::RenderContext, RenderPhase},
    tcs::world::World,
};

pub struct OverlayDriverNode;

impl RenderGraphNode for OverlayDriverNode {
    fn run(&self, ctx: &mut RenderContext, _: &[NodeInput], _: &mut [NodeOutput]) {
        let phase = ctx
            .world
            .resources
            .get::<RenderPhase<OverlayItem>>()
            .unwrap();
        phase.sort();
    }
}

pub fn queue_overlay_system(world: &mut World) {
    let phase = world
        .resources
        .get_mut::<RenderPhase<OverlayItem>>()
        .unwrap();

    overlay_text::extract_text(world, phase);
    //overlay_shapes::extract_shapes(world, phase);
    //overlay_images::extract_images(world, phase);
}

pub fn render_overlay(ctx: &mut RenderContext) {
    let phase = ctx
        .world
        .resources
        .get::<RenderPhase<OverlayItem>>()
        .unwrap();

    let pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("overlay pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: ctx.get_current_view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    for item in phase.items.iter() {
        match item.kind {
            super::overlay_phase::OverlayKind::Text => overlay_text::draw(ctx, &item),
            //super::overlay_phase::OverlayKind::Shape => overlay_shapes::draw(ctx, &item),
            //super::overlay_phase::OverlayKind::Image => overlay_images::draw(ctx, &item),
        }
    }
}

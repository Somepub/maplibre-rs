use std::rc::Rc;

use super::{overlay_pass::OverlayPassNode, overlay_phase::OverlayItem};
use crate::{
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{graph::RenderGraph, main_graph, render_phase::RenderPhase, RenderStageLabel},
    schedule::Schedule,
    tcs::{
        system::{stage::SystemStage, SystemContainer},
        world::World,
    },
};

pub struct OverlayPlugin;

impl<E: Environment> Plugin<E> for OverlayPlugin {
    fn build(
        &self,
        schedule: &mut Schedule,
        _kernel: Rc<Kernel<E>>,
        world: &mut World,
        graph: &mut RenderGraph,
    ) {
        world.resources.init::<RenderPhase<OverlayItem>>();

        let mut overlay_graph = RenderGraph::default();

        overlay_graph.add_node("overlay_pass", OverlayPassNode::new());

        let input = overlay_graph.set_input(vec![]);
        overlay_graph.add_node_edge(input, "overlay_pass").unwrap();

        graph.add_sub_graph("overlay_graph", overlay_graph);

        graph.add_node("overlay_driver", super::overlay_renderer::OverlayDriverNode);
        graph
            .add_node_edge(main_graph::node::MAIN_PASS_DRIVER, "overlay_driver")
            .unwrap();

        schedule.add_stage(RenderStageLabel::Prepare, SystemStage::default());
        schedule.add_stage(
            RenderStageLabel::Queue,
            SystemStage::default().with_system(SystemContainer::new(
                super::overlay_renderer::queue_overlay_system,
            )),
        );
        schedule.add_stage(RenderStageLabel::Render, SystemStage::default());
    }
}

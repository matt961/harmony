use crate::{ AssetManager, Application };
use super::{ Pipeline, SimplePipeline, SimplePipelineDesc };
use super::{pipeline::PrepareResult, pipelines::UnlitPipelineDesc, Renderer};

// TODO: handle node dependencies somehow.
pub struct RenderGraphNode {
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline>,
    pub(crate) command_buffers: Vec<wgpu::CommandBuffer>,
    pub(crate) dirty: bool,
}

pub struct RenderGraph {
    nodes: Vec<RenderGraphNode>
}

impl RenderGraph {
    pub fn new(app: &mut Application) -> Self {
        let mut nodes = Vec::new();
        let mut unlit_pipeline_desc = UnlitPipelineDesc::default();
        let pipeline = unlit_pipeline_desc.pipeline(app);
        let simple_pipeline: Box<dyn SimplePipeline> = Box::new(unlit_pipeline_desc.build(&app.renderer.device));
        nodes.push(RenderGraphNode {
            pipeline,
            simple_pipeline,
            command_buffers: Vec::new(),
            dirty: true, // Nodes always dirty at first.
        });
        RenderGraph {
            nodes,
        }
    }

    pub fn add<T: SimplePipeline + Sized + 'static>(&mut self, _pipeline: T) {
        // TODO: fix this code up to support custom pipelines..
    }

    pub fn remove(&mut self, _index: usize) {
        // self.pipeline.remove(index);
    }

    pub fn length(&self) -> usize {
        self.nodes.len()
    }

    pub fn render(&mut self, renderer: &mut Renderer, asset_manager: &AssetManager, world: &mut specs::World, frame: &wgpu::SwapChainOutput) {
        for node in self.nodes.iter_mut() {
            let node: &mut RenderGraphNode = node;
            if node.simple_pipeline.prepare() == PrepareResult::Record || node.dirty {
                node.command_buffers.clear();
                node.command_buffers.push(node.simple_pipeline.render(frame, &renderer.device, asset_manager, world, &node.pipeline));
                node.dirty = false;
            }

            renderer.queue.submit(&node.command_buffers);
        }
    }
}
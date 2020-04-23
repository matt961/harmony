use crate::{
    graphics::{
        pipeline::VertexStateBuilder, resources::RenderTarget, Pipeline, SimplePipeline,
        SimplePipelineDesc,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct SpecularBRDFPipeline {
    size: f32,
}

impl SimplePipeline for SpecularBRDFPipeline {
    fn prepare<'a>(
        &'a mut self,
        _device: &'a mut wgpu::Device,
        _pipeline: &'a Pipeline,
        _encoder: &'a mut wgpu::CommandEncoder,
        _world: &'a mut specs::World,
        _asset_manager: &'a mut AssetManager,
        _input: Option<&RenderTarget>,
    ) {
    }

    fn render<'a>(
        &'a mut self,
        render_pass: &'a mut wgpu::RenderPass<'a>,
        pipeline: &'a Pipeline,
        _asset_manager: &'a mut AssetManager,
        _world: &'a mut specs::World,
    ) {
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

#[derive(Debug, Default)]
pub struct SpecularBRDFPipelineDesc {
    size: f32,
}

impl SpecularBRDFPipelineDesc {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl SimplePipelineDesc for SpecularBRDFPipelineDesc {
    type Pipeline = SpecularBRDFPipeline;

    fn load_shader<'a>(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("specular_brdf.shader")
    }

    fn create_layout(&self, _device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // No bindings? No problem! Just remember that later on!
        vec![]
    }
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor {
        wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }
    }
    fn primitive_topology(&self) -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleList
    }
    fn color_states_desc(
        &self,
        _sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
        None
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let vertex_state_builder = VertexStateBuilder::new();
        vertex_state_builder
    }

    fn build(
        self,
        _device: &wgpu::Device,
        _bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> SpecularBRDFPipeline {
        SpecularBRDFPipeline { size: self.size }
    }
}

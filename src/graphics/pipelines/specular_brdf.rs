use crate::{
    graphics::{
        pipeline::VertexStateBuilder, resources::RenderTarget, Pipeline, SimplePipeline,
        SimplePipelineDesc, Renderer,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct SpecularBRDFPipeline {
    size: f32,
}

impl<'a> SimplePipeline<'a> for SpecularBRDFPipeline {
    fn prepare<'b>(
        &'b mut self,
        device: &'b mut wgpu::Device,
        pipeline: &'b Pipeline,
        encoder: &'b mut wgpu::CommandEncoder,
        world: &'b mut specs::World,
        asset_manager: &'b mut AssetManager,
        input: Option<&RenderTarget>,
    ) {
    }

    fn render(
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

impl<'a> SimplePipelineDesc<'a> for SpecularBRDFPipelineDesc {
    type Pipeline = SpecularBRDFPipeline;

    fn load_shader<'b>(
        &self,
        asset_manager: &'b crate::AssetManager,
    ) -> &'b crate::graphics::material::Shader {
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
        _renderer: &mut Renderer,
        _bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> SpecularBRDFPipeline {
        SpecularBRDFPipeline { size: self.size }
    }
}

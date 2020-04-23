use crate::{
    graphics::{
        pipeline::VertexStateBuilder, Pipeline, SimplePipeline,
        SimplePipelineDesc, resources::RenderTarget, Renderer,
    },
    AssetManager,
};

#[derive(Debug)]
pub struct CubeProjectionPipeline {
    texture: String,
    size: f32,
    bind_group: Option<wgpu::BindGroup>,
}

impl<'a> SimplePipeline<'a> for CubeProjectionPipeline {
    fn prepare(
        &'a mut self,
        device: &'a mut wgpu::Device,
        pipeline: &'a Pipeline,
        _encoder: &'a mut wgpu::CommandEncoder,
        _world: &'a mut specs::World,
        asset_manager: &'a mut AssetManager,
        _input: Option<&RenderTarget>,
    ) {
        let image = asset_manager.get_image(self.texture.clone());

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bind_group_layouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image.sampler),
                },
            ],
            label: None,
        }));
    }

    fn render(
        &'a mut self,
        render_pass: &'a mut wgpu::RenderPass<'a>,
        pipeline: &'a Pipeline,
        _asset_manager: &'a mut AssetManager,
        _world: &'a mut specs::World,
    ) {
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..6, 0..6);
    }
}

#[derive(Debug, Default)]
pub struct CubeProjectionPipelineDesc {
    texture: String,
    size: f32,
}

impl CubeProjectionPipelineDesc {
    pub fn new(texture: String, size: f32) -> Self {
        Self { texture, size }
    }
}

impl<'a> SimplePipelineDesc<'a> for CubeProjectionPipelineDesc {
    type Pipeline = CubeProjectionPipeline;

    fn load_shader(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("hdr_to_cubemap.shader")
    }

    fn create_layout(&self, device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });

        vec![global_bind_group_layout]
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
            format: wgpu::TextureFormat::Rgba32Float,
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
    ) -> CubeProjectionPipeline {
        CubeProjectionPipeline {
            texture: self.texture,
            size: self.size,
            bind_group: None,
        }
    }
}

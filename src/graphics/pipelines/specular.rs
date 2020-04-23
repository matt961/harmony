use crate::{
    graphics::{
        material::skybox::SPEC_CUBEMAP_MIP_LEVELS, pipeline::VertexStateBuilder,
        resources::RenderTarget, Pipeline, SimplePipeline, SimplePipelineDesc, Renderer,
    },
    AssetManager,
};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Uniforms {
    pub roughness: f32,
    pub resoultion: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            roughness: 1.0,
            resoultion: 1024.0,
        }
    }
}

unsafe impl Zeroable for Uniforms {}
unsafe impl Pod for Uniforms {}

#[derive(Debug)]
pub struct SpecularPipeline {
    constants_buffer: wgpu::Buffer,
    resoultion: f32,
    bind_group: Option<wgpu::BindGroup>,
}

impl<'a> SimplePipeline<'a> for SpecularPipeline {
    fn prepare<'b>(
        &'b mut self,
        device: &'b mut wgpu::Device,
        pipeline: &'b Pipeline,
        encoder: &'b mut wgpu::CommandEncoder,
        world: &'b mut specs::World,
        asset_manager: &'b mut AssetManager,
        input: Option<&RenderTarget>,
    ) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bind_group_layouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.constants_buffer,
                        range: 0..std::mem::size_of::<Uniforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &input.as_ref().unwrap().texture_view,
                    ),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&input.as_ref().unwrap().sampler),
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
pub struct SpecularPipelineDesc {
    mip_level: u32,
    resoultion: f32,
}

impl SpecularPipelineDesc {
    pub fn new(mip_level: u32, resoultion: f32) -> Self {
        Self {
            mip_level,
            resoultion,
        }
    }
}

impl<'a> SimplePipelineDesc<'a> for SpecularPipelineDesc {
    type Pipeline = SpecularPipeline;

    fn load_shader<'b>(
        &self,
        asset_manager: &'b crate::AssetManager,
    ) -> &'b crate::graphics::material::Shader {
        asset_manager.get_shader("specular.shader")
    }

    fn create_layout(&self, device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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
        renderer: &mut Renderer,
        _bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> SpecularPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = renderer.device.create_buffer_with_data(
            bytemuck::bytes_of(&Uniforms {
                roughness: self.mip_level as f32 / (SPEC_CUBEMAP_MIP_LEVELS - 1) as f32,
                resoultion: self.resoultion,
            }),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        SpecularPipeline {
            constants_buffer,
            resoultion: self.resoultion,
            bind_group: None,
        }
    }
}

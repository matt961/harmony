use specs::RunNow;
use std::mem;

use super::GlobalUniforms;
use crate::{
    graphics::{
        mesh::MeshVertexData,
        pipeline::VertexStateBuilder,
        Pipeline,
        SimplePipeline,
        SimplePipelineDesc, renderer::DEPTH_FORMAT, resources::RenderTarget, Renderer,
    },
    scene::systems::{PrepareUnlit, RenderUnlit},
    AssetManager,
};

#[derive(Debug)]
pub struct UnlitPipeline {
    constants_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
}

impl<'a> SimplePipeline<'a> for UnlitPipeline {
    fn prepare(
        &'a mut self,
        device: &'a mut wgpu::Device,
        pipeline: &'a Pipeline,
        encoder: &'a mut wgpu::CommandEncoder,
        world: &'a mut specs::World,
        asset_manager: &'a mut AssetManager,
        _input: Option<&RenderTarget>,
    ) {
        let mut prepare_unlit = PrepareUnlit {
            device,
            encoder,
            asset_manager,
            pipeline,
            constants_buffer: &self.constants_buffer,
        };
        RunNow::setup(&mut prepare_unlit, world);
        prepare_unlit.run_now(world);
    }

    fn render(
        &'a mut self,
        render_pass: &'a mut wgpu::RenderPass<'a>,
        pipeline: &'a Pipeline,
        asset_manager: &'a mut AssetManager,
        world: &'a mut specs::World,
    ) {
        let mut render_unlit = RenderUnlit::<'a> {
            render_pass,
            asset_manager,
            pipeline,
            global_bind_group: &self.global_bind_group,
        };
        RunNow::setup(&mut render_unlit, world);
        render_unlit.run_now(world);
    }
}

#[derive(Debug, Default)]
pub struct UnlitPipelineDesc;

impl<'a> SimplePipelineDesc<'a> for UnlitPipelineDesc {
    type Pipeline = UnlitPipeline;

    fn load_shader(
        &self,
        asset_manager: &'a crate::AssetManager,
    ) -> &'a crate::graphics::material::Shader {
        asset_manager.get_shader("unlit.shader")
    }

    fn create_layout(&self, device: &mut wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        // We can create whatever layout we want here.
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
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
        vec![global_bind_group_layout, material_bind_group_layout]
    }
    fn rasterization_state_desc(&self) -> wgpu::RasterizationStateDescriptor {
        wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::Back,
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
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Vec<wgpu::ColorStateDescriptor> {
        vec![wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }]
    }

    fn depth_stencil_state_desc(&self) -> Option<wgpu::DepthStencilStateDescriptor> {
        Some(wgpu::DepthStencilStateDescriptor {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        })
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let vertex_size = mem::size_of::<MeshVertexData>();

        let mut vertex_state_builder = VertexStateBuilder::new();

        vertex_state_builder
            .set_index_format(wgpu::IndexFormat::Uint32)
            .new_buffer_descriptor(
                vertex_size as wgpu::BufferAddress,
                wgpu::InputStepMode::Vertex,
                vec![
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: 4 * 3,
                        shader_location: 1,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 4 * (3 + 3),
                        shader_location: 2,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float4,
                        offset: 4 * (3 + 3 + 2),
                        shader_location: 3,
                    },
                ],
            );

        vertex_state_builder
    }

    fn build(
        self,
        renderer: &mut Renderer,
        bind_group_layouts: &Vec<wgpu::BindGroupLayout>,
    ) -> UnlitPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = renderer.device.create_buffer_with_data(
            bytemuck::bytes_of(&GlobalUniforms::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &constants_buffer,
                    range: 0..std::mem::size_of::<GlobalUniforms>() as u64,
                },
            }],
            label: None,
        });

        UnlitPipeline {
            constants_buffer,
            global_bind_group,
        }
    }
}

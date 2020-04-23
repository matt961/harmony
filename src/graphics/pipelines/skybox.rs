use bytemuck::{Pod, Zeroable};
use nalgebra_glm::Mat4;
use specs::WorldExt;

use crate::{
    graphics::{
        pipeline::VertexStateBuilder, renderer::DEPTH_FORMAT, resources::RenderTarget, Pipeline,
        SimplePipeline, SimplePipelineDesc, Renderer,
    },
    scene::components::CameraData,
    AssetManager,
};

#[derive(Debug)]
pub struct SkyboxPipeline {
    constants_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SkyboxUniforms {
    pub proj: Mat4,
    pub view: Mat4,
}

impl Default for SkyboxUniforms {
    fn default() -> Self {
        Self {
            proj: Mat4::identity(),
            view: Mat4::identity(),
        }
    }
}

unsafe impl Zeroable for SkyboxUniforms {}
unsafe impl Pod for SkyboxUniforms {}

impl<'a> SimplePipeline<'a> for SkyboxPipeline {
    fn prepare<'b>(
        &'b mut self,
        device: &'b mut wgpu::Device,
        pipeline: &'b Pipeline,
        encoder: &'b mut wgpu::CommandEncoder,
        world: &'b mut specs::World,
        asset_manager: &'b mut AssetManager,
        input: Option<&RenderTarget>,
    ) {
        // Buffers can/are stored per mesh.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let skybox = world.try_fetch::<crate::graphics::material::Skybox>();
        if skybox.is_none() {
            return;
        }
        let camera_data = world.read_component::<CameraData>();

        use specs::Join;
        let filtered_camera_data: Vec<&CameraData> = camera_data
            .join()
            .filter(|data: &&CameraData| data.active)
            .collect();
        let camera_data = filtered_camera_data.first();

        if camera_data.is_none() {
            return;
        }

        let camera_data = camera_data.unwrap();

        let uniforms = SkyboxUniforms {
            proj: camera_data.projection,
            view: camera_data.view,
        };

        let constants_buffer = device
            .create_buffer_with_data(bytemuck::bytes_of(&uniforms), wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &constants_buffer,
            0,
            &self.constants_buffer,
            0,
            std::mem::size_of::<SkyboxUniforms>() as u64,
        );

        
    }

    fn render(
        &'a mut self,
        _render_pass: &'a mut wgpu::RenderPass<'a>,
        _pipeline: &'a Pipeline,
        _asset_manager: &'a mut AssetManager,
        _world: &'a mut specs::World,
    ) {
        // TODO: Move to system so lifetimes work.
        
        // let skybox = world.try_fetch::<crate::graphics::material::Skybox>();
        // if skybox.is_none() {
        //     return;
        // }
        // let skybox = skybox.as_ref().unwrap();

        // render_pass.set_pipeline(&pipeline.pipeline);
        // render_pass.set_bind_group(0, &self.global_bind_group, &[]);
        // render_pass.set_bind_group(1, skybox.cubemap_bind_group.as_ref().unwrap(), &[]);
        // render_pass.draw(0..3 as u32, 0..1);
    }
}

#[derive(Debug, Default)]
pub struct SkyboxPipelineDesc;

impl<'a> SimplePipelineDesc<'a> for SkyboxPipelineDesc {
    type Pipeline = SkyboxPipeline;

    fn load_shader<'b>(
        &self,
        asset_manager: &'b crate::AssetManager,
    ) -> &'b crate::graphics::material::Shader {
        asset_manager.get_shader("skybox.shader")
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
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::Cube,
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

        vec![global_bind_group_layout, material_bind_group_layout]
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
        // None
    }

    fn vertex_state_desc(&self) -> VertexStateBuilder {
        let mut vertex_state_builder = VertexStateBuilder::new();
        vertex_state_builder.set_index_format(wgpu::IndexFormat::Uint16);

        vertex_state_builder
    }

    fn build<'b>(
        self,
        renderer: &'b mut Renderer,
        bind_group_layouts: &'b Vec<wgpu::BindGroupLayout>,
    ) -> SkyboxPipeline {
        // This data needs to be saved and passed onto the pipeline.
        let constants_buffer = renderer.device.create_buffer_with_data(
            bytemuck::bytes_of(&SkyboxUniforms::default()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &constants_buffer,
                    range: 0..std::mem::size_of::<SkyboxUniforms>() as u64,
                },
            }],
            label: None,
        });

        SkyboxPipeline {
            constants_buffer,
            global_bind_group,
        }
    }
}

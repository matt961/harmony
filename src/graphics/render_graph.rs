use super::{resources::RenderTarget, Pipeline, Renderer, SimplePipeline, SimplePipelineDesc};
use crate::AssetManager;
use solvent::DepGraph;
use std::collections::HashMap;

// TODO: handle node dependencies somehow.
#[derive(Debug)]
pub struct RenderGraphNode<'a> {
    pub(crate) pipeline: Pipeline,
    pub(crate) simple_pipeline: Box<dyn SimplePipeline<'a>>,
    pub use_output_from_dependency: bool,
    pub create_cubemap_from_output: bool,
}

pub struct RenderGraph<'a> {
    nodes: HashMap<String, RenderGraphNode<'a>>,
    pub(crate) outputs: HashMap<String, Option<RenderTarget>>,
    dep_graph: DepGraph<String>,
    pub(crate) local_bind_group_layout: wgpu::BindGroupLayout,
}

impl<'a> RenderGraph<'a> {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let mut dep_graph = DepGraph::new();
        dep_graph.register_node("root".to_string());
        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: None,
            });

        RenderGraph {
            nodes: HashMap::new(),
            outputs: HashMap::new(),
            dep_graph,
            local_bind_group_layout,
        }
    }

    /// `input` - Optional view to render from. useful for post processing chains.
    /// 'output' - Optional view to render to. If none is set it will render to the latest frame buffer.
    pub fn add<'b, T: SimplePipelineDesc<'a> + Sized, T2: Into<String>>(
        &'a mut self,
        asset_manager: &'b AssetManager,
        renderer: &'b mut Renderer,
        name: T2,
        mut pipeline_desc: T,
        dependency: Vec<&str>,
        include_local_bindings: bool,
        output: Option<RenderTarget>,
        use_output_from_dependency: bool,
        create_cubemap_from_output: bool,
    ) {
        let name = name.into();
        let pipeline = {
            pipeline_desc.pipeline(
                asset_manager,
                renderer,
                if include_local_bindings {
                    Some(&self.local_bind_group_layout)
                } else {
                    None
                },
            )
        };
        let built_pipeline: Box<dyn SimplePipeline<'a>> =
            Box::new(pipeline_desc.build(renderer, &pipeline.bind_group_layouts));
        let node = RenderGraphNode {
            pipeline,
            simple_pipeline: built_pipeline,
            use_output_from_dependency,
            create_cubemap_from_output,
        };
        self.nodes.insert(name.clone(), node);
        self.outputs.insert(name.clone(), output);
        self.dep_graph.register_node(name.clone());
        if dependency.len() > 0 {
            let dependency = dependency
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            self.dep_graph
                .register_dependencies(name.clone(), dependency);
        }
    }

    /// Allows you to take the output render target for a given node.
    pub fn pull_render_target<T>(&mut self, name: T) -> RenderTarget
    where
        T: Into<String>,
    {
        let name = name.into();
        let output = self.outputs.get_mut(&name).unwrap();
        output.take().unwrap()
    }

    /// Allows you to take the output render target for a given node.
    pub fn get<T>(&self, name: T) -> &RenderGraphNode<'a>
    where
        T: Into<String>,
    {
        self.nodes.get(&name.into()).unwrap()
    }

    pub(crate) fn render(
        &'a mut self,
        renderer: &'a mut Renderer,
        asset_manager: &'a mut AssetManager,
        world: &'a mut specs::World,
        frame: Option<&'a wgpu::SwapChainOutput>,
    ) -> wgpu::CommandBuffer {
        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Root Encoder"),
        });

        let mut order = Vec::new();
        for (name, _) in self.nodes.iter_mut() {
            let dependencies = self.dep_graph.dependencies_of(&name);
            if dependencies.is_ok() {
                for node in dependencies.unwrap() {
                    match node {
                        Ok(n) => {
                            if !order.contains(n) {
                                order.push(n.clone());
                            }
                        }
                        Err(e) => panic!("Solvent error detected: {:?}", e),
                    }
                }
            }
        }

        for name in order {
            let node = self.nodes.get_mut(&name).unwrap();
            let mut input = None;
            if node.use_output_from_dependency {
                let dependencies = self.dep_graph.dependencies_of(&name);
                if dependencies.is_ok() {
                    let mut dependencies = dependencies.unwrap();
                    let dependency = dependencies.next().unwrap();
                    if dependency.is_ok() {
                        let dependency = dependency.unwrap().to_string();
                        input = self.outputs.get(&dependency).unwrap().as_ref();
                    }
                }
            }
            let output = self.outputs.get(&name).unwrap().as_ref();
            
            node.simple_pipeline.prepare(&mut renderer.device, &node.pipeline, &mut encoder, world, asset_manager, input);
        
            {
                let mut render_pass = if output.is_some() {
                    // Custom outputs don't get depth for now..
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &output.as_ref().unwrap().texture_view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                        }],
                        depth_stencil_attachment: None,
                    })
                } else if frame.is_some() {
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.as_ref().unwrap().view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Load,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            },
                        }],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &renderer.forward_depth,
                            depth_load_op: wgpu::LoadOp::Load,
                            depth_store_op: wgpu::StoreOp::Store,
                            stencil_load_op: wgpu::LoadOp::Load,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_depth: 1.0,
                            clear_stencil: 0,
                        }),
                    })
                } else { panic!("RenderGraph Error: Couldn't find an output to attach to the render pass."); };
                
                node.simple_pipeline.render(
                    &mut render_pass,
                    &node.pipeline,
                    asset_manager,
                    world,
                );
            }

            // This process should mostly happen during load. or with really small textures..
            if node.create_cubemap_from_output && output.is_some() {
                let old_output = output.unwrap();
                let output = RenderTarget::new(
                    &renderer.device,
                    old_output.size.width as f32,
                    (old_output.size.height / 6) as f32, // Weird case because original texture was/should be (width x height * 6)
                    6,
                    1,
                    old_output.format,
                    wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
                );

                for i in 0..6 {
                    encoder.copy_texture_to_texture(
                        wgpu::TextureCopyView {
                            texture: &old_output.texture,
                            mip_level: 0,
                            array_layer: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: (old_output.size.height / 6) as u32 * i,
                                z: 0,
                            },
                        },
                        wgpu::TextureCopyView {
                            texture: &output.texture,
                            mip_level: 0,
                            array_layer: i,
                            origin: wgpu::Origin3d::ZERO,
                        },
                        wgpu::Extent3d {
                            width: old_output.size.width as u32,
                            height: old_output.size.height / 6 as u32,
                            depth: 1,
                        },
                    );
                }

                self.outputs.insert(name.clone(), Some(output));
            }
        }

        encoder.finish()
    }
}

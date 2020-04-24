use crate::AssetManager;
use crate::{
    graphics::{material::Material, Pipeline},
    scene::components::{Mesh, Transform},
};
use specs::{ReadStorage, System};

pub struct RenderUnlit<'a, 'b> {
    pub(crate) render_pass: &'b mut wgpu::RenderPass<'a>,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) global_bind_group: &'a wgpu::BindGroup,
}

impl<'a, 'b, 'c> System<'c> for RenderUnlit<'a, 'b> {
    type SystemData = (
        ReadStorage<'c, Mesh>,
        ReadStorage<'c, crate::scene::components::Material>,
        ReadStorage<'c, Transform>,
    );

    fn run(&mut self, (meshes, materials, transforms): Self::SystemData) {
        use specs::Join;
      
        self.render_pass.set_pipeline(&self.pipeline.pipeline);
        self.render_pass.set_bind_group(1, self.global_bind_group, &[]);

        let asset_materials = self.asset_manager.get_materials();
        /*
            TODO: It's not very efficient to loop through each entity that has a material. Fix that.
            Look into using: https://docs.rs/specs/0.16.1/specs/struct.FlaggedStorage.html
        */
        for asset_material in asset_materials {
            let joined_data = (&meshes, &materials, &transforms).join();
            match asset_material {
                Material::Unlit(unlit_material) => {
                    self.render_pass.set_bind_group(
                        2,
                        &unlit_material.bind_group_data.as_ref().unwrap().bind_group,
                        &[],
                    );
                    for (mesh, _, transform) in joined_data
                        .filter(|(_, material, _)| material.index == unlit_material.index)
                    {
                        self.render_pass.set_bind_group(0, &transform.bind_group, &[]);
                        let mesh: &Mesh = mesh;
                        let asset_mesh = self.asset_manager.get_mesh(mesh.mesh_name.clone());
                        for sub_mesh in asset_mesh.sub_meshes.iter() {
                            // render_pass.set_bind_group(1, &current_bind_group.bind_group, &[]);
                            self.render_pass.set_index_buffer(&sub_mesh.index_buffer, 0, 0);
                            self.render_pass.set_vertex_buffer(0, &sub_mesh.vertex_buffer, 0, 0);
                            self.render_pass.draw_indexed(0..sub_mesh.index_count as u32, 0, 0..1);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

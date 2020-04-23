use crate::AssetManager;
use crate::{
    graphics::{
        material::{Material, Skybox},
        Pipeline,
    },
    scene::components::{
        Mesh, Transform,
    },
};
use specs::{Read, ReadStorage, System, WriteStorage};

pub struct RenderPBR<'a> {
    pub(crate) render_pass: &'a mut wgpu::RenderPass<'a>,
    pub(crate) asset_manager: &'a AssetManager,
    pub(crate) pipeline: &'a Pipeline,
    pub(crate) global_bind_group: &'a wgpu::BindGroup,
}

impl<'a> System<'a> for RenderPBR<'a> {
    type SystemData = (
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, crate::scene::components::Material>,
        WriteStorage<'a, Transform>,
        Option<Read<'a, Skybox>>,
    );

    fn run(
        &mut self,
        (meshes, materials, transforms, skybox): Self::SystemData,
    ) {
        use specs::Join;
        if transforms.count() == 0 || skybox.is_none() {
            return;
        }
        let skybox = skybox.as_ref().unwrap();
        
        self.render_pass.set_pipeline(&self.pipeline.pipeline);
        self.render_pass.set_bind_group(1, self.global_bind_group, &[]);
        self.render_pass.set_bind_group(3, skybox.pbr_bind_group.as_ref().unwrap(), &[]);

        let asset_materials = self.asset_manager.get_materials();
        /*
            TODO: It's not very efficient to loop through each entity that has a material. Fix that.
            Look into using: https://docs.rs/specs/0.16.1/specs/struct.FlaggedStorage.html
        */
        for asset_material in asset_materials {
            let joined_data = (&meshes, &materials, &transforms).join();
            match asset_material {
                Material::PBR(pbr_material) => {
                    self.render_pass.set_bind_group(
                        2,
                        &pbr_material.bind_group_data.as_ref().unwrap().bind_group,
                        &[],
                    );
                    for (mesh, _, transform) in
                        joined_data.filter(|(_, material, _)| material.index == pbr_material.index)
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

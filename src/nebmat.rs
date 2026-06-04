use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

#[derive(Clone, Copy, ShaderType)]
pub struct NebParams {
    pub col_a: Vec4,
    pub col_b: Vec4,
    pub p: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct NebulaMat {
    #[uniform(0)]
    pub params: NebParams,
}

impl Material for NebulaMat {
    fn fragment_shader() -> ShaderRef {
        "shaders/nebula.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> core::result::Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct NebMatPlugin;

impl Plugin for NebMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<NebulaMat>::default());
    }
}

use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

#[derive(Clone, Copy, ShaderType)]
pub struct JetParams {
    pub color: Vec4,
    pub p: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct JetMat {
    #[uniform(0)]
    pub params: JetParams,
}

impl Material for JetMat {
    fn fragment_shader() -> ShaderRef {
        "shaders/jet.wgsl".into()
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

pub struct JetMatPlugin;

impl Plugin for JetMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<JetMat>::default());
    }
}

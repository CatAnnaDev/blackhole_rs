use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

#[derive(Clone, Copy, ShaderType)]
pub struct SnParams {
    pub c0: Vec4,
    pub c1: Vec4,
    pub progress: f32,
    pub seed: f32,
    pub mode: f32,
    pub _b: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SupernovaMat {
    #[uniform(0)]
    pub params: SnParams,
}

impl Material for SupernovaMat {
    fn fragment_shader() -> ShaderRef {
        "shaders/supernova.wgsl".into()
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

pub struct SnMatPlugin;

impl Plugin for SnMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SupernovaMat>::default());
    }
}

use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub const SKY_RADIUS: f32 = 2.0e13;

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct StarSky {}

impl Material for StarSky {
    fn fragment_shader() -> ShaderRef {
        "shaders/starsky.wgsl".into()
    }
}

#[derive(Component)]
pub struct SkyDome;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<StarSky>::default());
    }
}

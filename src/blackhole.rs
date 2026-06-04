use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::post_process::bloom::Bloom;
use bevy::render::view::Hdr;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType, TextureFormat};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{Material2d, Material2dPlugin, MeshMaterial2d};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, PrimaryEguiContext};

use crate::AppMode;

#[derive(Clone, Copy, ShaderType)]
pub struct BhParams {
    pub cam_pos: Vec4,
    pub right: Vec4,
    pub up: Vec4,
    pub fwd: Vec4,
    pub screen: Vec4,
    pub disk: Vec4,
    pub misc: Vec4,
    pub opts: Vec4,
    pub cfg: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BlackHoleMaterial {
    #[uniform(0)]
    pub params: BhParams,
}

impl Material2d for BlackHoleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blackhole.wgsl".into()
    }
}

#[derive(Resource)]
pub struct BhView {
    pub spin: f32,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_deg: f32,
    pub temp: f32,
    pub brightness: f32,
    pub r_out: f32,
    pub exposure: f32,
    pub max_steps: f32,
    pub aa: f32,
    pub render_scale: f32,
    pub doppler: f32,
    pub procedural: f32,
    pub glow: f32,
    pub disk_on: f32,
    pub disk_inner: f32,
}

impl Default for BhView {
    fn default() -> Self {
        Self {
            spin: 0.6,
            radius: 30.0,
            yaw: 0.0,
            pitch: 0.060,
            fov_deg: 50.0,
            temp: 4500.0,
            brightness: 0.55,
            r_out: 18.7,
            exposure: 2.0,
            max_steps: 2400.0,
            aa: 2.0,
            render_scale: 0.5,
            doppler: 0.0,
            procedural: 0.0,
            glow: 0.5,
            disk_on: 1.0,
            disk_inner: 9.26,
        }
    }
}

#[derive(Resource)]
struct BhHandle(Handle<BlackHoleMaterial>);

#[derive(Resource, Default)]
struct BhIdle {
    sig: u64,
    still: u32,
}

fn view_sig(v: &BhView, w: f32, h: f32) -> u64 {
    let mut x: u64 = 1469598103934665603;
    for f in [
        v.spin, v.radius, v.yaw, v.pitch, v.fov_deg, v.temp, v.brightness,
        v.r_out, v.exposure, v.max_steps, v.aa, w, h,
        v.doppler, v.procedural, v.glow, v.disk_on,
        v.disk_inner,
    ] {
        x ^= f.to_bits() as u64;
        x = x.wrapping_mul(1099511628211);
    }
    x
}

#[derive(Component)]
struct BhQuad;

#[derive(Component)]
struct BhScene;

#[derive(Component)]
struct BhDisplay;

#[derive(Component)]
struct BhCam;

#[derive(Resource)]
struct BhImage {
    handle: Handle<Image>,
    w: u32,
    h: u32,
}

fn render_dims(win_w: f32, win_h: f32, scale: f32) -> (u32, u32) {
    let s = scale.clamp(0.2, 1.0);
    (
        ((win_w * s) as u32).max(1),
        ((win_h * s) as u32).max(1),
    )
}

fn new_target(images: &mut Assets<Image>, w: u32, h: u32) -> Handle<Image> {
    images.add(Image::new_target_texture(
        w,
        h,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ))
}

pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlackHoleMaterial>::default())
            .init_resource::<BhView>()
            .init_resource::<BhIdle>()
            .add_systems(OnEnter(AppMode::BlackHole), setup)
            .add_systems(OnExit(AppMode::BlackHole), teardown)
            .add_systems(
                Update,
                (bh_mouse_input, update_view)
                    .chain()
                    .run_if(in_state(AppMode::BlackHole)),
            );
    }
}

fn teardown(mut commands: Commands, q: Query<Entity, With<BhScene>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
    commands.remove_resource::<BhHandle>();
    commands.remove_resource::<BhImage>();
}

fn zeroed_params() -> BhParams {
    BhParams {
        cam_pos: Vec4::ZERO,
        right: Vec4::ZERO,
        up: Vec4::ZERO,
        fwd: Vec4::ZERO,
        screen: Vec4::ZERO,
        disk: Vec4::ZERO,
        misc: Vec4::ZERO,
        opts: Vec4::ZERO,
        cfg: Vec4::ZERO,
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlackHoleMaterial>>,
    mut images: ResMut<Assets<Image>>,
    view: Res<BhView>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let (ww, wh) = window
        .single()
        .map(|w| (w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or((1280.0, 720.0));
    let (rw, rh) = render_dims(ww, wh, view.render_scale);

    let img = new_target(&mut images, rw, rh);

    let mat = materials.add(BlackHoleMaterial {
        params: zeroed_params(),
    });

    commands.spawn((
        BhCam,
        BhScene,
        Camera2d,
        Camera {
            order: -1,
            ..default()
        },
        Hdr,
        Tonemapping::AcesFitted,
        Bloom {
            intensity: 0.28,
            low_frequency_boost: 0.8,
            ..Bloom::NATURAL
        },
        RenderTarget::Image(img.clone().into()),
        RenderLayers::layer(1),
    ));

    commands.spawn((
        BhQuad,
        BhScene,
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(mat.clone()),
        Transform::default().with_scale(Vec3::new(rw as f32, rh as f32, 1.0)),
        RenderLayers::layer(1),
    ));

    commands.spawn((BhScene, Camera2d, Camera::default(), PrimaryEguiContext));

    commands.spawn((
        BhDisplay,
        BhScene,
        Sprite {
            image: img.clone(),
            custom_size: Some(Vec2::new(ww, wh)),
            ..default()
        },
        Transform::default(),
    ));

    commands.insert_resource(BhHandle(mat));
    commands.insert_resource(BhImage {
        handle: img,
        w: rw,
        h: rh,
    });
}

fn bh_mouse_input(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut view: ResMut<BhView>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_pointer_input() {
            return;
        }
    }
    if buttons.pressed(MouseButton::Left) {
        let d = motion.delta;
        view.yaw -= d.x * 0.005;
        view.pitch = (view.pitch - d.y * 0.005).clamp(-1.5, 1.5);
    }
    let s = scroll.delta.y;
    if s != 0.0 {
        view.radius = (view.radius * (1.0 - s * 0.1)).clamp(2.5, 400.0);
    }
}

fn update_view(
    time: Res<Time>,
    view: Res<BhView>,
    handle: Res<BhHandle>,
    mut idle: ResMut<BhIdle>,
    mut bh_image: ResMut<BhImage>,
    mut materials: ResMut<Assets<BlackHoleMaterial>>,
    mut images: ResMut<Assets<Image>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut quad: Query<&mut Transform, With<BhQuad>>,
    mut bh_cam: Query<&mut RenderTarget, With<BhCam>>,
    mut display: Query<&mut Sprite, With<BhDisplay>>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    let w = win.width().max(1.0);
    let h = win.height().max(1.0);

    let (rw, rh) = render_dims(w, h, view.render_scale);
    if rw != bh_image.w || rh != bh_image.h {
        let new_img = new_target(&mut images, rw, rh);
        if let Ok(mut rt) = bh_cam.single_mut() {
            *rt = RenderTarget::Image(new_img.clone().into());
        }
        if let Ok(mut spr) = display.single_mut() {
            spr.image = new_img.clone();
        }
        bh_image.handle = new_img;
        bh_image.w = rw;
        bh_image.h = rh;
    }

    if let Ok(mut tf) = quad.single_mut() {
        tf.scale = Vec3::new(rw as f32, rh as f32, 1.0);
    }
    if let Ok(mut spr) = display.single_mut() {
        spr.custom_size = Some(Vec2::new(w, h));
    }

    let cp = view.pitch.cos();
    let eye = Vec3::new(
        view.radius * cp * view.yaw.cos(),
        view.radius * view.pitch.sin(),
        view.radius * cp * view.yaw.sin(),
    );
    let fwd = (-eye).normalize();
    let right = fwd.cross(Vec3::Y).normalize();
    let up = right.cross(fwd);

    let sig = view_sig(&view, w, h);
    if sig == idle.sig {
        idle.still = idle.still.saturating_add(1);
    } else {
        idle.sig = sig;
        idle.still = 0;
    }
    let eff_aa = if idle.still > 4 { view.aa.max(1.0) } else { 1.0 };

    let tan_half = (view.fov_deg.to_radians() * 0.5).tan();
    let aspect = w / h;

    if let Some(mat) = materials.get_mut(&handle.0) {
        mat.params = BhParams {
            cam_pos: eye.extend(0.0),
            right: right.extend(0.0),
            up: up.extend(0.0),
            fwd: fwd.extend(0.0),
            screen: Vec4::new(tan_half, aspect, 1.0 / rw as f32, 1.0 / rh as f32),
            disk: Vec4::new(view.brightness, view.temp, view.r_out, view.exposure),
            misc: Vec4::new(
                view.spin,
                view.max_steps,
                eff_aa,
                time.elapsed_secs(),
            ),
            opts: Vec4::new(
                view.doppler,
                view.procedural,
                view.glow,
                view.disk_on,
            ),
            cfg: Vec4::new(view.disk_inner, 0.0, 0.0, 0.0),
        };
    }
}

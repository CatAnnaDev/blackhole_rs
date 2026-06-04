use bevy::color::palettes;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::camera::Exposure;
use bevy::math::DVec3;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use bevy_egui::{EguiContexts, PrimaryEguiContext};
use big_space::prelude::*;

use crate::AppMode;
use crate::auroramat::{AuroraMat, AuroraParams};
use crate::cometmat::{CometMat, CometParams};
use crate::diskmat::{DiskMat, DiskParams};
use crate::jetmat::{JetMat, JetParams};
use crate::nebmat::{NebParams, NebulaMat};
use crate::planetgen::{PlanetKind, make_texture};
use crate::sky::{SKY_RADIUS, SkyDome, StarSky};

#[derive(Resource)]
pub struct OrbitView {
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
}

#[derive(Resource, Default)]
pub struct CineCam(pub bool);

fn cine_cam(
    cine: Res<CineCam>,
    mode: Res<CamMode>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitView>,
) {
    if cine.0 && *mode == CamMode::Follow {
        orbit.yaw += 0.22 * time.delta_secs();
    }
}

impl Default for OrbitView {
    fn default() -> Self {
        Self {
            yaw: std::f32::consts::PI,
            pitch: 0.35,
            dist: 11.0,
        }
    }
}

pub const G: f64 = 6.674e-11;
pub const SUN_MASS: f64 = 1.98892e30;
pub const C_LIGHT: f64 = 299_792_458.0;
pub const EARTH_MASS: f64 = 5.972e24;
pub const MOON_MASS: f64 = 7.342e22;

pub const SUN_RADIUS_M: f64 = 695_508_000.0;
pub const EARTH_ORBIT_M: f64 = 149.60e9;
pub const EARTH_RADIUS_M: f64 = 6.371e6;
pub const MOON_ORBIT_M: f64 = 3.844e8;
pub const MOON_RADIUS_M: f64 = 1.7375e6;

#[derive(Component)]
pub struct SolarScene;

#[derive(Component)]
pub struct Grav {
    pub mass: f64,
    pub pos: DVec3,
    pub vel: DVec3,
}

#[derive(Component)]
pub struct FloatingCam;

#[derive(Component)]
pub struct Shape {
    pub base_radius: f64,
    pub radius: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BodyClass {
    Rocky,
    Terran,
    GasGiant,
    IceGiant,
    RedDwarf,
    SunLike,
    RedGiant,
    WhiteDwarf,
    NeutronStar,
    Pulsar,
    Magnetar,
    Quasar,
    BlackHole,
    Comet,
    OStar,
    BlueSupergiant,
    RedSupergiant,
    WolfRayet,
    CarbonStar,
    BrownDwarf,
    LavaWorld,
    OceanWorld,
    DesertWorld,
    CarbonPlanet,
    IntermediateBlackHole,
    SupermassiveBlackHole,
}

pub fn is_black_hole(c: BodyClass) -> bool {
    matches!(
        c,
        BodyClass::BlackHole
            | BodyClass::IntermediateBlackHole
            | BodyClass::SupermassiveBlackHole
    )
}

impl BodyClass {
    pub fn all() -> [BodyClass; 24] {
        use BodyClass::*;
        [
            Rocky, Terran, GasGiant, IceGiant, RedDwarf, SunLike, RedGiant,
            WhiteDwarf, NeutronStar, Pulsar, Magnetar, Quasar, BlackHole,
            Comet, OStar, BlueSupergiant, RedSupergiant, WolfRayet,
            CarbonStar, BrownDwarf, LavaWorld, OceanWorld, DesertWorld,
            CarbonPlanet,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            BodyClass::Rocky => "Rocky",
            BodyClass::Terran => "Terran",
            BodyClass::GasGiant => "Gas giant",
            BodyClass::IceGiant => "Ice giant",
            BodyClass::RedDwarf => "Red dwarf",
            BodyClass::SunLike => "Sun-like",
            BodyClass::RedGiant => "Red giant",
            BodyClass::WhiteDwarf => "White dwarf",
            BodyClass::NeutronStar => "Neutron star",
            BodyClass::Pulsar => "Pulsar",
            BodyClass::Magnetar => "Magnetar",
            BodyClass::Quasar => "Quasar",
            BodyClass::BlackHole => "Black hole",
            BodyClass::Comet => "Comet",
            BodyClass::OStar => "O-type star",
            BodyClass::BlueSupergiant => "Blue supergiant",
            BodyClass::RedSupergiant => "Red supergiant",
            BodyClass::WolfRayet => "Wolf-Rayet",
            BodyClass::CarbonStar => "Carbon star",
            BodyClass::BrownDwarf => "Brown dwarf",
            BodyClass::LavaWorld => "Lava world",
            BodyClass::OceanWorld => "Ocean world",
            BodyClass::DesertWorld => "Desert world",
            BodyClass::CarbonPlanet => "Carbon planet",
            BodyClass::IntermediateBlackHole => {
                "Intermediate black hole"
            }
            BodyClass::SupermassiveBlackHole => {
                "Supermassive black hole"
            }
        }
    }
    /// (mass kg, radius m, luminous, temperature K, kind for texture)
    pub fn props(&self) -> (f64, f64, bool, f32, PlanetKind) {
        match self {
            BodyClass::Rocky => {
                (3.3e23, 2.44e6, false, 0.0, PlanetKind::Rocky { r: 0.55, g: 0.5, b: 0.47 })
            }
            BodyClass::Terran => {
                (5.97e24, 6.37e6, false, 0.0, PlanetKind::Terran)
            }
            BodyClass::GasGiant => (
                1.898e27,
                6.99e7,
                false,
                0.0,
                PlanetKind::GasBands { warm: true },
            ),
            BodyClass::IceGiant => (
                1.02e26,
                2.46e7,
                false,
                0.0,
                PlanetKind::IceGiant { deep: true },
            ),
            BodyClass::RedDwarf => {
                (1.6e29, 1.0e8, true, 3200.0, PlanetKind::Terran)
            }
            BodyClass::SunLike => {
                (1.989e30, 6.96e8, true, 5778.0, PlanetKind::Terran)
            }
            BodyClass::RedGiant => {
                (1.5e30, 3.0e10, true, 3500.0, PlanetKind::Terran)
            }
            BodyClass::WhiteDwarf => {
                (1.2e30, 6.4e6, true, 14000.0, PlanetKind::Terran)
            }
            BodyClass::NeutronStar => {
                (2.8e30, 1.2e4, true, 1.0e6, PlanetKind::Terran)
            }
            BodyClass::Pulsar => {
                (2.78e30, 1.2e4, true, 1.2e6, PlanetKind::Terran)
            }
            BodyClass::Magnetar => {
                (2.9e30, 1.1e4, true, 1.0e6, PlanetKind::Terran)
            }
            BodyClass::Quasar => {
                (2.0e38, 1.0e11, true, 12000.0, PlanetKind::Terran)
            }
            BodyClass::BlackHole => {
                (1.2e31, 1.8e4, false, 0.0, PlanetKind::Rocky {
                    r: 0.01,
                    g: 0.01,
                    b: 0.02,
                })
            }
            BodyClass::Comet => {
                (2.2e13, 2.0e3, false, 0.0, PlanetKind::Rocky {
                    r: 0.42,
                    g: 0.46,
                    b: 0.52,
                })
            }
            BodyClass::OStar => {
                (8.0e31, 7.0e9, true, 40000.0, PlanetKind::Terran)
            }
            BodyClass::BlueSupergiant => {
                (4.2e31, 5.1e10, true, 12100.0, PlanetKind::Terran)
            }
            BodyClass::RedSupergiant => {
                (3.2e31, 4.9e11, true, 3600.0, PlanetKind::Terran)
            }
            BodyClass::WolfRayet => {
                (3.0e31, 3.5e9, true, 40000.0, PlanetKind::Terran)
            }
            BodyClass::CarbonStar => {
                (4.0e30, 2.5e11, true, 3000.0, PlanetKind::Terran)
            }
            BodyClass::BrownDwarf => {
                (1.0e29, 8.4e7, true, 1200.0, PlanetKind::Terran)
            }
            BodyClass::LavaWorld => {
                (3.0e25, 1.1e7, false, 0.0, PlanetKind::Lava)
            }
            BodyClass::OceanWorld => {
                (1.8e25, 9.5e6, false, 0.0, PlanetKind::Ocean)
            }
            BodyClass::DesertWorld => {
                (5.0e24, 5.8e6, false, 0.0, PlanetKind::Desert)
            }
            BodyClass::CarbonPlanet => {
                (6.0e24, 6.4e6, false, 0.0, PlanetKind::Carbon)
            }
            BodyClass::IntermediateBlackHole => {
                (2.0e33, 3.0e6, false, 0.0, PlanetKind::Rocky {
                    r: 0.01,
                    g: 0.01,
                    b: 0.02,
                })
            }
            BodyClass::SupermassiveBlackHole => {
                (8.0e36, 1.2e10, false, 0.0, PlanetKind::Rocky {
                    r: 0.01,
                    g: 0.01,
                    b: 0.02,
                })
            }
        }
    }
    pub fn emissive(&self) -> LinearRgba {
        let (_, _, lum, t, _) = self.props();
        if !lum {
            return LinearRgba::BLACK;
        }
        let c = if matches!(self, BodyClass::CarbonStar) {
            [0.86, 0.11, 0.03]
        } else {
            crate::planetgen::blackbody_rgb(t)
        };
        let i = match self {
            BodyClass::NeutronStar => 320.0,
            BodyClass::Pulsar => 380.0,
            BodyClass::Magnetar => 360.0,
            BodyClass::Quasar => 260.0,
            BodyClass::WhiteDwarf => 220.0,
            BodyClass::RedGiant => 90.0,
            BodyClass::RedDwarf => 70.0,
            BodyClass::OStar => 260.0,
            BodyClass::WolfRayet => 280.0,
            BodyClass::BlueSupergiant => 210.0,
            BodyClass::RedSupergiant => 95.0,
            BodyClass::CarbonStar => 70.0,
            BodyClass::BrownDwarf => 8.0,
            _ => 170.0,
        };
        LinearRgba::rgb(c[0] * i, c[1] * i, c[2] * i)
    }
}

#[derive(Resource)]
pub struct NewBodyType(pub BodyClass);

impl Default for NewBodyType {
    fn default() -> Self {
        Self(BodyClass::Rocky)
    }
}

#[derive(Component)]
pub struct Spin(pub f32);

pub const SUN_MS_LIFE: f64 = 4.0e9;
pub const AU_M: f64 = 1.495_978_707e11;

#[derive(Component)]
pub struct Temperature(pub f32);

#[derive(Component)]
pub struct TidalHeat(pub f32);

#[derive(Component)]
pub struct Atmosphere(pub f32);

#[derive(Component)]
pub struct AtmoShell;

#[derive(Component)]
pub struct AuroraShell(pub Entity);

const SIGMA: f64 = 5.670e-8;
const K_TIDE: f64 = 5.0e-11;
const K_B: f64 = 1.380_649e-23;
const M_N2: f64 = 4.651_73e-26;
const M_H2: f64 = 3.347_2e-27;
const ESC_TAU: f64 = 3.0e4;
const ESC_LAM0: f64 = 3.0;

const TIDE_Q: f64 = 100.0;
const TIDE_K2: f64 = 0.3;

fn tidal_locking(
    paused: Res<Paused>,
    time: Res<Time>,
    speed: Res<SimSpeed>,
    field_q: Query<&Grav>,
    mut bodies: Query<(&Grav, &Shape, &Visual, &mut Spin)>,
) {
    if paused.0 {
        return;
    }
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    let field: Vec<(f64, DVec3, DVec3)> = field_q
        .iter()
        .map(|g| (g.mass, g.pos, g.vel))
        .collect();
    for (g, sh, v, mut spin) in &mut bodies {
        if v.star {
            continue;
        }
        let rs = sh.base_radius.max(1.0);
        let mut best = (0.0_f64, 0.0_f64, DVec3::ZERO, DVec3::ZERO);
        for &(mo, po, pv) in &field {
            let d = (g.pos - po).length();
            if d < rs {
                continue;
            }
            let tide = mo / (d * d * d);
            if tide > best.0 {
                best = (tide, mo, po, pv);
            }
        }
        let (_, mp, po, pv) = best;
        if mp <= 0.0 {
            continue;
        }
        let mu = G * mp;
        let r = g.pos - po;
        let vv = g.vel - pv;
        let rm = r.length().max(1.0);
        let v2 = vv.dot(vv);
        let sma = 1.0 / (2.0 / rm - v2 / mu);
        if sma <= 0.0 {
            continue;
        }
        let evec = (r * (v2 - mu / rm) - vv * r.dot(vv)) / mu;
        let e = evec.length().clamp(0.0, 0.95);
        let n = (mu / (sma * sma * sma)).sqrt();
        let w_eq = if e > 0.15 {
            n * 1.5
        } else {
            n * (1.0 + 6.0 * e * e)
        };
        let w = spin.0 as f64;
        let wa = w.abs().max(1.0e-12);
        let tau = wa * sma.powi(6) * 0.4 * g.mass * TIDE_Q
            / (3.0 * G * mp * mp * TIDE_K2 * rs.powi(3));
        let frac = if tau <= 0.0 {
            1.0
        } else {
            (sim_dt / tau).clamp(0.0, 1.0)
        };
        let nw = w + (w_eq - w) * frac;
        spin.0 = nw as f32;
    }
}

fn planet_climate(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stars: Query<(&Grav, &Visual)>,
    mut bodies: Query<(
        Entity,
        &Grav,
        &Shape,
        &Visual,
        &MeshMaterial3d<StandardMaterial>,
        Option<&mut Temperature>,
    )>,
) {
    let star = stars.iter().filter(|(_, v)| v.star).fold(
        (0.0_f64, DVec3::ZERO),
        |acc, (g, _)| if g.mass > acc.0 { (g.mass, g.pos) } else { acc },
    );
    if star.0 <= 0.0 {
        return;
    }
    let field: Vec<(f64, DVec3)> =
        stars.iter().map(|(g, _)| (g.mass, g.pos)).collect();
    let lum = (star.0 / SUN_MASS).powf(3.5);
    for (e, g, sh, v, mm, temp) in &mut bodies {
        if v.star || g.mass > 0.02 * SUN_MASS {
            continue;
        }
        let d_au = ((g.pos - star.1).length() / AU_M).max(1.0e-3);
        let t_eq = 278.5 * 0.915 * lum.powf(0.25) / d_au.sqrt();
        let greenhouse = if v.atmo.is_some() { 33.0 } else { 0.0 };
        let t_eq_g = t_eq + greenhouse;

        let mut best = (0.0_f64, 0.0_f64);
        for &(mo, po) in &field {
            let d = (g.pos - po).length();
            if d < sh.base_radius.max(1.0) {
                continue;
            }
            let tide = mo / (d * d * d);
            if tide > best.0 {
                best = (tide, d);
            }
        }
        let t_tidal = if best.1 > 0.0 {
            let rs = sh.base_radius.max(1.0e3);
            let mo = best.0 * best.1 * best.1 * best.1;
            let flux =
                K_TIDE * G * mo * mo * rs * rs * rs / best.1.powi(6);
            (flux.max(0.0) / SIGMA).powf(0.25)
        } else {
            0.0
        };
        let t_surf =
            (t_eq_g.powi(4) + t_tidal.powi(4)).powf(0.25) as f32;
        commands.entity(e).insert(TidalHeat(t_tidal as f32));
        match temp {
            Some(mut t) => t.0 = t_surf,
            None => {
                commands.entity(e).insert(Temperature(t_surf));
            }
        }
        let tint = if t_surf < 248.0 {
            Color::srgb(0.78, 0.86, 1.0)
        } else if t_surf < 330.0 {
            Color::WHITE
        } else if t_surf < 600.0 {
            Color::srgb(1.0, 0.66, 0.45)
        } else {
            Color::srgb(1.0, 0.42, 0.22)
        };
        if let Some(mat) = materials.get_mut(&mm.0) {
            mat.base_color = tint;
            mat.emissive = if t_surf >= 600.0 {
                let k = ((t_surf - 600.0) / 1500.0).clamp(0.0, 1.0);
                LinearRgba::rgb(2.5 * k, 0.6 * k, 0.15 * k)
            } else {
                LinearRgba::BLACK
            };
        }
    }
}

fn atmosphere_escape(
    time: Res<Time>,
    speed: Res<SimSpeed>,
    paused: Res<Paused>,
    mut commands: Commands,
    children: Query<&Children>,
    shells: Query<(), With<AtmoShell>>,
    mut tf: Query<&mut Transform, With<AtmoShell>>,
    mut bodies: Query<(
        Entity,
        &Grav,
        &Shape,
        &Temperature,
        &mut Visual,
        Option<&mut Atmosphere>,
    )>,
) {
    if paused.0 {
        return;
    }
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    for (e, g, sh, t, mut v, atmo) in &mut bodies {
        if v.star || v.atmo.is_none() {
            continue;
        }
        let r = sh.base_radius.max(1.0e3);
        let temp = (t.0 as f64).max(3.0);
        let m_gas = match v.kind {
            PlanetKind::GasBands { .. } | PlanetKind::IceGiant { .. } => {
                M_H2
            }
            _ => M_N2,
        };
        let lambda = G * g.mass * m_gas / (K_B * temp * r);
        let mut frac = match atmo {
            Some(a) => a.0,
            None => {
                commands.entity(e).insert(Atmosphere(1.0));
                1.0
            }
        };
        if lambda < 36.0 {
            let tau = ESC_TAU * (lambda / ESC_LAM0).exp();
            let loss = (frac as f64) * sim_dt / tau;
            frac = (frac - loss as f32).max(0.0);
            commands.entity(e).insert(Atmosphere(frac));
        }
        if let Ok(kids) = children.get(e) {
            for &k in kids {
                if shells.get(k).is_ok() {
                    if frac <= 0.02 {
                        commands.entity(k).despawn();
                    } else if let Ok(mut s) = tf.get_mut(k) {
                        let gw = 0.04 * frac;
                        s.scale = Vec3::splat(1.0 + gw.max(0.004));
                    }
                }
            }
        }
        if frac <= 0.02 {
            v.atmo = None;
            commands.entity(e).remove::<Atmosphere>();
        }
    }
}

#[derive(Component)]
pub struct Star {
    pub age: f64,
    pub t_ms: f64,
    pub stage: u8,
}

pub fn ms_lifetime(mass: f64) -> f64 {
    (mass / SUN_MASS).max(0.02).powf(-2.5) * SUN_MS_LIFE
}

pub fn is_main_seq(mass: f64, radius: f64, lum: bool) -> bool {
    lum && radius > 1.0e7 && mass < 60.0 * SUN_MASS
}

#[derive(Component)]
pub struct Jet;

#[derive(Component)]
pub struct FocusRadius(pub f64);

#[derive(Component)]
pub struct JetRig {
    pub body: Entity,
    pub rate: f32,
}

fn sync_jet_rigs(
    time: Res<Time>,
    mut commands: Commands,
    grid: Query<&Grid>,
    bodies: Query<&Grav>,
    mut rigs: Query<(Entity, &JetRig, &mut CellCoord, &mut Transform)>,
) {
    let Ok(grid) = grid.single() else {
        return;
    };
    let t = time.elapsed_secs();
    for (e, rig, mut cell, mut tf) in &mut rigs {
        let Ok(g) = bodies.get(rig.body) else {
            commands.entity(e).despawn();
            continue;
        };
        let (c, off) = grid.translation_to_grid(g.pos);
        *cell = c;
        *tf = Transform::from_translation(off)
            .with_rotation(Quat::from_rotation_y(t * rig.rate));
    }
}

pub fn jet_class(c: BodyClass) -> Option<f32> {
    match c {
        BodyClass::Pulsar | BodyClass::Magnetar => Some(0.0),
        BodyClass::Quasar => Some(1.0),
        _ => None,
    }
}

pub fn disk_kind(c: BodyClass) -> Option<f32> {
    match c {
        BodyClass::BlackHole
        | BodyClass::IntermediateBlackHole => Some(0.0),
        BodyClass::Quasar
        | BodyClass::SupermassiveBlackHole => Some(1.0),
        _ => None,
    }
}

pub fn is_exotic(c: BodyClass) -> bool {
    jet_class(c).is_some() || disk_kind(c).is_some()
}

type JetBundle = (Mesh3d, MeshMaterial3d<JetMat>, Transform, Jet);

fn make_jets(
    meshes: &mut Assets<Mesh>,
    jets: &mut Assets<JetMat>,
    radius: f64,
    kind: f32,
) -> (Vec<JetBundle>, f64) {
    let pulsar = kind < 0.5;
    let base = (radius as f32 * 60.0).clamp(3.0e8, 9.0e11);
    let h = base * if pulsar { 16.0 } else { 11.0 };
    let jr = base * if pulsar { 0.085 } else { 0.20 };
    let mesh = meshes.add(
        Cylinder::new(jr, h)
            .mesh()
            .resolution(32)
            .without_caps()
            .build(),
    );
    let color = if pulsar {
        Vec4::new(0.58, 0.74, 1.0, 1.0)
    } else {
        Vec4::new(0.62, 0.78, 1.0, 1.0)
    };
    let gain = if pulsar { 7.0 } else { 4.5 };
    let tilt: f32 = if pulsar { 0.30 } else { 0.05 };
    let up = Vec3::new(0.0, tilt.cos(), tilt.sin());
    let mut out = Vec::new();
    for (i, d) in [up, -up].iter().enumerate() {
        let d = *d;
        let beam = if i == 0 {
            1.0
        } else if pulsar {
            0.85
        } else {
            0.18
        };
        let m = jets.add(JetMat {
            params: JetParams {
                color,
                p: Vec4::new(0.0, h, kind, gain * beam),
            },
        });
        out.push((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(m),
            Transform {
                translation: d * (h * 0.5),
                rotation: Quat::from_rotation_arc(Vec3::Y, d),
                ..default()
            },
            Jet,
        ));
    }
    (out, h as f64)
}

fn make_disk(
    meshes: &mut Assets<Mesh>,
    disks: &mut Assets<DiskMat>,
    radius: f64,
    kind: f32,
) -> ((Mesh3d, MeshMaterial3d<DiskMat>, Transform, Jet), f64) {
    let quasar = kind > 0.5;
    let base = (radius as f32 * 40.0).clamp(2.5e8, 7.0e11);
    let r_in = base * 1.6;
    let r_out = (base * if quasar { 12.0 } else { 9.0 }).min(6.0e12);
    let mesh = meshes.add(
        Annulus::new(r_in, r_out)
            .mesh()
            .resolution(96)
            .build(),
    );
    let gain = if quasar { 5.5 } else { 3.2 };
    let mat = disks.add(DiskMat {
        params: DiskParams {
            hot: Vec4::new(0.85, 0.92, 1.0, 1.0),
            cool: Vec4::new(1.0, 0.52, 0.18, 1.0),
            p: Vec4::new(0.0, 1.0, 3.0 + kind * 4.0, gain),
        },
    });
    let tf = Transform::from_rotation(Quat::from_rotation_x(
        -std::f32::consts::FRAC_PI_2,
    ));
    ((Mesh3d(mesh), MeshMaterial3d(mat), tf, Jet), r_out as f64)
}

fn spawn_exotic_rig(
    commands: &mut Commands,
    grid_entity: Entity,
    grid: &Grid,
    body: Entity,
    pos: DVec3,
    meshes: &mut Assets<Mesh>,
    jet_mats: &mut Assets<JetMat>,
    disk_mats: &mut Assets<DiskMat>,
    class: BodyClass,
) -> f64 {
    let (cell, off) = grid.translation_to_grid(pos);
    let rate = match class {
        BodyClass::Pulsar | BodyClass::Magnetar => 2.4,
        BodyClass::Quasar => 0.5,
        _ => 0.06,
    };
    let rig = commands
        .spawn((
            JetRig { body, rate },
            cell,
            Transform::from_translation(off),
            Visibility::Visible,
        ))
        .id();
    commands.entity(grid_entity).add_child(rig);

    let (_, radius, _, _, _) = class.props();
    let mut reach = 0.0_f64;
    let jets = jet_class(class)
        .map(|k| make_jets(meshes, jet_mats, radius, k));
    let disk = disk_kind(class)
        .map(|k| make_disk(meshes, disk_mats, radius, k));
    if let Some((_, h)) = &jets {
        reach = reach.max(*h);
    }
    if let Some((_, ro)) = &disk {
        reach = reach.max(*ro);
    }
    commands.entity(rig).with_children(|c| {
        if let Some((bundles, _)) = jets {
            for b in bundles {
                c.spawn(b);
            }
        }
        if let Some((db, _)) = disk {
            c.spawn(db);
        }
    });
    reach.max(radius)
}

#[derive(Component)]
pub struct CometRig {
    pub body: Entity,
}

type CometBundle = (Mesh3d, MeshMaterial3d<CometMat>, Transform, Jet);

fn make_comet(
    meshes: &mut Assets<Mesh>,
    comets: &mut Assets<CometMat>,
    radius: f64,
) -> (Vec<CometBundle>, f64) {
    let coma_r = (radius as f32 * 400.0).max(6.0e10);
    let ion_len = coma_r * 26.0;
    let dust_len = coma_r * 15.0;
    let ion = meshes.add(
        Cone {
            radius: coma_r * 0.35,
            height: ion_len,
        }
        .mesh()
        .resolution(28),
    );
    let dust = meshes.add(
        Cone {
            radius: coma_r * 1.5,
            height: dust_len,
        }
        .mesh()
        .resolution(28),
    );
    let coma = meshes.add(Sphere::new(coma_r * 0.45).mesh().ico(4).unwrap());
    let anti_len = coma_r * 9.0;
    let anti = meshes.add(
        Cone {
            radius: coma_r * 1.25,
            height: anti_len,
        }
        .mesh()
        .resolution(24),
    );
    let anti_mat = comets.add(CometMat {
        params: CometParams {
            color: Vec4::new(1.0, 0.95, 0.78, 1.0),
            p: Vec4::new(0.0, 0.0, 3.0, 1.0),
        },
    });
    let anti_dir = Vec3::new(0.22, -1.0, 0.0).normalize();

    let ion_mat = comets.add(CometMat {
        params: CometParams {
            color: Vec4::new(0.45, 0.65, 1.0, 1.0),
            p: Vec4::new(0.0, 0.0, 0.0, 2.6),
        },
    });
    let dust_mat = comets.add(CometMat {
        params: CometParams {
            color: Vec4::new(1.0, 0.92, 0.72, 1.0),
            p: Vec4::new(0.0, 0.0, 1.0, 1.7),
        },
    });
    let coma_mat = comets.add(CometMat {
        params: CometParams {
            color: Vec4::new(0.7, 0.85, 1.0, 1.0),
            p: Vec4::new(0.0, 0.0, 2.0, 2.2),
        },
    });

    let dust_dir = Quat::from_rotation_x(0.34) * Vec3::Y;
    let bundles = vec![
        (
            Mesh3d(ion),
            MeshMaterial3d(ion_mat),
            Transform {
                translation: Vec3::Y * (ion_len * 0.5),
                ..default()
            },
            Jet,
        ),
        (
            Mesh3d(dust),
            MeshMaterial3d(dust_mat),
            Transform {
                translation: dust_dir * (dust_len * 0.5),
                rotation: Quat::from_rotation_arc(Vec3::Y, dust_dir),
                ..default()
            },
            Jet,
        ),
        (
            Mesh3d(anti),
            MeshMaterial3d(anti_mat),
            Transform {
                translation: anti_dir * (anti_len * 0.5),
                rotation: Quat::from_rotation_arc(Vec3::Y, anti_dir),
                ..default()
            },
            Jet,
        ),
        (
            Mesh3d(coma),
            MeshMaterial3d(coma_mat),
            Transform::default(),
            Jet,
        ),
    ];
    (bundles, (coma_r * 4.5) as f64)
}

fn spawn_comet_rig(
    commands: &mut Commands,
    grid_entity: Entity,
    grid: &Grid,
    body: Entity,
    pos: DVec3,
    meshes: &mut Assets<Mesh>,
    comets: &mut Assets<CometMat>,
    radius: f64,
) -> f64 {
    let (cell, off) = grid.translation_to_grid(pos);
    let (bundles, reach) = make_comet(meshes, comets, radius);
    let rig = commands
        .spawn((
            CometRig { body },
            cell,
            Transform::from_translation(off),
            Visibility::Visible,
        ))
        .id();
    commands.entity(grid_entity).add_child(rig);
    commands.entity(rig).with_children(|c| {
        for b in bundles {
            c.spawn(b);
        }
    });
    reach
}

fn sync_comet_rigs(
    mut commands: Commands,
    grid: Query<&Grid>,
    bodies: Query<(&Grav, Entity)>,
    mut rigs: Query<(Entity, &CometRig, &mut CellCoord, &mut Transform)>,
) {
    let Ok(grid) = grid.single() else {
        return;
    };
    let star = bodies.iter().fold(
        (0.0_f64, DVec3::ZERO),
        |acc, (g, _)| if g.mass > acc.0 { (g.mass, g.pos) } else { acc },
    );
    for (e, rig, mut cell, mut tf) in &mut rigs {
        let Ok((g, _)) = bodies.get(rig.body) else {
            commands.entity(e).despawn();
            continue;
        };
        let anti = g.pos - star.1;
        let dir = if anti.length() > 1.0 {
            anti.normalize().as_vec3()
        } else {
            Vec3::Y
        };
        let (c, off) = grid.translation_to_grid(g.pos);
        *cell = c;
        *tf = Transform::from_translation(off)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, dir));
    }
}

const SUBL_Z3: f64 = 2.8e-5;
const SUBL_R_AU: f64 = 3.0;
const COMET_DEATH: f64 = 1.0e10;

fn comet_sublimation(
    paused: Res<Paused>,
    time: Res<Time>,
    speed: Res<SimSpeed>,
    mut commands: Commands,
    mut comets: ResMut<Assets<CometMat>>,
    rigs: Query<(&CometRig, &Children)>,
    childmat: Query<&MeshMaterial3d<CometMat>>,
    mut q: Query<(Entity, &mut Grav, &mut Shape)>,
) {
    if paused.0 {
        return;
    }
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    let star = q.iter().fold(
        (0.0_f64, DVec3::ZERO),
        |acc, (_, g, _)| {
            if g.mass > acc.0 {
                (g.mass, g.pos)
            } else {
                acc
            }
        },
    );
    for (rig, kids) in &rigs {
        let Ok((be, g, s)) = q.get(rig.body) else {
            continue;
        };
        let (bm, bp, br, bbase) =
            (g.mass, g.pos, s.radius, s.base_radius);
        let r_au = ((bp - star.1).length() / AU_M).max(1.0e-4);
        let act = if r_au < SUBL_R_AU {
            (SUBL_R_AU / r_au).powi(2)
        } else {
            0.0
        };
        let vis = (act / 9.0).clamp(0.0, 1.6) as f32;
        for &k in kids {
            if let Ok(mm) = childmat.get(k) {
                if let Some(m) = comets.get_mut(&mm.0) {
                    let base = match m.params.p.z as i32 {
                        0 => 2.6,
                        1 => 1.7,
                        _ => 2.2,
                    };
                    m.params.p.w = base * vis;
                }
            }
        }
        if act > 0.0 {
            let z = SUBL_Z3 * (SUBL_R_AU / r_au).powi(2);
            let area = std::f64::consts::PI * br * br;
            let dm = z * area * sim_dt;
            let nm = bm - dm;
            if nm <= COMET_DEATH {
                commands.entity(be).despawn();
                continue;
            }
            let shrink = (nm / bm).cbrt();
            if let Ok((_, mut gg, mut ss)) = q.get_mut(be) {
                gg.mass = nm;
                ss.radius = (br * shrink).max(bbase * 0.05);
            }
        }
    }
}

fn animate_comets(time: Res<Time>, mut mats: ResMut<Assets<CometMat>>) {
    let t = time.elapsed_secs();
    for (_, m) in mats.iter_mut() {
        m.params.p.x = t;
    }
}

fn animate_jets(time: Res<Time>, mut mats: ResMut<Assets<JetMat>>) {
    let t = time.elapsed_secs();
    for (_, m) in mats.iter_mut() {
        m.params.p.x = t;
    }
}

fn animate_disks(time: Res<Time>, mut mats: ResMut<Assets<DiskMat>>) {
    let t = time.elapsed_secs();
    for (_, m) in mats.iter_mut() {
        m.params.p.x = t;
    }
}

fn animate_nebulae(time: Res<Time>, mut mats: ResMut<Assets<NebulaMat>>) {
    let t = time.elapsed_secs();
    for (_, m) in mats.iter_mut() {
        m.params.p.x = t;
    }
}

fn aurora_drive(
    time: Res<Time>,
    mut auroras: ResMut<Assets<AuroraMat>>,
    bodies: Query<(&Grav, &Visual)>,
    shells: Query<(&AuroraShell, &MeshMaterial3d<AuroraMat>)>,
) {
    let star = bodies.iter().filter(|(_, v)| v.star).fold(
        (0.0_f64, DVec3::ZERO),
        |a, (g, _)| {
            if g.mass > a.0 {
                (g.mass, g.pos)
            } else {
                a
            }
        },
    );
    let tt = time.elapsed_secs();
    for (sh, mm) in &shells {
        let Some(mat) = auroras.get_mut(&mm.0) else {
            continue;
        };
        mat.params.p.x = tt;
        if let Ok((g, _)) = bodies.get(sh.0) {
            let r_au =
                ((g.pos - star.1).length() / AU_M).max(1.0e-3);
            let act = ((1.0 / r_au).powi(2) * 0.6).clamp(0.0, 1.0);
            mat.params.p.y = act as f32;
        }
    }
}

#[derive(Component, Default)]
pub struct Trail {
    pub pts: std::collections::VecDeque<DVec3>,
}

const TRAIL_CAP: usize = 320;

#[derive(Resource)]
pub struct SizeExaggeration(pub f32);

impl Default for SizeExaggeration {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Resource, Default)]
pub struct Selected(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct SpawnRequest(pub bool);

#[derive(Resource)]
pub struct SimSpeed(pub f64);

impl Default for SimSpeed {
    fn default() -> Self {
        Self(2.0e5)
    }
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(Resource, Default)]
pub struct SimClock(pub f64);

#[derive(Resource, Default)]
pub struct ShowGrid(pub bool);

#[derive(Resource)]
pub struct Relativity(pub bool);

impl Default for Relativity {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Resource, Default)]
pub struct PrecessionInfo {
    pub pred_arcsec: f64,
    pub pred_century: f64,
}

#[derive(Resource, Default)]
pub struct OrbitInfo {
    pub valid: bool,
    pub a_au: f64,
    pub e: f64,
    pub inc_deg: f64,
    pub period_days: f64,
    pub peri_au: f64,
    pub apo_au: f64,
    pub hill_au: f64,
    pub primary: String,
    pub resonance: String,
    pub class: String,
    pub v_esc_kms: f64,
    pub vinf_kms: f64,
    pub reflex_ms: f64,
    pub bary_rsun: f64,
    pub sat_status: String,
    pub teq_k: f64,
    pub hz: String,
    pub roche_au: f64,
    pub roche_status: String,
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn precession(
    sel: Res<Selected>,
    mut info: ResMut<PrecessionInfo>,
    mut orb: ResMut<OrbitInfo>,
    bodies: Query<(Entity, &Grav, &Name, Option<&Shape>)>,
) {
    info.pred_arcsec = 0.0;
    orb.valid = false;
    let Some(target) = sel.0 else {
        return;
    };
    let Ok((_, bg, _, bsh)) = bodies.get(target) else {
        return;
    };
    let tr = bsh.map(|s| s.radius).unwrap_or(0.0);
    let mut pm = 0.0_f64;
    let mut ppos = DVec3::ZERO;
    let mut pvel = DVec3::ZERO;
    let mut pname = String::new();
    let mut best_a = f64::MAX;
    let mut best_pull = 0.0_f64;
    let mut fb_pm = 0.0_f64;
    let mut fb_pos = DVec3::ZERO;
    let mut fb_vel = DVec3::ZERO;
    let mut fb_name = String::new();
    let mut pr = 0.0_f64;
    let mut fb_pr = 0.0_f64;
    for (e, g, nm, sh) in &bodies {
        if e == target || g.mass <= bg.mass {
            continue;
        }
        let grad = sh.map(|s| s.radius).unwrap_or(0.0);
        let rc = bg.pos - g.pos;
        let vc = bg.vel - g.vel;
        let rcm = rc.length().max(1.0);
        let muc = G * g.mass;
        let ec = vc.dot(vc) / 2.0 - muc / rcm;
        let pull = g.mass / (rcm * rcm);
        if pull > best_pull {
            best_pull = pull;
            fb_pm = g.mass;
            fb_pos = g.pos;
            fb_vel = g.vel;
            fb_name = nm.as_str().to_string();
            fb_pr = grad;
        }
        if ec < 0.0 {
            let ac = -muc / (2.0 * ec);
            if ac < best_a {
                best_a = ac;
                pm = g.mass;
                ppos = g.pos;
                pvel = g.vel;
                pname = nm.as_str().to_string();
                pr = grad;
            }
        }
    }
    if pm <= 0.0 {
        pm = fb_pm;
        ppos = fb_pos;
        pvel = fb_vel;
        pname = fb_name;
        pr = fb_pr;
    }
    if pm <= 0.0 {
        return;
    }
    let mu = G * pm;
    let r = bg.pos - ppos;
    let v = bg.vel - pvel;
    let rm = r.length().max(1.0);
    let vv = v.dot(v);
    let a = 1.0 / (2.0 / rm - vv / mu);
    let evec = (r * (vv - mu / rm) - v * r.dot(v)) / mu;
    let e = evec.length().clamp(0.0, 50.0);
    let energy = vv / 2.0 - mu / rm;
    let v_esc = (2.0 * mu / rm).sqrt();
    let bound = energy < 0.0 && a > 0.0;
    let hvec = r.cross(v);
    let hm = hvec.length().max(1.0);
    let inc = (-hvec.y / hm).clamp(-1.0, 1.0).acos();
    let hill = rm * (bg.mass / (3.0 * pm)).cbrt();

    orb.valid = true;
    orb.a_au = a / AU_M;
    orb.e = e;
    orb.inc_deg = inc.to_degrees();
    orb.peri_au = a * (1.0 - e) / AU_M;
    orb.primary = pname;

    {
        let mr = pm / 1.989e30_f64;
        if mr > 0.07 && mr < 25.0 {
            let lum = if mr < 0.43 {
                0.23 * mr.powf(2.3)
            } else if mr < 2.0 {
                mr.powi(4)
            } else {
                1.4 * mr.powf(3.5)
            };
            let d_au = if bound {
                (a / AU_M).abs()
            } else {
                rm / AU_M
            };
            orb.teq_k = 278.5 * lum.powf(0.25) / d_au.sqrt()
                * (1.0 - 0.3_f64).powf(0.25);
            let s = lum / (d_au * d_au);
            orb.hz = if s > 1.014 {
                format!("too hot · S={s:.2} S⊕")
            } else if s < 0.343 {
                format!("too cold · S={s:.2} S⊕")
            } else {
                format!("✓ in habitable zone · S={s:.2} S⊕")
            };
        } else {
            orb.teq_k = 0.0;
            orb.hz = String::new();
        }
    }

    {
        let _ = pr;
        if tr > 0.0 && pm > bg.mass {
            let d_roche =
                2.44 * tr * (pm / bg.mass).powf(1.0 / 3.0);
            let closest = if bound {
                a * (1.0 - e)
            } else {
                rm
            };
            orb.roche_au = d_roche / AU_M;
            orb.roche_status = if closest < d_roche {
                "INSIDE — tidal disruption".into()
            } else if closest < 1.5 * d_roche {
                "grazing — strong tides".into()
            } else {
                "safe".into()
            };
        } else {
            orb.roche_au = 0.0;
            orb.roche_status = String::new();
        }
    }

    {
        let mut gp_pull = 0.0_f64;
        let mut gmass = 0.0_f64;
        let mut gpos = DVec3::ZERO;
        for (oe, og, _, _) in &bodies {
            if oe == target || og.pos == ppos {
                continue;
            }
            let d = (og.pos - ppos).length().max(1.0);
            let pull = og.mass / (d * d);
            if pull > gp_pull {
                gp_pull = pull;
                gmass = og.mass;
                gpos = og.pos;
            }
        }
        if gmass > pm && pm > 0.0 {
            let a_par = (ppos - gpos).length().max(1.0);
            let rh_par =
                a_par * (pm / (3.0 * gmass)).cbrt();
            let frac = rm / rh_par.max(1.0);
            orb.sat_status = if frac < 0.47 {
                format!("stable ({frac:.2} R_H)")
            } else if frac < 0.67 {
                format!("marginal ({frac:.2} R_H)")
            } else if frac < 1.0 {
                format!("unstable ({frac:.2} R_H)")
            } else {
                format!("UNBOUND — stripping ({frac:.2} R_H)")
            };
        } else {
            orb.sat_status = String::new();
        }
    }

    {
        let mut msum = 0.0_f64;
        let mut wpos = DVec3::ZERO;
        let mut sun = (0.0_f64, DVec3::ZERO);
        for (_, g, _, _) in &bodies {
            msum += g.mass;
            wpos += g.pos * g.mass;
            if g.mass > sun.0 {
                sun = (g.mass, g.pos);
            }
        }
        if msum > 0.0 && sun.0 > 0.0 {
            let com = wpos / msum;
            orb.bary_rsun =
                (sun.1 - com).length() / 6.957e8;
            let mut pl = (0.0_f64, DVec3::ZERO, DVec3::ZERO);
            for (_, g, _, _) in &bodies {
                if g.mass < 0.05 * sun.0 && g.mass > pl.0 {
                    pl = (g.mass, g.pos, g.vel);
                }
            }
            if pl.0 > 0.0 {
                let mus = G * sun.0;
                let rr = (pl.1 - sun.1).length().max(1.0);
                let vv2 = pl.2.length_squared();
                let ap = 1.0 / (2.0 / rr - vv2 / mus);
                if ap > 0.0 {
                    let pp = std::f64::consts::TAU
                        * (ap * ap * ap / mus).sqrt();
                    let k = (std::f64::consts::TAU * G / pp)
                        .powf(1.0 / 3.0)
                        * pl.0
                        / sun.0.powf(2.0 / 3.0);
                    orb.reflex_ms = k;
                }
            }
        }
    }
    orb.v_esc_kms = v_esc / 1000.0;
    if bound {
        let period =
            std::f64::consts::TAU * (a * a * a / mu).sqrt();
        orb.period_days = period / 86400.0;
        orb.apo_au = a * (1.0 + e) / AU_M;
        orb.hill_au = hill / AU_M;
        orb.vinf_kms = 0.0;
        orb.class = if e < 0.02 {
            "bound · near-circular".into()
        } else {
            "bound · elliptic".into()
        };
    } else {
        orb.period_days = 0.0;
        orb.apo_au = 0.0;
        orb.hill_au = 0.0;
        orb.vinf_kms = (2.0 * energy.max(0.0)).sqrt() / 1000.0;
        orb.class = if energy.abs()
            < 1.0e-6 * (mu / rm)
        {
            "parabolic · escaping".into()
        } else {
            "HYPERBOLIC · escaping".into()
        };
        orb.resonance = "—".into();
    }

    let period = std::f64::consts::TAU
        * (a.abs() * a.abs() * a.abs() / mu).sqrt();
    if !bound {
        let per = 6.0 * std::f64::consts::PI * mu
            / (C_LIGHT * C_LIGHT * a.abs() * (e * e - 1.0).max(1e-6));
        info.pred_arcsec = per.to_degrees() * 3600.0;
        info.pred_century = 0.0;
        return;
    }

    let mut best = (0.06_f64, String::new());
    for (oe, og, on, _) in &bodies {
        if oe == target || og.mass < 1.0e18 {
            continue;
        }
        let rj = og.pos - ppos;
        let vj = og.vel - pvel;
        let rjm = rj.length();
        if rjm < 1.0 || rjm > 40.0 * rm.max(1.0) {
            continue;
        }
        let aj = 1.0 / (2.0 / rjm - vj.dot(vj) / mu);
        if aj <= 0.0 {
            continue;
        }
        let tj = std::f64::consts::TAU * (aj * aj * aj / mu).sqrt();
        if tj <= 0.0 {
            continue;
        }
        let ratio = if period >= tj {
            period / tj
        } else {
            tj / period
        };
        for p in 2..=6 {
            for q in 1..p {
                if gcd(p as u32, q as u32) != 1 {
                    continue;
                }
                let rel = (ratio - p as f64 / q as f64).abs()
                    / (p as f64 / q as f64);
                if rel < best.0 {
                    best = (
                        rel,
                        format!(
                            "{p}:{q} with {}",
                            on.as_str()
                        ),
                    );
                }
            }
        }
    }
    orb.resonance = if best.1.is_empty() {
        "none".into()
    } else {
        best.1
    };

    let per = 6.0 * std::f64::consts::PI * mu
        / (C_LIGHT * C_LIGHT * a * (1.0 - e * e));
    info.pred_arcsec = per.to_degrees() * 3600.0;
    let orbits_century =
        (100.0 * 365.25 * 86400.0) / period.max(1.0);
    info.pred_century = info.pred_arcsec * orbits_century;
}

#[derive(Resource, Default)]
pub struct ConvertReq(pub Option<BodyClass>);

#[derive(Resource, Default)]
pub struct SupernovaReq(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct KilonovaReq(pub Vec<DVec3>);

#[derive(Clone, Copy, PartialEq)]
pub enum Chaos {
    Shatter,
    RogueBh,
    Barrage,
    Kick,
    Vaporize,
    Clone,
    GiantImpact,
    StarFall,
    CometSwarm,
    ScatterAll,
    Freeze,
    Reverse,
}

#[derive(Resource, Default)]
pub struct ChaosReq(pub Option<Chaos>);

#[derive(Resource, Default)]
pub struct Boom {
    pub amt: f32,
    pub col: [f32; 3],
}

fn is_neutron_like(mass: f64, radius: f64) -> bool {
    (1.2e30..=3.2e30).contains(&mass) && radius < 5.0e4
}

fn is_compact(mass: f64, radius: f64) -> bool {
    mass > 1.0e29 && radius > 0.0 && radius < 5.0e5
}

#[derive(Component, Clone)]
pub struct GwPair {
    pub other: Entity,
    pub a: f64,
    pub phase: f64,
    pub e1: DVec3,
    pub e2: DVec3,
    pub com: DVec3,
    pub vcom: DVec3,
}

#[derive(Component)]
pub struct GwBound;

const GW_CAPTURE: f64 = 6.0e8;

fn gw_inspiral(
    paused: Res<Paused>,
    time: Res<Time>,
    speed: Res<SimSpeed>,
    mut commands: Commands,
    grid: Query<&Grid>,
    mut q: Query<(
        Entity,
        &mut Grav,
        &Shape,
        &mut CellCoord,
        &mut Transform,
        Option<&GwPair>,
    )>,
) {
    if paused.0 {
        return;
    }
    let Ok(grid) = grid.single() else {
        return;
    };
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }

    let snap: Vec<(Entity, f64, DVec3, DVec3, f64, Option<GwPair>)> = q
        .iter()
        .map(|(e, g, s, _, _, gp)| {
            (e, g.mass, g.pos, g.vel, s.base_radius, gp.cloned())
        })
        .collect();
    let mut bound: std::collections::HashSet<Entity> =
        std::collections::HashSet::new();
    for (e, _, _, _, _, gp) in &snap {
        if let Some(p) = gp {
            bound.insert(*e);
            bound.insert(p.other);
        }
    }

    let comp: Vec<(Entity, f64, DVec3, DVec3)> = snap
        .iter()
        .filter(|(e, m, _, _, r, _)| {
            is_compact(*m, *r) && !bound.contains(e)
        })
        .map(|(e, m, p, v, _, _)| (*e, *m, *p, *v))
        .collect();
    let n = comp.len();
    let mut writes: Vec<(Entity, DVec3, DVec3)> = Vec::new();
    for i in 0..n {
        let mut best = (f64::MAX, usize::MAX);
        for j in 0..n {
            if i != j {
                let d = (comp[i].2 - comp[j].2).length();
                if d < best.0 {
                    best = (d, j);
                }
            }
        }
        let j = best.1;
        if j == usize::MAX || j < i || best.0 > GW_CAPTURE {
            continue;
        }
        let mut bk = (f64::MAX, usize::MAX);
        for k in 0..n {
            if k != j {
                let d = (comp[j].2 - comp[k].2).length();
                if d < bk.0 {
                    bk = (d, k);
                }
            }
        }
        if bk.1 != i {
            continue;
        }
        let (ei, m1, p1, v1) = comp[i];
        let (ej, m2, p2, v2) = comp[j];
        let mtot = m1 + m2;
        let rvec = p1 - p2;
        let a = rvec.length();
        if a <= 0.0 {
            continue;
        }
        let vrel = v1 - v2;
        let nrm = rvec.cross(vrel).normalize_or_zero();
        let nrm = if nrm.length() < 0.5 { DVec3::Y } else { nrm };
        let e1 = (rvec / a) - nrm * (rvec / a).dot(nrm);
        let e1 = e1.normalize_or_zero();
        let e1 = if e1.length() < 0.5 { DVec3::X } else { e1 };
        let e2 = nrm.cross(e1).normalize_or_zero();
        commands.entity(ei).insert((
            GwPair {
                other: ej,
                a,
                phase: 0.0,
                e1,
                e2,
                com: (p1 * m1 + p2 * m2) / mtot,
                vcom: (v1 * m1 + v2 * m2) / mtot,
            },
            GwBound,
        ));
        commands.entity(ej).insert(GwBound);
    }

    let masses: std::collections::HashMap<Entity, (f64, f64)> = snap
        .iter()
        .map(|(e, m, _, _, r, _)| (*e, (*m, *r)))
        .collect();
    for (ei, _, _, _, _, gp) in &snap {
        let Some(gp) = gp else { continue };
        let (Some(&(m1, r1)), Some(&(m2, r2))) =
            (masses.get(ei), masses.get(&gp.other))
        else {
            commands.entity(*ei).remove::<GwPair>();
            continue;
        };
        let mtot = m1 + m2;
        let touch = (r1 + r2).max(1.0);
        let da = -(64.0 / 5.0) * G.powi(3) * m1 * m2 * mtot
            / (C_LIGHT.powi(5) * gp.a.powi(3))
            * sim_dt;
        let a = (gp.a + da).max(touch * 0.5);
        let omega = (G * mtot / a.powi(3)).sqrt();
        let phase = gp.phase + omega * sim_dt;
        let com = gp.com + gp.vcom * sim_dt;
        let dir = gp.e1 * phase.cos() + gp.e2 * phase.sin();
        let tang = -gp.e1 * phase.sin() + gp.e2 * phase.cos();
        let vc = (G * mtot / a).sqrt();
        let p1 = com + dir * (a * m2 / mtot);
        let p2 = com - dir * (a * m1 / mtot);
        let v1 = gp.vcom + tang * (vc * m2 / mtot);
        let v2 = gp.vcom - tang * (vc * m1 / mtot);
        if a <= touch {
            commands.entity(*ei).remove::<GwPair>().remove::<GwBound>();
            commands.entity(gp.other).remove::<GwBound>();
        } else {
            commands.entity(*ei).insert(GwPair {
                other: gp.other,
                a,
                phase,
                e1: gp.e1,
                e2: gp.e2,
                com,
                vcom: gp.vcom,
            });
        }
        writes.push((*ei, p1, v1));
        writes.push((gp.other, p2, v2));
    }

    for (e, np, nv) in writes {
        if let Ok((_, mut g, _, mut cell, mut t, _)) = q.get_mut(e) {
            g.pos = np;
            g.vel = nv;
            let (c, off) = grid.translation_to_grid(np);
            *cell = c;
            t.translation = off;
        }
    }
}


#[derive(Component)]
pub struct Flash {
    pub age: f32,
    pub life: f32,
    pub expand: f32,
    pub c0: LinearRgba,
    pub c1: LinearRgba,
}

#[derive(Component)]
pub struct Magnetar {
    pub clock: f32,
    pub next: f32,
    pub flare: f32,
    pub period: f32,
}

impl Default for Magnetar {
    fn default() -> Self {
        Self {
            clock: 0.0,
            next: 4.0,
            flare: 0.0,
            period: 7.56,
        }
    }
}

fn magnetar_flare(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&mut Magnetar, &MeshMaterial3d<StandardMaterial>)>,
) {
    let dt = time.delta_secs();
    for (mut m, mm) in &mut q {
        m.clock += dt;
        if m.flare <= 0.0 {
            if m.clock >= m.next {
                m.flare = 1.0;
                m.next = m.clock + 7.0 + (m.clock * 1.37).sin().abs() * 9.0;
            }
        } else {
            m.flare = (m.flare - dt / 2.4).max(0.0);
        }
        let Some(mat) = materials.get_mut(&mm.0) else {
            continue;
        };
        let spin = 0.55 + 0.45 * (m.clock * std::f32::consts::TAU
            / m.period)
            .sin();
        let base = 360.0;
        let burst = m.flare * m.flare * 90.0 * spin;
        let i = base + burst * base;
        mat.emissive = LinearRgba::rgb(0.62 * i, 0.74 * i, 1.0 * i);
    }
}

#[derive(Resource, Default)]
pub struct ActivePreset(pub usize);

#[derive(Resource, Default)]
pub struct ReloadReq(pub bool);

#[derive(Resource, Default)]
pub struct SaveReq(pub bool);

#[derive(Resource, Default)]
pub struct LoadReq(pub bool);

#[derive(Resource, Default)]
pub struct PendingLoad(pub Option<Vec<SpawnSpec>>);

#[derive(Resource, Default)]
pub struct CamWorld {
    pub pos: DVec3,
    pub fwd: DVec3,
    pub right: DVec3,
    pub up: DVec3,
}

#[derive(Resource, PartialEq, Eq, Clone, Copy, Default)]
pub enum CamMode {
    #[default]
    Follow,
    Fly,
}

#[derive(Resource, Default)]
pub struct FlyState {
    pub pos: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f64,
}

pub struct Preset {
    pub name: &'static str,
    pub star_name: &'static str,
    pub star_radius: f64,
    pub star_mass: f64,
    pub star_emissive: LinearRgba,
    pub bodies: &'static [PlanetDef],
    pub with_moon: bool,
    pub proc: bool,
    pub spiral: bool,
    pub galaxy: u8,
    pub scenario: u8,
}

pub const PRESETS: [Preset; 19] = [
    Preset {
        name: "Solar System",
        star_name: "Sun",
        star_radius: 695_508_000.0,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &PLANETS,
        with_moon: true,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 0,
    },
    Preset {
        name: "TRAPPIST-1",
        star_name: "TRAPPIST-1",
        star_radius: 8.30e7,
        star_mass: 1.786e29,
        star_emissive: LinearRgba::rgb(120.0, 36.0, 14.0),
        bodies: &TRAPPIST,
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 0,
    },
    Preset {
        name: "Star Cluster",
        star_name: "Core",
        star_radius: 1.0e9,
        star_mass: 4.0e35,
        star_emissive: LinearRgba::rgb(40.0, 30.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 0,
        scenario: 0,
    },
    Preset {
        name: "Milky Way Sector",
        star_name: "Sgr A*",
        star_radius: 2.2e10,
        star_mass: 8.0e36,
        star_emissive: LinearRgba::rgb(70.0, 40.0, 110.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: true,
        galaxy: 1,
        scenario: 0,
    },
    Preset {
        name: "Barred Spiral",
        star_name: "Galactic Core",
        star_radius: 2.2e10,
        star_mass: 8.0e36,
        star_emissive: LinearRgba::rgb(80.0, 45.0, 120.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: true,
        galaxy: 2,
        scenario: 0,
    },
    Preset {
        name: "Elliptical Galaxy",
        star_name: "cD Core",
        star_radius: 3.0e10,
        star_mass: 2.0e37,
        star_emissive: LinearRgba::rgb(60.0, 35.0, 90.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 3,
        scenario: 0,
    },
    Preset {
        name: "Lenticular Galaxy",
        star_name: "S0 Core",
        star_radius: 2.5e10,
        star_mass: 1.0e37,
        star_emissive: LinearRgba::rgb(70.0, 45.0, 95.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 4,
        scenario: 0,
    },
    Preset {
        name: "Irregular Galaxy",
        star_name: "Irr Core",
        star_radius: 1.2e10,
        star_mass: 3.0e36,
        star_emissive: LinearRgba::rgb(45.0, 55.0, 110.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 5,
        scenario: 0,
    },
    Preset {
        name: "Ring Galaxy",
        star_name: "Ring Core",
        star_radius: 2.0e10,
        star_mass: 6.0e36,
        star_emissive: LinearRgba::rgb(70.0, 40.0, 110.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 6,
        scenario: 0,
    },
    Preset {
        name: "Dwarf Galaxy",
        star_name: "Dwarf Core",
        star_radius: 6.0e9,
        star_mass: 5.0e35,
        star_emissive: LinearRgba::rgb(50.0, 40.0, 80.0),
        bodies: &[],
        with_moon: false,
        proc: true,
        spiral: false,
        galaxy: 7,
        scenario: 0,
    },
    Preset {
        name: "★ Rogue Star Flyby",
        star_name: "Sun",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 1,
    },
    Preset {
        name: "★ Death of the Sun",
        star_name: "Dying Sun",
        star_radius: 3.0e10,
        star_mass: 1.4e30,
        star_emissive: LinearRgba::rgb(120.0, 36.0, 14.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 2,
    },
    Preset {
        name: "★ Twin Suns (Tatooine)",
        star_name: "Sun A",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 3,
    },
    Preset {
        name: "★ Black Hole Feast",
        star_name: "Black hole",
        star_radius: 1.8e4,
        star_mass: 1.2e31,
        star_emissive: LinearRgba::BLACK,
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 4,
    },
    Preset {
        name: "★ Planet Pinball",
        star_name: "Sun",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 5,
    },
    Preset {
        name: "★ Earth–Moon Smash",
        star_name: "Sun",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 6,
    },
    Preset {
        name: "★ Hypervelocity Star",
        star_name: "Sun",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 7,
    },
    Preset {
        name: "★ Grand Tack",
        star_name: "Sun",
        star_radius: 6.96e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 8,
    },
    Preset {
        name: "★ Triple-Star Chaos",
        star_name: "Star 1",
        star_radius: 6.0e8,
        star_mass: 1.98892e30,
        star_emissive: LinearRgba::rgb(180.0, 130.0, 60.0),
        bodies: &[],
        with_moon: false,
        proc: false,
        spiral: false,
        galaxy: 0,
        scenario: 9,
    },
];

const TRAPPIST: [PlanetDef; 7] = [
    PlanetDef { name: "b", orbit: 1.730e9, radius: 7.06e6, mass: 8.20e24, kind: PlanetKind::Rocky { r: 0.62, g: 0.34, b: 0.24 }, atmo: None },
    PlanetDef { name: "c", orbit: 2.370e9, radius: 6.90e6, mass: 7.80e24, kind: PlanetKind::Rocky { r: 0.66, g: 0.42, b: 0.30 }, atmo: None },
    PlanetDef { name: "d", orbit: 3.340e9, radius: 4.90e6, mass: 2.50e24, kind: PlanetKind::Terran, atmo: Some(palettes::css::SKY_BLUE) },
    PlanetDef { name: "e", orbit: 4.390e9, radius: 5.80e6, mass: 4.10e24, kind: PlanetKind::Terran, atmo: Some(palettes::css::SKY_BLUE) },
    PlanetDef { name: "f", orbit: 5.780e9, radius: 6.70e6, mass: 6.30e24, kind: PlanetKind::Terran, atmo: Some(palettes::css::CORNFLOWER_BLUE) },
    PlanetDef { name: "g", orbit: 7.030e9, radius: 7.30e6, mass: 8.10e24, kind: PlanetKind::IceGiant { deep: false }, atmo: Some(palettes::css::LIGHT_CYAN) },
    PlanetDef { name: "h", orbit: 9.290e9, radius: 4.90e6, mass: 1.90e24, kind: PlanetKind::IceGiant { deep: true }, atmo: None },
];

pub struct SolarPlugin;

impl Plugin for SolarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimSpeed>()
            .init_resource::<Selected>()
            .init_resource::<SpawnRequest>()
            .init_resource::<OrbitView>()
            .init_resource::<CineCam>()
            .init_resource::<Paused>()
            .init_resource::<SimClock>()
            .init_resource::<ShowGrid>()
            .init_resource::<Relativity>()
            .init_resource::<PrecessionInfo>()
            .init_resource::<OrbitInfo>()
            .init_resource::<MassXfer>()
            .init_resource::<NewBodyType>()
            .init_resource::<ConvertReq>()
            .init_resource::<SupernovaReq>()
            .init_resource::<KilonovaReq>()
            .init_resource::<ChaosReq>()
            .init_resource::<Boom>()
            .init_resource::<ActivePreset>()
            .init_resource::<ReloadReq>()
            .init_resource::<SaveReq>()
            .init_resource::<LoadReq>()
            .init_resource::<PendingLoad>()
            .init_resource::<SizeExaggeration>()
            .init_resource::<CamWorld>()
            .init_resource::<CamMode>()
            .init_resource::<FlyState>()
            .add_systems(OnEnter(AppMode::SolarSystem), spawn_solar_system)
            .add_systems(OnExit(AppMode::SolarSystem), teardown_solar)
            .add_systems(
                Update,
                (
                    (
                        reload_world,
                        solar_mouse_input,
                        nbody,
                        gw_inspiral,
                        collisions,
                        tidal_disruption,
                        debris_accretion,
                        mass_transfer,
                        record_trails,
                        precession,
                        sun_light,
                    )
                        .chain(),
                    (
                        cine_cam,
                        follow_cam,
                        fly_cam,
                        draw_trails,
                        follow_sky,
                        pick_body,
                        draw_selection,
                        draw_grid,
                    )
                        .chain(),
                    (
                        apply_shape,
                        rotate_bodies,
                        handle_spawn,
                        chaos,
                        drag_create,
                        convert_selected,
                        sync_jet_rigs,
                        sync_comet_rigs,
                        sync_mt_rig,
                        planet_climate,
                        tidal_locking,
                    )
                        .chain(),
                    (
                        atmosphere_escape,
                        stellar_evolution,
                        supernova,
                        kilonova,
                        flash_fx,
                        magnetar_flare,
                        animate_jets,
                        animate_disks,
                        animate_nebulae,
                        aurora_drive,
                        comet_sublimation,
                        animate_comets,
                        save_scene,
                        load_scene,
                    )
                        .chain(),
                )
                    .chain()
                    .run_if(in_state(AppMode::SolarSystem)),
            );
    }
}

fn teardown_solar(mut commands: Commands, q: Query<Entity, With<SolarScene>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

pub struct PlanetDef {
    name: &'static str,
    orbit: f64,
    radius: f64,
    mass: f64,
    kind: PlanetKind,
    atmo: Option<Srgba>,
}

const PLANETS: [PlanetDef; 8] = [
    PlanetDef { name: "Mercury", orbit: 5.79e10, radius: 2.4397e6, mass: 3.301e23, kind: PlanetKind::Rocky { r: 0.55, g: 0.52, b: 0.50 }, atmo: None },
    PlanetDef { name: "Venus", orbit: 1.082e11, radius: 6.0518e6, mass: 4.867e24, kind: PlanetKind::Rocky { r: 0.86, g: 0.74, b: 0.46 }, atmo: Some(palettes::css::LIGHT_YELLOW) },
    PlanetDef { name: "Earth", orbit: 1.496e11, radius: 6.371e6, mass: 5.972e24, kind: PlanetKind::Terran, atmo: Some(palettes::css::SKY_BLUE) },
    PlanetDef { name: "Mars", orbit: 2.279e11, radius: 3.3895e6, mass: 6.417e23, kind: PlanetKind::Rocky { r: 0.66, g: 0.34, b: 0.22 }, atmo: Some(palettes::css::SALMON) },
    PlanetDef { name: "Jupiter", orbit: 7.785e11, radius: 6.9911e7, mass: 1.898e27, kind: PlanetKind::GasBands { warm: true }, atmo: None },
    PlanetDef { name: "Saturn", orbit: 1.434e12, radius: 5.8232e7, mass: 5.683e26, kind: PlanetKind::GasBands { warm: false }, atmo: None },
    PlanetDef { name: "Uranus", orbit: 2.871e12, radius: 2.5362e7, mass: 8.681e25, kind: PlanetKind::IceGiant { deep: false }, atmo: Some(palettes::css::LIGHT_CYAN) },
    PlanetDef { name: "Neptune", orbit: 4.495e12, radius: 2.4622e7, mass: 1.024e26, kind: PlanetKind::IceGiant { deep: true }, atmo: Some(palettes::css::CORNFLOWER_BLUE) },
];

fn spawn_solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skies: ResMut<Assets<StarSky>>,
    mut nebs: ResMut<Assets<NebulaMat>>,
    mut auroras: ResMut<Assets<AuroraMat>>,
    mut images: ResMut<Assets<Image>>,
    mut selected: ResMut<Selected>,
    active: Res<ActivePreset>,
) {
    let p = &PRESETS[active.0.min(PRESETS.len() - 1)];
    let specs = if p.scenario > 0 {
        specs_scenario(p)
    } else if p.proc {
        if p.galaxy > 0 { specs_galaxy(p, 0xC0FFEE) } else { specs_procedural(p, 0xC0FFEE) }
    } else {
        specs_from_preset(p)
    };
    build_from_specs(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut skies,
        &mut nebs,
        &mut auroras,
        &mut images,
        &mut selected,
        &specs,
    );
}

fn reload_world(
    mut req: ResMut<ReloadReq>,
    mut pending: ResMut<PendingLoad>,
    mut commands: Commands,
    existing: Query<Entity, With<SolarScene>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skies: ResMut<Assets<StarSky>>,
    mut nebs: ResMut<Assets<NebulaMat>>,
    mut auroras: ResMut<Assets<AuroraMat>>,
    mut images: ResMut<Assets<Image>>,
    mut selected: ResMut<Selected>,
    active: Res<ActivePreset>,
) {
    let loaded = pending.0.take();
    if !req.0 && loaded.is_none() {
        return;
    }
    req.0 = false;
    for e in &existing {
        commands.entity(e).despawn();
    }
    let specs = loaded.unwrap_or_else(|| {
        let p = &PRESETS[active.0.min(PRESETS.len() - 1)];
        if p.scenario > 0 {
            specs_scenario(p)
        } else if p.proc {
            if p.galaxy > 0 { specs_galaxy(p, 0xC0FFEE) } else { specs_procedural(p, 0xC0FFEE) }
        } else {
            specs_from_preset(p)
        }
    });
    build_from_specs(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut skies,
        &mut nebs,
        &mut auroras,
        &mut images,
        &mut selected,
        &specs,
    );
}

#[derive(Component, Clone, Copy)]
pub struct Visual {
    pub kind: PlanetKind,
    pub star: bool,
    pub atmo: Option<Srgba>,
    pub rings: bool,
    pub emissive: LinearRgba,
}

pub struct SpawnSpec {
    pub name: String,
    pub kind: PlanetKind,
    pub star: bool,
    pub emissive: LinearRgba,
    pub radius: f64,
    pub mass: f64,
    pub pos: DVec3,
    pub vel: DVec3,
    pub atmo: Option<Srgba>,
    pub rings: bool,
}

fn obliquity_rad(name: &str) -> f32 {
    let deg = match name {
        "Mercury" => 0.03,
        "Venus" => 177.4,
        "Earth" => 23.45,
        "Mars" => 23.98,
        "Jupiter" => 3.08,
        "Saturn" => 26.73,
        "Uranus" => 97.92,
        "Neptune" => 29.6,
        "Moon" => 6.68,
        _ => {
            let mut h: u64 = 1469598103934665603;
            for b in name.bytes() {
                h = (h ^ b as u64).wrapping_mul(1099511628211);
            }
            (h % 4500) as f64 / 100.0
        }
    };
    (deg as f32).to_radians()
}

fn specs_from_preset(preset: &Preset) -> Vec<SpawnSpec> {
    let mut v = vec![SpawnSpec {
        name: preset.star_name.into(),
        kind: PlanetKind::Terran,
        star: true,
        emissive: preset.star_emissive,
        radius: preset.star_radius,
        mass: preset.star_mass,
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        atmo: None,
        rings: false,
    }];
    for p in preset.bodies {
        let pos = DVec3::new(p.orbit, 0.0, 0.0);
        let vel = DVec3::new(0.0, 0.0, (G * preset.star_mass / p.orbit).sqrt());
        v.push(SpawnSpec {
            name: p.name.into(),
            kind: p.kind,
            star: false,
            emissive: LinearRgba::BLACK,
            radius: p.radius,
            mass: p.mass,
            pos,
            vel,
            atmo: p.atmo,
            rings: matches!(
                p.name,
                "Saturn" | "Jupiter" | "Uranus" | "Neptune"
            ),
        });
        if preset.with_moon && p.name == "Earth" {
            v.push(SpawnSpec {
                name: "Moon".into(),
                kind: PlanetKind::Rocky { r: 0.55, g: 0.54, b: 0.52 },
                star: false,
                emissive: LinearRgba::BLACK,
                radius: MOON_RADIUS_M,
                mass: MOON_MASS,
                pos: pos + DVec3::new(MOON_ORBIT_M, 0.0, 0.0),
                vel: vel + DVec3::new(0.0, 0.0, (G * p.mass / MOON_ORBIT_M).sqrt()),
                atmo: None,
                rings: false,
            });
        }
    }
    v
}

fn specs_scenario(preset: &Preset) -> Vec<SpawnSpec> {
    let sun = SUN_MASS;
    let mk = |name: &str,
              kind: PlanetKind,
              star: bool,
              emissive: LinearRgba,
              radius: f64,
              mass: f64,
              pos: DVec3,
              vel: DVec3|
     -> SpawnSpec {
        SpawnSpec {
            name: name.into(),
            kind,
            star,
            emissive,
            radius,
            mass,
            pos,
            vel,
            atmo: None,
            rings: false,
        }
    };
    let solar_planets = |v: &mut Vec<SpawnSpec>, m_star: f64| {
        for p in PLANETS {
            let pos = DVec3::new(p.orbit, 0.0, 0.0);
            let vel = DVec3::new(
                0.0,
                0.0,
                (G * m_star / p.orbit).sqrt(),
            );
            v.push(SpawnSpec {
                name: p.name.into(),
                kind: p.kind,
                star: false,
                emissive: LinearRgba::BLACK,
                radius: p.radius,
                mass: p.mass,
                pos,
                vel,
                atmo: p.atmo,
                rings: matches!(
                    p.name,
                    "Saturn" | "Jupiter" | "Uranus" | "Neptune"
                ),
            });
        }
    };
    let sun_em = BodyClass::SunLike.emissive();
    let mut rng = Lcg(0x5CE7A12 ^ preset.scenario as u64);

    match preset.scenario {
        1 => {
            let mut v = vec![mk(
                "Sun",
                PlanetKind::Terran,
                true,
                sun_em,
                6.96e8,
                sun,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            solar_planets(&mut v, sun);
            let p = DVec3::new(9.0e12, 6.0e11, 3.0e12);
            let aim =
                DVec3::new(2.5e11, 0.0, 0.0) - p;
            v.push(mk(
                "Rogue star",
                PlanetKind::Terran,
                true,
                sun_em,
                6.0e8,
                1.2 * sun,
                p,
                aim.normalize_or_zero() * 3.0e4,
            ));
            v
        }
        2 => {
            let mut v = vec![mk(
                "Dying Sun",
                PlanetKind::Terran,
                true,
                BodyClass::RedGiant.emissive(),
                3.0e10,
                1.4e30,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            solar_planets(&mut v, 1.4e30);
            v
        }
        3 => {
            let sep = 3.2e10;
            let mtot = 2.0 * sun;
            let vrel = (G * mtot / sep).sqrt();
            let mut v = vec![
                mk(
                    "Sun A",
                    PlanetKind::Terran,
                    true,
                    sun_em,
                    6.96e8,
                    sun,
                    DVec3::new(-sep * 0.5, 0.0, 0.0),
                    DVec3::new(0.0, 0.0, -vrel * 0.5),
                ),
                mk(
                    "Sun B",
                    PlanetKind::Terran,
                    true,
                    BodyClass::RedDwarf.emissive(),
                    5.0e8,
                    sun,
                    DVec3::new(sep * 0.5, 0.0, 0.0),
                    DVec3::new(0.0, 0.0, vrel * 0.5),
                ),
            ];
            for (i, a) in
                [4.0e11_f64, 9.0e11, 1.8e12].iter().enumerate()
            {
                let vc = (G * mtot / a).sqrt();
                v.push(mk(
                    &format!("Tatooine {}", i + 1),
                    PlanetKind::Desert,
                    false,
                    LinearRgba::BLACK,
                    6.0e6,
                    5.0e24,
                    DVec3::new(*a, 0.0, 0.0),
                    DVec3::new(0.0, 0.0, vc),
                ));
            }
            v
        }
        4 => {
            let bhm = 1.2e31;
            let mut v = vec![mk(
                "Black hole",
                PlanetKind::Rocky { r: 0.01, g: 0.01, b: 0.02 },
                false,
                LinearRgba::BLACK,
                1.8e4,
                bhm,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            for i in 0..6 {
                let a = 3.0e11 + 2.0e11 * i as f64;
                let vc = (G * bhm / a).sqrt()
                    * (0.55 + 0.15 * rng.u());
                let th = rng.u() * std::f64::consts::TAU;
                let pos = DVec3::new(
                    a * th.cos(),
                    (rng.u() - 0.5) * 4.0e10,
                    a * th.sin(),
                );
                let tang =
                    DVec3::new(-th.sin(), 0.0, th.cos());
                v.push(mk(
                    &format!("Doomed star {}", i + 1),
                    PlanetKind::Terran,
                    true,
                    if i % 2 == 0 {
                        sun_em
                    } else {
                        BodyClass::RedGiant.emissive()
                    },
                    6.0e8,
                    sun * (0.6 + rng.u()),
                    pos,
                    tang * vc,
                ));
            }
            v
        }
        6 => {
            let mut v = vec![mk(
                "Sun",
                PlanetKind::Terran,
                true,
                sun_em,
                6.96e8,
                sun,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            let ep = DVec3::new(AU_M, 0.0, 0.0);
            let ev = DVec3::new(
                0.0,
                0.0,
                (G * sun / AU_M).sqrt(),
            );
            v.push(mk(
                "Earth",
                PlanetKind::Terran,
                false,
                LinearRgba::BLACK,
                6.37e6,
                5.97e24,
                ep,
                ev,
            ));
            let tp = ep
                + DVec3::new(0.0, 8.0e8, -6.0e8);
            v.push(mk(
                "Theia",
                PlanetKind::Rocky { r: 0.6, g: 0.4, b: 0.32 },
                false,
                LinearRgba::BLACK,
                3.4e6,
                6.4e23,
                tp,
                ev + (ep - tp).normalize_or_zero() * 1.2e4,
            ));
            v
        }
        7 => {
            let mut v = vec![mk(
                "Sun",
                PlanetKind::Terran,
                true,
                sun_em,
                6.96e8,
                sun,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            solar_planets(&mut v, sun);
            v.push(mk(
                "Hypervelocity star",
                PlanetKind::Terran,
                true,
                BodyClass::OStar.emissive(),
                7.0e9,
                8.0e31,
                DVec3::new(-1.2e13, 2.0e11, -4.0e12),
                DVec3::new(1.0e6, 0.0, 3.0e5),
            ));
            v
        }
        8 => {
            let mut v = vec![mk(
                "Sun",
                PlanetKind::Terran,
                true,
                sun_em,
                6.96e8,
                sun,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            for (i, a) in [
                0.39_f64, 0.72, 1.0, 1.52,
            ]
            .iter()
            .enumerate()
            {
                let r = a * AU_M;
                v.push(mk(
                    &format!("Inner {}", i + 1),
                    PlanetKind::Rocky { r: 0.6, g: 0.5, b: 0.42 },
                    false,
                    LinearRgba::BLACK,
                    5.0e6,
                    4.0e24,
                    DVec3::new(r, 0.0, 0.0),
                    DVec3::new(
                        0.0,
                        0.0,
                        (G * sun / r).sqrt(),
                    ),
                ));
            }
            let jr = 1.5 * AU_M;
            v.push(mk(
                "Migrating Jupiter",
                PlanetKind::GasBands { warm: true },
                false,
                LinearRgba::BLACK,
                6.99e7,
                1.898e27,
                DVec3::new(jr, 0.0, 0.0),
                DVec3::new(
                    0.0,
                    0.0,
                    (G * sun / jr).sqrt() * 0.86,
                ),
            ));
            v
        }
        9 => {
            let sep = 5.0e10;
            let mtot = 3.0 * sun;
            let vc = (G * mtot / sep).sqrt() * 0.6;
            let tri = |k: usize| {
                let a = k as f64
                    * std::f64::consts::TAU
                    / 3.0;
                DVec3::new(a.cos(), 0.0, a.sin()) * sep
            };
            let mut v = Vec::new();
            for k in 0..3 {
                let p = tri(k);
                let t = DVec3::new(-p.z, 0.0, p.x)
                    .normalize_or_zero();
                v.push(mk(
                    &format!("Star {}", k + 1),
                    PlanetKind::Terran,
                    true,
                    if k == 0 {
                        sun_em
                    } else if k == 1 {
                        BodyClass::RedDwarf.emissive()
                    } else {
                        BodyClass::OStar.emissive()
                    },
                    6.0e8,
                    sun,
                    p,
                    t * vc,
                ));
            }
            for i in 0..4 {
                let r = (3.0 + 1.5 * i as f64) * AU_M;
                v.push(mk(
                    &format!("Planet {}", i + 1),
                    PlanetKind::Rocky { r: 0.6, g: 0.5, b: 0.4 },
                    false,
                    LinearRgba::BLACK,
                    6.0e6,
                    5.0e24,
                    DVec3::new(r, 0.0, 0.0),
                    DVec3::new(
                        0.0,
                        0.0,
                        (G * mtot / r).sqrt(),
                    ),
                ));
            }
            v
        }
        _ => {
            let mut v = vec![mk(
                "Sun",
                PlanetKind::Terran,
                true,
                sun_em,
                6.96e8,
                sun,
                DVec3::ZERO,
                DVec3::ZERO,
            )];
            for i in 0..14 {
                let a = (0.5 + 0.55 * i as f64) * AU_M;
                let e = 0.30 + 0.35 * rng.u();
                let rp = a * (1.0 - e);
                let vp =
                    (G * sun * (1.0 + e) / (a * (1.0 - e)))
                        .sqrt();
                let th = rng.u() * std::f64::consts::TAU;
                let inc = (rng.u() - 0.5) * 0.7;
                let pos = DVec3::new(
                    rp * th.cos(),
                    rp * inc,
                    rp * th.sin(),
                );
                let tang =
                    DVec3::new(-th.sin(), 0.0, th.cos());
                v.push(mk(
                    &format!("Pinball {}", i + 1),
                    if i % 3 == 0 {
                        PlanetKind::Lava
                    } else {
                        PlanetKind::Rocky {
                            r: 0.6,
                            g: 0.45,
                            b: 0.4,
                        }
                    },
                    false,
                    LinearRgba::BLACK,
                    5.0e6,
                    3.0e24,
                    pos,
                    tang * vp,
                ));
            }
            v
        }
    }
}

struct Lcg(u64);
impl Lcg {
    fn u(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f64) / (u32::MAX as f64 + 1.0)
    }
}

fn specs_procedural(preset: &Preset, seed: u64) -> Vec<SpawnSpec> {
    let mut rng = Lcg(seed ^ 0x9E3779B97F4A7C15);
    let core_m = preset.star_mass;
    let mut v = vec![SpawnSpec {
        name: preset.star_name.into(),
        kind: PlanetKind::Terran,
        star: true,
        emissive: preset.star_emissive,
        radius: preset.star_radius,
        mass: core_m,
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        atmo: None,
        rings: false,
    }];

    let n = 110usize;
    let rmax = 4.0e12;
    for i in 0..n {
        let u0 = rng.u();
        let r = rmax * (0.02 + 0.98 * u0.sqrt());
        let th = rng.u() * std::f64::consts::TAU;
        let y = (rng.u() - 0.5) * rmax * 0.05;
        let pos = DVec3::new(r * th.cos(), y, r * th.sin());

        let pick = rng.u();
        let class = if pick < 0.60 {
            BodyClass::RedDwarf
        } else if pick < 0.84 {
            BodyClass::SunLike
        } else if pick < 0.92 {
            BodyClass::RedGiant
        } else if pick < 0.97 {
            BodyClass::WhiteDwarf
        } else {
            BodyClass::NeutronStar
        };
        let (mass, radius, _l, _t, kind) = class.props();

        let vc = (G * core_m / r).sqrt() * (0.85 + 0.3 * rng.u());
        let tang = DVec3::new(-th.sin(), 0.0, th.cos());
        let vel = tang * vc
            + DVec3::new(0.0, (rng.u() - 0.5) * vc * 0.04, 0.0);

        v.push(SpawnSpec {
            name: format!("Star {i}"),
            kind,
            star: true,
            emissive: class.emissive(),
            radius,
            mass,
            pos,
            vel,
            atmo: None,
            rings: false,
        });
    }
    v
}

fn specs_galaxy(preset: &Preset, seed: u64) -> Vec<SpawnSpec> {
    let mut rng = Lcg(seed ^ 0xD1B54A32D192ED03);
    let core_m = preset.star_mass;
    let mut v = vec![SpawnSpec {
        name: preset.star_name.into(),
        kind: PlanetKind::Terran,
        star: true,
        emissive: preset.star_emissive,
        radius: preset.star_radius,
        mass: core_m,
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        atmo: None,
        rings: false,
    }];

    let gid = preset.galaxy;
    let n = if gid == 7 { 90usize } else { 260usize };
    let pitch = 0.22_f64;
    let h_scale = 1.7e12;
    let rmax = 9.0e12;
    let tau = std::f64::consts::TAU;
    let clumps: Vec<DVec3> = {
        let mut c = Lcg(seed ^ 0xC1A3);
        (0..5)
            .map(|_| {
                DVec3::new(
                    (c.u() - 0.5) * 1.4 * rmax,
                    (c.u() - 0.5) * 0.3 * rmax,
                    (c.u() - 0.5) * 1.4 * rmax,
                )
            })
            .collect()
    };
    for i in 0..n {
        let u0 = rng.u();
        let g1 = rng.u() + rng.u() + rng.u() - 1.5;
        let g2 = rng.u() + rng.u() + rng.u() - 1.5;
        let g3 = rng.u() + rng.u() + rng.u() - 1.5;
        let (pos, vel) = match gid {
            3 => {
                let r = rmax * u0.powf(0.55);
                let ct = 2.0 * rng.u() - 1.0;
                let st = (1.0 - ct * ct).sqrt();
                let ph = rng.u() * tau;
                let p = DVec3::new(
                    r * st * ph.cos(),
                    r * ct * 0.62,
                    r * st * ph.sin(),
                );
                let s = (G * core_m / r.max(1.0)).sqrt() * 0.55;
                let vv = DVec3::new(g1, g2, g3) * s;
                (p, vv)
            }
            5 => {
                let cc = clumps[i % clumps.len()];
                let sc = 0.22 * rmax;
                let p = cc
                    + DVec3::new(g1, g2 * 0.5, g3) * sc;
                let rr = p.length().max(1.0e11);
                let vc = (G * core_m / rr).sqrt() * 0.5;
                let t = DVec3::new(-p.z, 0.0, p.x)
                    .normalize_or_zero();
                let vv = t * vc * (0.4 + 0.6 * rng.u())
                    + DVec3::new(g1, g2, g3) * vc * 0.4;
                (p, vv)
            }
            7 => {
                let r = 0.18 * rmax * u0.powf(0.5);
                let ct = 2.0 * rng.u() - 1.0;
                let st = (1.0 - ct * ct).sqrt();
                let ph = rng.u() * tau;
                let p = DVec3::new(
                    r * st * ph.cos(),
                    r * ct * 0.8,
                    r * st * ph.sin(),
                );
                let s = (G * core_m / r.max(1.0)).sqrt() * 0.45;
                (p, DVec3::new(g1, g2, g3) * s)
            }
            _ => {
                let mut r =
                    -h_scale * (1.0 - 0.985 * u0).ln();
                r = r.clamp(2.0e11, rmax);
                let bulge = gid == 4 && rng.u() < 0.30;
                if bulge {
                    r = r.min(2.2e12) * 0.5;
                }
                let theta = if gid == 1 {
                    let arm = (i % 2) as f64;
                    r.ln() / pitch.tan()
                        + arm * tau / 2.0
                        + (0.35 + 1.6e12 / r) * g1
                } else if gid == 2 {
                    if r < 0.42 * rmax {
                        g1 * 0.18
                    } else {
                        let arm = (i % 2) as f64;
                        r.ln() / pitch.tan()
                            + arm * tau / 2.0
                            + (0.30 + 1.4e12 / r) * g1
                    }
                } else if gid == 6 {
                    rng.u() * tau
                } else {
                    rng.u() * tau
                };
                if gid == 2 && r < 0.42 * rmax {
                    r *= 1.0 + 0.6 * (rng.u() - 0.5);
                }
                if gid == 6 {
                    r = if rng.u() < 0.82 {
                        rmax * (0.62 + 0.14 * g1)
                    } else {
                        r.min(1.5e12)
                    };
                }
                let yscale =
                    if bulge { 0.5 } else { 0.035 };
                let y = g2 * (yscale * r + 6.0e10);
                let p = DVec3::new(
                    r * theta.cos(),
                    y,
                    r * theta.sin(),
                );
                let vc = (G * core_m / r.max(1.0)).sqrt()
                    * (0.9 + 0.2 * rng.u());
                let t =
                    DVec3::new(-theta.sin(), 0.0, theta.cos());
                let vv = t * vc
                    + DVec3::new(0.0, g3 * vc * 0.03, 0.0);
                (p, vv)
            }
        };

        let pick = rng.u();
        let class = if pick < 0.74 {
            BodyClass::RedDwarf
        } else if pick < 0.90 {
            BodyClass::SunLike
        } else if pick < 0.95 {
            BodyClass::RedGiant
        } else if pick < 0.972 {
            BodyClass::WhiteDwarf
        } else if pick < 0.984 {
            BodyClass::NeutronStar
        } else if pick < 0.990 {
            BodyClass::Pulsar
        } else if pick < 0.994 {
            BodyClass::Magnetar
        } else if pick < 0.997 {
            BodyClass::BlackHole
        } else {
            BodyClass::Quasar
        };
        let (mass, radius, _l, _t, kind) = class.props();

        v.push(SpawnSpec {
            name: format!("{} {i}", class.label()),
            kind,
            star: class != BodyClass::BlackHole,
            emissive: class.emissive(),
            radius,
            mass,
            pos,
            vel,
            atmo: None,
            rings: false,
        });
    }
    v
}

fn build_from_specs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    skies: &mut Assets<StarSky>,
    nebs: &mut Assets<NebulaMat>,
    auroras: &mut Assets<AuroraMat>,
    images: &mut Assets<Image>,
    selected: &mut Selected,
    specs: &[SpawnSpec],
) {
    commands.spawn((
        SolarScene,
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 120_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut home_entity: Option<Entity> = None;
    let mut star_glow: Option<(Entity, f32, LinearRgba)> = None;
    let mut evolving: Vec<(Entity, f64)> = Vec::new();
    let mut extras: Vec<(Entity, Option<Srgba>, u8, f32, bool)> = Vec::new();
    let cam_anchor = specs
        .iter()
        .find(|s| !s.star)
        .map(|s| (s.pos, s.radius))
        .unwrap_or((DVec3::new(1.5e11, 0.0, 0.0), 6.4e6));

    commands.spawn_big_space_default(|root| {
        root.insert(SolarScene);

        for (i, s) in specs.iter().enumerate() {
            let (cell, off) = root.grid().translation_to_grid(s.pos);
            let mat = if s.star {
                materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    emissive: s.emissive,
                    emissive_texture: Some(
                        crate::planetgen::make_star_texture(
                            images,
                            7 + i as u32,
                        ),
                    ),
                    ..default()
                })
            } else {
                materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: Some(make_texture(
                        images,
                        s.kind,
                        1000 + i as u32,
                    )),
                    perceptual_roughness: 0.9,
                    ..default()
                })
            };
            let mesh = if s.star {
                meshes.add(Sphere::new(s.radius as f32).mesh().ico(6).unwrap())
            } else {
                meshes.add(Sphere::new(s.radius as f32).mesh().uv(48, 32))
            };
            let id = root
                .spawn_spatial((
                    Grav {
                        mass: s.mass,
                        pos: s.pos,
                        vel: s.vel,
                    },
                    Name::new(s.name.clone()),
                    Shape {
                        base_radius: s.radius,
                        radius: s.radius,
                    },
                    Visual {
                        kind: s.kind,
                        star: s.star,
                        atmo: s.atmo,
                        rings: s.rings,
                        emissive: s.emissive,
                    },
                    Spin(if s.star { 2.9e-6 } else { 7.27e-5 }),
                    Trail::default(),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_translation(off).with_rotation(
                        if s.star {
                            Quat::IDENTITY
                        } else {
                            Quat::from_rotation_z(obliquity_rad(&s.name))
                        },
                    ),
                    cell,
                ))
                .id();
            if !s.star {
                let rstyle: u8 = if !s.rings {
                    0
                } else {
                    match s.name.as_str() {
                        "Saturn" => 1,
                        "Jupiter" => 2,
                        "Uranus" => 3,
                        "Neptune" => 4,
                        _ => 1,
                    }
                };
                extras.push((
                    id,
                    s.atmo,
                    rstyle,
                    s.radius as f32,
                    matches!(s.kind, PlanetKind::Terran),
                ));
                if home_entity.is_none() {
                    home_entity = Some(id);
                }
                if s.name == "Earth" {
                    home_entity = Some(id);
                }
            } else {
                star_glow = Some((id, s.radius as f32, s.emissive));
                if is_main_seq(s.mass, s.radius, true) {
                    evolving.push((id, s.mass));
                }
            }
        }

        let cam_pos = cam_anchor.0
            + DVec3::new(0.0, cam_anchor.1 * 4.0, -cam_anchor.1 * 11.0);
        let (cam_cell, cam_off) = root.grid().translation_to_grid(cam_pos);
        root.spawn_spatial((
            Camera3d::default(),
            Projection::from(PerspectiveProjection {
                near: 1.0e3,
                far: 5.0e13,
                ..default()
            }),
            Hdr,
            Exposure::SUNLIGHT,
            Bloom::NATURAL,
            PrimaryEguiContext,
            FloatingCam,
            Transform::from_translation(cam_off).looking_to(Vec3::Z, Vec3::Y),
            cam_cell,
            FloatingOrigin,
        ));

        root.spawn_spatial((
            SolarScene,
            SkyDome,
            Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
            MeshMaterial3d(skies.add(StarSky::default())),
            Transform::from_translation(cam_off)
                .with_scale(Vec3::splat(-SKY_RADIUS)),
            cam_cell,
        ));

        let neb_mesh = meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap());
        let nebulae = [
            (
                DVec3::new(8.0e12, 1.5e12, -4.0e12),
                3.0e12,
                Vec4::new(1.0, 0.30, 0.42, 1.0),
                Vec4::new(0.30, 0.55, 1.0, 1.0),
                1.3,
                0.9,
            ),
            (
                DVec3::new(-6.0e12, -2.0e12, 7.0e12),
                3.6e12,
                Vec4::new(0.25, 0.95, 0.75, 1.0),
                Vec4::new(0.35, 0.45, 1.0, 1.0),
                4.7,
                0.8,
            ),
            (
                DVec3::new(2.0e12, 5.0e12, 9.0e12),
                2.4e12,
                Vec4::new(1.0, 0.55, 0.22, 1.0),
                Vec4::new(0.85, 0.20, 0.55, 1.0),
                8.1,
                0.7,
            ),
        ];
        for (pos, rad, ca, cb, seed, d) in nebulae {
            let (nc, no) = root.grid().translation_to_grid(pos);
            root.spawn_spatial((
                SolarScene,
                Mesh3d(neb_mesh.clone()),
                MeshMaterial3d(nebs.add(NebulaMat {
                    params: NebParams {
                        col_a: ca,
                        col_b: cb,
                        p: Vec4::new(0.0, 7.0, seed, d),
                    },
                })),
                Transform::from_translation(no)
                    .with_scale(Vec3::splat(rad as f32)),
                nc,
            ));
        }

        let is_solar = specs.iter().any(|s| s.name == "Jupiter")
            && specs.iter().any(|s| s.name == "Mars");
        if is_solar {
            let sun = specs
                .iter()
                .filter(|s| s.star)
                .fold((0.0_f64, DVec3::ZERO), |a, s| {
                    if s.mass > a.0 {
                        (s.mass, s.pos)
                    } else {
                        a
                    }
                });
            if sun.0 > 0.0 {
                let ast_mesh = meshes
                    .add(Sphere::new(6.0e5).mesh().ico(2).unwrap());
                let ast_mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.42, 0.39, 0.35),
                    perceptual_roughness: 1.0,
                    ..default()
                });
                let gaps = [2.06_f64, 2.50, 2.82, 3.28];
                let mut rng = Lcg(0xA57E_201D);
                for _ in 0..150 {
                    let mut a_au = 2.06 + rng.u() * (3.28 - 2.06);
                    for g in gaps {
                        if (a_au - g).abs() < 0.035 {
                            a_au += 0.07;
                        }
                    }
                    let a = a_au * AU_M;
                    let e = (rng.u() * rng.u()) * 0.22;
                    let inc = (rng.u() * rng.u()) * 0.30;
                    let node = rng.u() * std::f64::consts::TAU;
                    let nu = rng.u() * std::f64::consts::TAU;
                    let r = a * (1.0 - e * nu.cos());
                    let vc = (G * sun.0 / r).sqrt()
                        * (1.0 + (rng.u() - 0.5) * 0.06);
                    let p0 = DVec3::new(
                        r * nu.cos(),
                        0.0,
                        r * nu.sin(),
                    );
                    let v0 = DVec3::new(
                        -nu.sin() * vc,
                        0.0,
                        nu.cos() * vc,
                    );
                    let axis = DVec3::new(
                        node.cos(),
                        0.0,
                        node.sin(),
                    );
                    let q = bevy::math::DQuat::from_axis_angle(
                        axis, inc,
                    );
                    let pos = sun.1 + q * p0;
                    let vel = q * v0;
                    let (cell, off) =
                        root.grid().translation_to_grid(pos);
                    root.spawn_spatial((
                        SolarScene,
                        Asteroid,
                        Grav {
                            mass: 1.0e16,
                            pos,
                            vel,
                        },
                        Shape {
                            base_radius: 6.0e5,
                            radius: 6.0e5,
                        },
                        Name::new("Asteroid"),
                        Mesh3d(ast_mesh.clone()),
                        MeshMaterial3d(ast_mat.clone()),
                        Transform::from_translation(off),
                        cell,
                    ));
                }

                if let Some(jup) =
                    specs.iter().find(|s| s.name == "Jupiter")
                {
                    let rj = (jup.pos - sun.1).length();
                    let th_j = (jup.pos.z - sun.1.z)
                        .atan2(jup.pos.x - sun.1.x);
                    let camps = [
                        (std::f64::consts::FRAC_PI_3, 60usize),
                        (-std::f64::consts::FRAC_PI_3, 40usize),
                    ];
                    for (lead, count) in camps {
                        for _ in 0..count {
                            let th = th_j
                                + lead
                                + (rng.u() - 0.5) * 0.9;
                            let r = rj * (1.0 + (rng.u() - 0.5) * 0.08);
                            let inc = (rng.u() - 0.5) * 0.24;
                            let vc = (G * sun.0 / r).sqrt();
                            let p0 = DVec3::new(
                                r * th.cos(),
                                0.0,
                                r * th.sin(),
                            );
                            let v0 = DVec3::new(
                                -th.sin() * vc,
                                0.0,
                                th.cos() * vc,
                            );
                            let node = rng.u()
                                * std::f64::consts::TAU;
                            let axis = DVec3::new(
                                node.cos(),
                                0.0,
                                node.sin(),
                            );
                            let q =
                                bevy::math::DQuat::from_axis_angle(
                                    axis, inc,
                                );
                            let pos = sun.1 + q * p0;
                            let vel = q * v0;
                            let (cell, off) = root
                                .grid()
                                .translation_to_grid(pos);
                            root.spawn_spatial((
                                SolarScene,
                                Asteroid,
                                Grav {
                                    mass: 1.0e16,
                                    pos,
                                    vel,
                                },
                                Shape {
                                    base_radius: 6.0e5,
                                    radius: 6.0e5,
                                },
                                Name::new("Trojan"),
                                Mesh3d(ast_mesh.clone()),
                                MeshMaterial3d(ast_mat.clone()),
                                Transform::from_translation(off),
                                cell,
                            ));
                        }
                    }
                }
            }
        }
    });

    selected.0 = home_entity;

    for (planet, atmo, rstyle, r, clouds) in extras {
        let cloud_tex = if clouds {
            Some(crate::planetgen::make_cloud_texture(images, 4242))
        } else {
            None
        };
        commands.entity(planet).with_children(|c| {
            if let Some(ct) = cloud_tex {
                c.spawn((
                    Spin(9.0e-5),
                    Mesh3d(meshes.add(Sphere::new(r).mesh().uv(40, 28))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(ct),
                        alpha_mode: AlphaMode::Blend,
                        perceptual_roughness: 1.0,
                        ..default()
                    })),
                    Transform::from_scale(Vec3::splat(1.02)),
                ));
            }
            if let Some(tint) = atmo {
                c.spawn((
                    Mesh3d(meshes.add(Sphere::new(r).mesh().ico(6).unwrap())),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::BLACK,
                        emissive: LinearRgba::from(Color::Srgba(tint)) * 0.5,
                        unlit: true,
                        alpha_mode: AlphaMode::Add,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_scale(Vec3::splat(1.04)),
                    AtmoShell,
                ));
                c.spawn((
                    Mesh3d(meshes.add(Sphere::new(r).mesh().ico(6).unwrap())),
                    MeshMaterial3d(auroras.add(AuroraMat {
                        params: AuroraParams {
                            col: Vec4::new(0.22, 1.0, 0.48, 1.0),
                            p: Vec4::new(0.0, 0.0, 0.0, (r * 0.37) % 10.0),
                        },
                    })),
                    Transform::from_scale(Vec3::splat(1.07)),
                    AuroraShell(planet),
                ));
            }
            if rstyle > 0 {
                let (ri, ro) = match rstyle {
                    2 => (1.45, 1.80),
                    3 => (1.60, 2.00),
                    4 => (1.70, 2.50),
                    _ => (1.30, 2.30),
                };
                c.spawn((
                    Mesh3d(meshes.add(
                        Annulus::new(r * ri, r * ro).mesh().build(),
                    )),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(
                            crate::planetgen::make_ring_texture(
                                images, rstyle,
                            ),
                        ),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_rotation(Quat::from_rotation_x(
                        -std::f32::consts::FRAC_PI_2,
                    )),
                ));
            }
        });
    }

    for (e, m) in evolving {
        commands.entity(e).insert(Star {
            age: 0.0,
            t_ms: ms_lifetime(m),
            stage: 0,
        });
    }

    if let Some((star, sr, em)) = star_glow {
        commands.entity(star).with_children(|c| {
            c.spawn((
                Mesh3d(meshes.add(Sphere::new(sr).mesh().ico(5).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    emissive: em * 0.10,
                    unlit: true,
                    alpha_mode: AlphaMode::Add,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_scale(Vec3::splat(2.6)),
            ));
        });
    }
}

fn nbody(
    time: Res<Time>,
    speed: Res<SimSpeed>,
    paused: Res<Paused>,
    rel: Res<Relativity>,
    mut clock: ResMut<SimClock>,
    grid: Query<&Grid>,
    mut bodies: Query<
        (&mut Grav, &mut CellCoord, &mut Transform),
        Without<GwBound>,
    >,
) {
    if paused.0 {
        return;
    }
    let Ok(grid) = grid.single() else {
        return;
    };

    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    clock.0 += sim_dt;
    let substeps = (sim_dt / 1800.0).ceil().clamp(1.0, 96.0) as u32;
    let h = sim_dt / substeps as f64;

    let mut state: Vec<(f64, DVec3, DVec3)> = bodies
        .iter()
        .map(|(g, _, _)| (g.mass, g.pos, g.vel))
        .collect();

    let star = state
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap())
        .map(|(i, _)| i);
    let c2 = C_LIGHT * C_LIGHT;
    let vmax = 0.999 * C_LIGHT;

    for _ in 0..substeps {
        let snapshot: Vec<(f64, DVec3, DVec3)> = state.clone();
        let (sm, sp, sv) = star
            .map(|s| (snapshot[s].0, snapshot[s].1, snapshot[s].2))
            .unwrap_or((0.0, DVec3::ZERO, DVec3::ZERO));
        for (i, item) in state.iter_mut().enumerate() {
            let mut acc = DVec3::ZERO;
            for (j, (mj, pj, _)) in snapshot.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = *pj - item.1;
                let r2 = d.length_squared().max(1.0e6);
                acc += d * (G * *mj / (r2 * r2.sqrt()));
            }
            if rel.0 && star != Some(i) && sm > 0.0 {
                // 1PN Schwarzschild correction from the dominant mass
                let r = item.1 - sp;
                let rm = r.length().max(1.0e3);
                let v = snapshot[i].2 - sv;
                let gm = G * sm;
                let rv = r.dot(v);
                let a_pn = (gm / (c2 * rm * rm * rm))
                    * ((4.0 * gm / rm - v.dot(v)) * r + 4.0 * rv * v);
                acc += a_pn;
            }
            item.2 += acc * h;
            let sp2 = item.2.length();
            if sp2 > vmax {
                item.2 *= vmax / sp2;
            }
        }
        for item in state.iter_mut() {
            item.1 += item.2 * h;
        }
    }

    for ((mut g, mut cell, mut tf), s) in bodies.iter_mut().zip(state) {
        g.pos = s.1;
        g.vel = s.2;
        let (new_cell, off) = grid.translation_to_grid(g.pos);
        *cell = new_cell;
        tf.translation = off;
    }
}

fn solar_mouse_input(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut orbit: ResMut<OrbitView>,
    mut paused: ResMut<Paused>,
    mut grid_vis: ResMut<ShowGrid>,
    mut mode: ResMut<CamMode>,
    mut fly: ResMut<FlyState>,
    cw: Res<CamWorld>,
) {
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        grid_vis.0 = !grid_vis.0;
    }
    if keys.just_pressed(KeyCode::KeyF) {
        *mode = if *mode == CamMode::Fly {
            CamMode::Follow
        } else {
            let f = cw.fwd;
            fly.pos = cw.pos;
            fly.yaw = (f.z as f32).atan2(f.x as f32);
            fly.pitch =
                (f.y as f32).clamp(-1.0, 1.0).asin().clamp(-1.5, 1.5);
            if fly.speed <= 0.0 {
                fly.speed = 2.0e9;
            }
            CamMode::Fly
        };
    }
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_pointer_input() {
            return;
        }
    }
    if buttons.pressed(MouseButton::Left) {
        let d = motion.delta;
        orbit.yaw -= d.x * 0.005;
        orbit.pitch = (orbit.pitch - d.y * 0.005).clamp(-1.5, 1.5);
    }
    let s = scroll.delta.y;
    if s != 0.0 {
        orbit.dist = (orbit.dist * (1.0 - s * 0.1)).clamp(2.5, 600.0);
    }
}

fn follow_cam(
    time: Res<Time>,
    mode: Res<CamMode>,
    selected: Res<Selected>,
    orbit: Res<OrbitView>,
    exag: Res<SizeExaggeration>,
    grid: Query<&Grid>,
    target: Query<(&Grav, &Shape, Option<&FocusRadius>)>,
    mut cam: Query<(&mut CellCoord, &mut Transform), With<FloatingCam>>,
    mut cw: ResMut<CamWorld>,
    mut tr: Local<Option<(Entity, DVec3, f32, DVec3)>>,
) {
    if *mode == CamMode::Fly {
        return;
    }
    let Ok(grid) = grid.single() else {
        return;
    };
    let Some(sel) = selected.0 else {
        return;
    };
    let Ok((t, shape, focus_r)) = target.get(sel) else {
        return;
    };
    let Ok((mut cell, mut tf)) = cam.single_mut() else {
        return;
    };

    let dt = time.delta().as_secs_f64() as f32;
    let desired = t.pos;
    let (from, prog) = match *tr {
        Some((psel, from, prog, _)) if psel == sel => {
            (from, (prog + dt / 0.8).min(1.0))
        }
        Some((_, _, _, last_focus)) => (last_focus, 0.0),
        None => (desired, 1.0),
    };
    let k = (prog * prog * (3.0 - 2.0 * prog)) as f64;
    let focus = from + (desired - from) * k;
    *tr = Some((sel, from, prog, focus));

    let extent = match focus_r {
        Some(f) => f.0 * 0.22,
        None => shape.radius.max(shape.base_radius),
    };
    let r = extent * exag.0.max(1.0) as f64 * orbit.dist as f64;
    let cp = orbit.pitch.cos() as f64;
    let dir_off = DVec3::new(
        cp * orbit.yaw.cos() as f64,
        orbit.pitch.sin() as f64,
        cp * orbit.yaw.sin() as f64,
    ) * r;
    let cam_pos = focus + dir_off;
    let (new_cell, off) = grid.translation_to_grid(cam_pos);
    let fwd = (focus - cam_pos).normalize();
    let right = fwd.cross(DVec3::Y).normalize();
    let up = right.cross(fwd);

    *cell = new_cell;
    *tf = Transform::from_translation(off)
        .looking_to(fwd.as_vec3(), Vec3::Y);

    cw.pos = cam_pos;
    cw.fwd = fwd;
    cw.right = right;
    cw.up = up;
}

fn fly_cam(
    time: Res<Time>,
    mode: Res<CamMode>,
    mut contexts: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    grid: Query<&Grid>,
    mut fly: ResMut<FlyState>,
    mut cam: Query<(&mut CellCoord, &mut Transform), With<FloatingCam>>,
    mut cw: ResMut<CamWorld>,
) {
    if *mode != CamMode::Fly {
        return;
    }
    let Ok(grid) = grid.single() else {
        return;
    };
    let Ok((mut cell, mut tf)) = cam.single_mut() else {
        return;
    };

    let mut over_egui = false;
    if let Ok(ctx) = contexts.ctx_mut() {
        over_egui = ctx.wants_pointer_input();
    }
    if !over_egui {
        let s = scroll.delta.y;
        if s != 0.0 {
            fly.speed =
                (fly.speed * (1.0 - s as f64 * 0.18)).clamp(1.0e5, 5.0e13);
        }
        if buttons.pressed(MouseButton::Left) {
            let d = motion.delta;
            fly.yaw += d.x * 0.0032;
            fly.pitch = (fly.pitch - d.y * 0.0032).clamp(-1.54, 1.54);
        }
    }

    let cp = fly.pitch.cos() as f64;
    let fwd = DVec3::new(
        cp * fly.yaw.cos() as f64,
        fly.pitch.sin() as f64,
        cp * fly.yaw.sin() as f64,
    )
    .normalize();
    let right = fwd.cross(DVec3::Y).normalize();
    let up = right.cross(fwd);

    let mut dir = DVec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += fwd;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= fwd;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keys.pressed(KeyCode::Space) {
        dir += DVec3::Y;
    }
    if keys.pressed(KeyCode::ControlLeft) {
        dir -= DVec3::Y;
    }
    let boost = if keys.pressed(KeyCode::ShiftLeft) {
        24.0
    } else if keys.pressed(KeyCode::AltLeft) {
        0.06
    } else {
        1.0
    };
    if dir != DVec3::ZERO {
        let dt = time.delta().as_secs_f64();
        let step = dir.normalize() * fly.speed * boost * dt;
        fly.pos += step;
    }

    let (new_cell, off) = grid.translation_to_grid(fly.pos);
    *cell = new_cell;
    *tf = Transform::from_translation(off).looking_to(fwd.as_vec3(), Vec3::Y);
    cw.pos = fly.pos;
    cw.fwd = fwd;
    cw.right = right;
    cw.up = up;
}

fn follow_sky(
    cam: Query<(&CellCoord, &Transform), (With<FloatingCam>, Without<SkyDome>)>,
    mut sky: Query<(&mut CellCoord, &mut Transform), With<SkyDome>>,
) {
    let Ok((cc, ct)) = cam.single() else {
        return;
    };
    let Ok((mut sc, mut st)) = sky.single_mut() else {
        return;
    };
    *sc = *cc;
    st.translation = ct.translation;
    st.rotation = Quat::IDENTITY;
    st.scale = Vec3::splat(-SKY_RADIUS);
}

fn apply_shape(
    exag: Res<SizeExaggeration>,
    mut q: Query<(&Shape, &mut Transform), (Without<SkyDome>, Without<FloatingCam>)>,
) {
    for (shape, mut tf) in &mut q {
        let s = (shape.radius / shape.base_radius) as f32 * exag.0.max(1.0);
        tf.scale = Vec3::splat(s.max(1.0e-4));
    }
}

#[derive(Component)]
pub struct Debris;

#[derive(Component)]
pub struct Asteroid;

fn density(mass: f64, radius: f64) -> f64 {
    let r = radius.max(1.0);
    mass / (4.18879 * r * r * r)
}

fn tidal_disruption(
    paused: Res<Paused>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    grid: Query<(Entity, &Grid)>,
    q: Query<(Entity, &Grav, &Shape, Option<&Debris>)>,
) {
    if paused.0 {
        return;
    }
    let Ok((grid_e, g_grid)) = grid.single() else {
        return;
    };
    let all: Vec<(Entity, f64, DVec3, DVec3, f64, bool)> = q
        .iter()
        .map(|(e, g, s, d)| {
            (e, g.mass, g.pos, g.vel, s.base_radius, d.is_some())
        })
        .collect();
    if all.len() > 380 {
        return;
    }

    let mut rng = Lcg(time.elapsed().as_nanos() as u64 | 1);
    for &(se, sm, sp, sv, sr, sdeb) in &all {
        if sdeb || sm < 1.0e15 || sr <= 0.0 {
            continue;
        }
        let mut hit: Option<(f64, DVec3)> = None;
        for &(be, bm, bp, _, br, bdeb) in &all {
            if be == se || bdeb || bm < 20.0 * sm || br <= 0.0 {
                continue;
            }
            let dist = (sp - bp).length();
            let roche = 2.44
                * br
                * (density(bm, br) / density(sm, sr)).cbrt();
            if dist < roche && dist > br * 1.05 {
                hit = Some((bm, bp));
                break;
            }
        }
        let Some((bm, bp)) = hit else {
            continue;
        };

        let n = 16usize;
        let fm = sm / n as f64;
        let fr = (sr / (n as f64).cbrt()).max(1.0e3);
        let dist = (sp - bp).length();
        let rr = (sp - bp) / dist;
        let nrm = rr.cross(sv).normalize_or_zero();
        let nrm = if nrm.length() < 0.5 { DVec3::Y } else { nrm };
        let circ = (G * bm / dist).sqrt();
        let mesh = meshes.add(Sphere::new(fr as f32).mesh().uv(16, 12));
        let mat = class_material(
            BodyClass::Rocky,
            &mut materials,
            &mut images,
        );
        commands.entity(se).despawn();
        for k in 0..n {
            let a = (k as f64 / n as f64 - 0.5) * 0.5
                + (rng.u() - 0.5) * 0.05;
            let (sa, ca) = (a.sin(), a.cos());
            let dir = rr * ca + nrm.cross(rr) * sa;
            let rd = dist * (0.97 + 0.06 * rng.u());
            let pos = bp + dir * rd;
            let tang = nrm.cross(dir).normalize_or_zero();
            let vel = tang * circ * (0.97 + 0.06 * rng.u())
                + dir * (rng.u() - 0.5) * circ * 0.03;
            let (cell, off) = g_grid.translation_to_grid(pos);
            let frag = commands
                .spawn((
                    Grav {
                        mass: fm,
                        pos,
                        vel,
                    },
                    Shape {
                        base_radius: fr,
                        radius: fr,
                    },
                    Visual {
                        kind: PlanetKind::Rocky {
                            r: 0.5,
                            g: 0.47,
                            b: 0.43,
                        },
                        star: false,
                        atmo: None,
                        rings: false,
                        emissive: LinearRgba::BLACK,
                    },
                    Debris,
                    Trail::default(),
                    Name::new("Debris"),
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(off),
                    cell,
                ))
                .id();
            commands.entity(grid_e).add_child(frag);
        }
    }
}

#[derive(Resource)]
pub struct AutoAccrete(pub u32);

pub fn auto_accrete(
    mut a: ResMut<AutoAccrete>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut sel: ResMut<Selected>,
    grid: Query<(Entity, &Grid)>,
    q: Query<(Entity, &Grav, &Shape, Option<&Debris>)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let Ok((grid_e, g_grid)) = grid.single() else {
        return;
    };
    let mut prim = (0.0_f64, DVec3::ZERO, DVec3::ZERO, 0.0_f64, None);
    for (e, g, s, d) in &q {
        if d.is_none()
            && g.mass < 1.0e29
            && g.mass > prim.0
            && s.base_radius > 0.0
        {
            prim = (g.mass, g.pos, g.vel, s.base_radius, Some(e));
        }
    }
    if prim.4.is_none() {
        return;
    }
    let n = 30usize;
    let fm = 4.0e17;
    let fr = 3.6e4;
    let a_orb = prim.3 * 9.0;
    let circ = (G * prim.0 / a_orb).sqrt();
    let mesh = meshes.add(Sphere::new(fr as f32).mesh().uv(16, 12));
    let mat =
        class_material(BodyClass::Rocky, &mut materials, &mut images);
    let mut rng = Lcg(0x9e3779b97f4a7c15);
    let dir0 = DVec3::new(1.0, 0.0, 0.0);
    let tang0 = DVec3::new(0.0, 0.0, 1.0);
    let center = prim.1 + dir0 * a_orb;
    let blob = 4.0e5;
    for _ in 0..n {
        let pos = center
            + DVec3::new(
                (rng.u() - 0.5) * 2.0 * blob,
                (rng.u() - 0.5) * 2.0 * blob,
                (rng.u() - 0.5) * 2.0 * blob,
            );
        let vel = prim.2
            + tang0 * circ
            + DVec3::new(
                (rng.u() - 0.5) * 8.0,
                (rng.u() - 0.5) * 8.0,
                (rng.u() - 0.5) * 8.0,
            );
        let (cell, off) = g_grid.translation_to_grid(pos);
        let frag = commands
            .spawn((
                Grav {
                    mass: fm,
                    pos,
                    vel,
                },
                Shape {
                    base_radius: fr,
                    radius: fr,
                },
                Visual {
                    kind: PlanetKind::Rocky {
                        r: 0.5,
                        g: 0.47,
                        b: 0.43,
                    },
                    star: false,
                    atmo: None,
                    rings: false,
                    emissive: LinearRgba::BLACK,
                },
                Debris,
                Trail::default(),
                Name::new("Debris"),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_translation(off),
                cell,
            ))
            .id();
        commands.entity(grid_e).add_child(frag);
    }
    sel.0 = prim.4;
    a.0 = u32::MAX;
}

const MOONLET_MASS: f64 = 1.5e18;

fn debris_accretion(
    paused: Res<Paused>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Grav, &mut Shape, Option<&Debris>)>,
) {
    if paused.0 {
        return;
    }
    let snap: Vec<(Entity, f64, DVec3, DVec3, f64, bool)> = q
        .iter()
        .map(|(e, g, s, d)| {
            (e, g.mass, g.pos, g.vel, s.base_radius, d.is_some())
        })
        .collect();
    if snap.len() > 460 {
        return;
    }
    let mut prim = (0.0_f64, DVec3::ZERO, 1.0_f64);
    for &(_, m, p, _, r, deb) in &snap {
        if !deb && m > prim.0 {
            prim = (m, p, r.max(1.0));
        }
    }
    if prim.0 <= 0.0 {
        return;
    }
    let rho_p = density(prim.0, prim.2);
    let mut gone: Vec<Entity> = Vec::new();
    let mut grew: Vec<(Entity, f64, DVec3, DVec3, f64)> = Vec::new();
    let mut used = vec![false; snap.len()];
    let mut merges = 0usize;
    for i in 0..snap.len() {
        if used[i] || !snap[i].5 || merges >= 24 {
            continue;
        }
        let (ei, mi, pi, vi, ri, _) = snap[i];
        let ai = (pi - prim.1).length();
        let rho_d = density(mi, ri);
        let d_roche = 2.44 * prim.2 * (rho_p / rho_d.max(1.0)).cbrt();
        if ai <= d_roche {
            continue;
        }
        for j in (i + 1)..snap.len() {
            if used[j] || !snap[j].5 {
                continue;
            }
            let (ej, mj, pj, vj, rj, _) = snap[j];
            let aj = (pj - prim.1).length();
            if aj <= d_roche {
                continue;
            }
            let sep = (pi - pj).length();
            if sep <= 0.0 {
                continue;
            }
            let v_esc =
                (2.0 * G * (mi + mj) / (ri + rj)).sqrt();
            let v_rel = (vi - vj).length();
            if v_rel >= v_esc {
                continue;
            }
            let focus =
                (1.0 + (v_esc / v_rel.max(1.0e-3)).powi(2)).sqrt();
            let cap = (ri + rj) * focus;
            let am = 0.5 * (ai + aj);
            let r_hill =
                am * ((mi + mj) / (3.0 * prim.0)).cbrt();
            let lim = cap.min(r_hill.max(ri + rj));
            if sep > lim {
                continue;
            }
            let (m, v, r) = crate::physics::merge_bodies(
                mi,
                [vi.x, vi.y, vi.z],
                ri,
                mj,
                [vj.x, vj.y, vj.z],
                rj,
            );
            let pos = (pi * mi + pj * mj) / m;
            grew.push((
                ei,
                m,
                pos,
                DVec3::new(v[0], v[1], v[2]),
                r,
            ));
            gone.push(ej);
            used[i] = true;
            used[j] = true;
            merges += 1;
            break;
        }
    }
    for e in gone {
        commands.entity(e).despawn();
    }
    for (e, m, pos, vel, r) in grew {
        if let Ok((_, mut g, mut s, _)) = q.get_mut(e) {
            g.mass = m;
            g.pos = pos;
            g.vel = vel;
            s.radius = r;
            if m >= MOONLET_MASS {
                commands
                    .entity(e)
                    .remove::<Debris>()
                    .insert(Name::new("Moonlet"));
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct MassXfer {
    pub active: bool,
    pub donor: String,
    pub acc: String,
    pub fill: f64,
    pub rate_msun_yr: f64,
    pub acc_e: Option<Entity>,
    pub donor_e: Option<Entity>,
}

#[derive(Component)]
pub struct MtRig {
    pub acc: Entity,
}

fn eggleton_rl(a: f64, q: f64) -> f64 {
    let q3 = q.powf(1.0 / 3.0);
    let q23 = q3 * q3;
    a * 0.49 * q23 / (0.6 * q23 + (1.0 + q3).ln())
}

const K_MT: f64 = 6.0e-3;

fn mass_transfer(
    paused: Res<Paused>,
    time: Res<Time>,
    speed: Res<SimSpeed>,
    mut info: ResMut<MassXfer>,
    names: Query<&Name>,
    mut q: Query<(Entity, &mut Grav, &mut Shape, &Visual)>,
) {
    info.active = false;
    if paused.0 {
        return;
    }
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    let snap: Vec<(Entity, f64, DVec3, DVec3, f64, bool)> = q
        .iter()
        .map(|(e, g, s, v)| {
            (e, g.mass, g.pos, g.vel, s.base_radius, v.star)
        })
        .collect();
    let stars: Vec<usize> = (0..snap.len())
        .filter(|&i| snap[i].5 && snap[i].1 > 0.0)
        .collect();
    if stars.len() < 2 {
        return;
    }
    let mut bestpair = (f64::MAX, 0usize, 0usize);
    for ia in 0..stars.len() {
        for ib in (ia + 1)..stars.len() {
            let i = stars[ia];
            let j = stars[ib];
            let d = (snap[i].2 - snap[j].2).length();
            if d < bestpair.0 {
                bestpair = (d, i, j);
            }
        }
    }
    let (d, i, j) = bestpair;
    if d <= 0.0 {
        return;
    }
    let (ei, mi, _pi, vi, ri, _) = snap[i];
    let (ej, mj, _pj, vj, rj, _) = snap[j];
    let vrel = (vi - vj).length_squared();
    let eorb = 0.5 * vrel - G * (mi + mj) / d;
    if eorb >= 0.0 {
        return;
    }
    let rl_i = eggleton_rl(d, mi / mj);
    let rl_j = eggleton_rl(d, mj / mi);
    let fill_i = ri / rl_i.max(1.0);
    let fill_j = rj / rl_j.max(1.0);
    let (de, ae, dm0, dv, am0, av, fill) = if fill_i >= fill_j {
        (ei, ej, mi, vi, mj, vj, fill_i)
    } else {
        (ej, ei, mj, vj, mi, vi, fill_j)
    };
    if fill <= 1.0 {
        return;
    }
    let over = (fill - 1.0).min(1.5);
    let mut dm = K_MT * over * over * over * dm0 * sim_dt / 3.15e7;
    dm = dm.min(0.02 * dm0).max(0.0);
    if dm <= 0.0 {
        return;
    }
    let nd = dm0 - dm;
    let na = am0 + dm;
    let nav = (av * am0 + dv * dm) / na;
    let rd_scale = (nd / dm0).cbrt();
    let ra_scale = (na / am0).cbrt();
    if let Ok((_, mut g, mut s, _)) = q.get_mut(de) {
        g.mass = nd;
        s.base_radius *= rd_scale;
        s.radius *= rd_scale;
    }
    if let Ok((_, mut g, mut s, _)) = q.get_mut(ae) {
        g.mass = na;
        g.vel = nav;
        s.base_radius *= ra_scale;
        s.radius *= ra_scale;
    }
    info.active = true;
    info.donor =
        names.get(de).map(|n| n.as_str().into()).unwrap_or_default();
    info.acc =
        names.get(ae).map(|n| n.as_str().into()).unwrap_or_default();
    info.fill = fill;
    info.rate_msun_yr =
        (dm / sim_dt) * 3.15e7 / SUN_MASS;
    info.acc_e = Some(ae);
    info.donor_e = Some(de);
}

fn sync_mt_rig(
    mut commands: Commands,
    mx: Res<MassXfer>,
    grid: Query<(Entity, &Grid)>,
    bodies: Query<(&Grav, &Shape)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut disks: ResMut<Assets<DiskMat>>,
    mut rigs: Query<(Entity, &MtRig, &mut CellCoord, &mut Transform)>,
) {
    let Ok((grid_e, g_grid)) = grid.single() else {
        return;
    };
    let live = mx.active && mx.acc_e.is_some();
    if !live {
        for (e, _, _, _) in &rigs {
            commands.entity(e).despawn();
        }
        return;
    }
    let acc = mx.acc_e.unwrap();
    let Ok((ag, ash)) = bodies.get(acc) else {
        return;
    };
    let donor_d = mx
        .donor_e
        .and_then(|d| bodies.get(d).ok())
        .map(|(dg, _)| (dg.pos - ag.pos).length())
        .unwrap_or(4.0e9);
    let mut found = false;
    for (_, rig, mut cell, mut tf) in &mut rigs {
        if rig.acc != acc {
            continue;
        }
        found = true;
        let (c, off) = g_grid.translation_to_grid(ag.pos);
        *cell = c;
        *tf = Transform::from_translation(off).with_rotation(
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        );
    }
    if found {
        return;
    }
    let r_in = (ash.base_radius as f32 * 2.5)
        .max(donor_d as f32 * 0.05);
    let r_out = (donor_d as f32 * 0.45).max(r_in * 2.0);
    let mesh = meshes.add(
        Annulus::new(r_in, r_out).mesh().resolution(96).build(),
    );
    let mat = disks.add(DiskMat {
        params: DiskParams {
            hot: Vec4::new(0.9, 0.95, 1.0, 1.0),
            cool: Vec4::new(1.0, 0.5, 0.16, 1.0),
            p: Vec4::new(0.0, 1.0, 3.0, 3.4),
        },
    });
    let (c, off) = g_grid.translation_to_grid(ag.pos);
    let rig = commands
        .spawn((
            MtRig { acc },
            c,
            Transform::from_translation(off).with_rotation(
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            Visibility::Visible,
        ))
        .id();
    commands.entity(grid_e).add_child(rig);
    commands.entity(rig).with_children(|p| {
        p.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::default(),
            Jet,
        ));
    });
}

fn collisions(
    paused: Res<Paused>,
    mut selected: ResMut<Selected>,
    mut commands: Commands,
    mut kreq: ResMut<KilonovaReq>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    grid: Query<(Entity, &Grid)>,
    mut q: Query<(Entity, &mut Grav, &mut Shape)>,
) {
    if paused.0 {
        return;
    }
    let mut bodies: Vec<(Entity, f64, DVec3, DVec3, f64)> = q
        .iter()
        .map(|(e, g, s)| (e, g.mass, g.pos, g.vel, s.radius))
        .collect();

    let mut did_frag = false;
    if bodies.len() <= 360 {
        if let Ok((grid_e, g_grid)) = grid.single() {
            let nb = bodies.len();
            'outer: for i in 0..nb {
                for j in (i + 1)..nb {
                    let d = (bodies[i].2 - bodies[j].2).length();
                    if d >= bodies[i].4 + bodies[j].4 {
                        continue;
                    }
                    let (bi, si) = if bodies[i].1 >= bodies[j].1 {
                        (i, j)
                    } else {
                        (j, i)
                    };
                    let (m1, p1, v1, r1) = (
                        bodies[bi].1,
                        bodies[bi].2,
                        bodies[bi].3,
                        bodies[bi].4,
                    );
                    let (m2, p2, v2, r2) = (
                        bodies[si].1,
                        bodies[si].2,
                        bodies[si].3,
                        bodies[si].4,
                    );
                    let mm = m1 + m2;
                    if mm <= 0.0 {
                        continue;
                    }
                    let v_esc =
                        (2.0 * G * mm / (r1 + r2).max(1.0)).sqrt();
                    let v_imp = (v1 - v2).length();
                    if v_imp < 1.4 * v_esc {
                        continue;
                    }
                    let ratio = (v_imp / v_esc.max(1.0)).min(6.0);
                    let f_lr = (1.15 - 0.55 * ratio).clamp(0.12, 0.85);
                    let v_cm = (v1 * m1 + v2 * m2) / mm;
                    let com = (p1 * m1 + p2 * m2) / mm;
                    let m_rem = mm * f_lr;
                    let k = 14usize;
                    let mf = (mm - m_rem) / k as f64;
                    let r_tot = (r1 * r1 * r1 + r2 * r2 * r2).cbrt();
                    let r_rem = (r_tot * f_lr.cbrt()).max(1.0e3);
                    let rf =
                        (r_rem * (mf / m_rem.max(1.0)).cbrt()).max(1.0e3);
                    let mut rng =
                        Lcg((p1.x.to_bits() ^ v_imp.to_bits()) | 1);
                    let mut ej: Vec<DVec3> = Vec::with_capacity(k);
                    let mut mean = DVec3::ZERO;
                    for _ in 0..k {
                        let u = rng.u() * std::f64::consts::TAU;
                        let w = rng.u() * 2.0 - 1.0;
                        let s = (1.0 - w * w).max(0.0).sqrt();
                        let dir = DVec3::new(
                            s * u.cos(),
                            s * u.sin(),
                            w,
                        );
                        let sp = v_esc * (0.8 + 0.9 * rng.u());
                        let e = dir * sp;
                        ej.push(e);
                        mean += e;
                    }
                    mean /= k as f64;
                    let be = bodies[bi].0;
                    let se = bodies[si].0;
                    commands.entity(be).despawn();
                    commands.entity(se).despawn();
                    let mesh_r =
                        meshes.add(Sphere::new(r_rem as f32).mesh().uv(28, 18));
                    let mesh_f =
                        meshes.add(Sphere::new(rf as f32).mesh().uv(14, 10));
                    let mat = class_material(
                        BodyClass::Rocky,
                        &mut materials,
                        &mut images,
                    );
                    let vis = Visual {
                        kind: PlanetKind::Rocky {
                            r: 0.52,
                            g: 0.48,
                            b: 0.44,
                        },
                        star: false,
                        atmo: None,
                        rings: false,
                        emissive: LinearRgba::BLACK,
                    };
                    let (rc, ro) = g_grid.translation_to_grid(com);
                    let rem = commands
                        .spawn((
                            Grav {
                                mass: m_rem,
                                pos: com,
                                vel: v_cm,
                            },
                            Shape {
                                base_radius: r_rem,
                                radius: r_rem,
                            },
                            vis,
                            Trail::default(),
                            Name::new("Impact remnant"),
                            Mesh3d(mesh_r),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(ro),
                            rc,
                        ))
                        .id();
                    commands.entity(grid_e).add_child(rem);
                    if selected.0 == Some(be) || selected.0 == Some(se) {
                        selected.0 = Some(rem);
                    }
                    for e in ej {
                        let dv = e - mean;
                        let fp = com
                            + dv.normalize_or_zero() * (r_tot * 1.4);
                        let (fc, fo) = g_grid.translation_to_grid(fp);
                        let frag = commands
                            .spawn((
                                Grav {
                                    mass: mf,
                                    pos: fp,
                                    vel: v_cm + dv,
                                },
                                Shape {
                                    base_radius: rf,
                                    radius: rf,
                                },
                                vis,
                                Debris,
                                Trail::default(),
                                Name::new("Debris"),
                                Mesh3d(mesh_f.clone()),
                                MeshMaterial3d(mat.clone()),
                                Transform::from_translation(fo),
                                fc,
                            ))
                            .id();
                        commands.entity(grid_e).add_child(frag);
                    }
                    did_frag = true;
                    break 'outer;
                }
            }
        }
    }
    if did_frag {
        return;
    }

    let n = bodies.len();
    let mut absorbed: std::collections::HashMap<Entity, Entity> =
        std::collections::HashMap::new();

    for i in 0..n {
        if absorbed.contains_key(&bodies[i].0) {
            continue;
        }
        for j in (i + 1)..n {
            if absorbed.contains_key(&bodies[j].0) {
                continue;
            }
            let d = (bodies[i].2 - bodies[j].2).length();
            if d >= bodies[i].4 + bodies[j].4 {
                continue;
            }
            let (big, small) = if bodies[i].1 >= bodies[j].1 {
                (i, j)
            } else {
                (j, i)
            };
            let (m1, p1, v1, r1) =
                (bodies[big].1, bodies[big].2, bodies[big].3, bodies[big].4);
            let (m2, v2, r2) =
                (bodies[small].1, bodies[small].3, bodies[small].4);
            if is_neutron_like(m1, r1) && is_neutron_like(m2, r2) {
                kreq.0.push(p1);
            }
            let (m, v, r) = crate::physics::merge_bodies(
                m1,
                [v1.x, v1.y, v1.z],
                r1,
                m2,
                [v2.x, v2.y, v2.z],
                r2,
            );
            bodies[big] =
                (bodies[big].0, m, p1, DVec3::new(v[0], v[1], v[2]), r);
            absorbed.insert(bodies[small].0, bodies[big].0);
        }
    }

    if absorbed.is_empty() {
        return;
    }

    for i in 0..n {
        let e = bodies[i].0;
        if absorbed.contains_key(&e) {
            continue;
        }
        if let Ok((_, mut g, mut s)) = q.get_mut(e) {
            g.mass = bodies[i].1;
            g.vel = bodies[i].3;
            s.radius = bodies[i].4;
        }
    }

    for (gone, mut survivor) in absorbed.clone() {
        while let Some(next) = absorbed.get(&survivor) {
            survivor = *next;
        }
        if selected.0 == Some(gone) {
            selected.0 = Some(survivor);
        }
        commands.entity(gone).despawn();
    }
}

fn record_trails(
    paused: Res<Paused>,
    mut q: Query<(&Grav, &mut Trail, &Visual)>,
) {
    if paused.0 {
        return;
    }
    for (g, mut tr, v) in &mut q {
        if v.star {
            continue;
        }
        tr.pts.push_back(g.pos);
        while tr.pts.len() > TRAIL_CAP {
            tr.pts.pop_front();
        }
    }
}

fn draw_trails(
    cw: Res<CamWorld>,
    mut gizmos: Gizmos,
    q: Query<(&Trail, &Visual)>,
) {
    for (tr, v) in &q {
        if v.star || tr.pts.len() < 2 {
            continue;
        }
        let base = match v.kind {
            PlanetKind::Terran => Color::srgba(0.40, 0.75, 1.0, 0.5),
            PlanetKind::GasBands { .. } => Color::srgba(1.0, 0.78, 0.45, 0.5),
            PlanetKind::IceGiant { .. } => Color::srgba(0.5, 0.85, 1.0, 0.5),
            PlanetKind::Rocky { .. } => Color::srgba(0.85, 0.6, 0.5, 0.5),
            PlanetKind::Lava => Color::srgba(1.0, 0.45, 0.15, 0.5),
            PlanetKind::Ocean => Color::srgba(0.25, 0.6, 1.0, 0.5),
            PlanetKind::Desert => Color::srgba(0.9, 0.72, 0.4, 0.5),
            PlanetKind::Carbon => Color::srgba(0.5, 0.5, 0.55, 0.5),
        };
        let pts: Vec<Vec3> =
            tr.pts.iter().map(|p| (*p - cw.pos).as_vec3()).collect();
        gizmos.linestrip(pts, base);
    }
}

fn pick_body(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    exag: Res<SizeExaggeration>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<FloatingCam>>,
    bodies: Query<(Entity, &GlobalTransform, &Shape, &Visual)>,
    mut selected: ResMut<Selected>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let over_ui = contexts
        .ctx_mut()
        .map(|c| c.wants_pointer_input())
        .unwrap_or(false);
    if over_ui {
        return;
    }
    let Ok(win) = windows.single() else {
        return;
    };
    let Some(cur) = win.cursor_position() else {
        return;
    };
    let Ok((cam, ctf)) = cam_q.single() else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(ctf, cur) else {
        return;
    };

    let mut best = (f32::MAX, None);
    for (e, gt, s, _v) in &bodies {
        let c = gt.translation();
        let r = (s.radius as f32) * exag.0.max(1.0) * 1.5 + 5.0e6;
        let oc = ray.origin - c;
        let b = oc.dot(*ray.direction);
        let cc = oc.length_squared() - r * r;
        let disc = b * b - cc;
        if disc < 0.0 {
            continue;
        }
        let t = -b - disc.sqrt();
        if t > 0.0 && t < best.0 {
            best = (t, Some(e));
        }
    }
    if let Some(e) = best.1 {
        selected.0 = Some(e);
    }
}

fn draw_grid(
    show: Res<ShowGrid>,
    cw: Res<CamWorld>,
    selected: Res<Selected>,
    mut gizmos: Gizmos,
    bodies: Query<&Grav>,
) {
    if !show.0 {
        return;
    }
    let focus = selected
        .0
        .and_then(|e| bodies.get(e).ok())
        .map(|g| g.pos)
        .unwrap_or(DVec3::ZERO);
    let dist = (cw.pos - focus).length().max(1.0e7);
    let extent = dist * 1.8;
    let n: usize = 26;
    let soft = (0.04 * extent) * (0.04 * extent);

    let masses: Vec<(f64, DVec3)> =
        bodies.iter().map(|g| (g.mass, g.pos)).collect();

    let mut grid: Vec<Vec<(DVec3, f64)>> = Vec::with_capacity(n + 1);
    let mut maxphi = 1.0e-30_f64;
    for i in 0..=n {
        let mut row = Vec::with_capacity(n + 1);
        for j in 0..=n {
            let x = focus.x + (i as f64 / n as f64 - 0.5) * extent;
            let z = focus.z + (j as f64 / n as f64 - 0.5) * extent;
            let mut phi = 0.0;
            for (m, p) in &masses {
                let dx = x - p.x;
                let dz = z - p.z;
                phi += G * m / (dx * dx + dz * dz + soft).sqrt();
            }
            if phi > maxphi {
                maxphi = phi;
            }
            row.push((DVec3::new(x, focus.y, z), phi));
        }
        grid.push(row);
    }

    let depth = 0.32 * extent;
    let warp = |p: DVec3, phi: f64| -> Vec3 {
        let t = (phi / maxphi).clamp(0.0, 1.0);
        (DVec3::new(p.x, p.y - depth * t.powf(0.6), p.z) - cw.pos).as_vec3()
    };
    let shade = |t: f64| {
        let t = t as f32;
        Color::srgb(0.10 + 0.85 * t, 0.55 - 0.2 * t, 0.70 + 0.30 * t)
    };

    for row in &grid {
        let pts: Vec<Vec3> = row.iter().map(|(p, ph)| warp(*p, *ph)).collect();
        let avg = row.iter().map(|(_, ph)| ph).sum::<f64>()
            / row.len() as f64
            / maxphi;
        gizmos.linestrip(pts, shade(avg));
    }
    for j in 0..=n {
        let col: Vec<Vec3> =
            (0..=n).map(|i| warp(grid[i][j].0, grid[i][j].1)).collect();
        let avg = (0..=n).map(|i| grid[i][j].1).sum::<f64>()
            / (n as f64 + 1.0)
            / maxphi;
        gizmos.linestrip(col, shade(avg));
    }
}

fn draw_selection(
    selected: Res<Selected>,
    exag: Res<SizeExaggeration>,
    mut gizmos: Gizmos,
    q: Query<(&GlobalTransform, &Shape)>,
) {
    let Some(e) = selected.0 else {
        return;
    };
    let Ok((gt, s)) = q.get(e) else {
        return;
    };
    let c = gt.translation();
    let r = (s.radius as f32) * exag.0.max(1.0) * 1.35 + 2.0e6;
    let accent = Color::srgb(0.0, 0.78, 0.86);
    gizmos.sphere(Isometry3d::from_translation(c), r, accent);
}

fn sun_light(
    selected: Res<Selected>,
    gravs: Query<&Grav>,
    stars: Query<(&Grav, &Visual)>,
    mut light: Query<&mut Transform, With<DirectionalLight>>,
) {
    let star_pos = stars
        .iter()
        .find(|(_, v)| v.star)
        .map(|(g, _)| g.pos)
        .unwrap_or(DVec3::ZERO);
    let body_pos = selected
        .0
        .and_then(|e| gravs.get(e).ok())
        .map(|g| g.pos)
        .unwrap_or(DVec3::new(1.0, 0.0, 0.0));
    let dir = (body_pos - star_pos).normalize_or_zero().as_vec3();
    let dir = if dir.length_squared() < 1.0e-6 {
        Vec3::NEG_X
    } else {
        dir
    };
    let up = if dir.abs_diff_eq(Vec3::Y, 0.01) {
        Vec3::Z
    } else {
        Vec3::Y
    };
    if let Ok(mut tf) = light.single_mut() {
        *tf = Transform::IDENTITY.looking_to(dir, up);
    }
}

fn rotate_bodies(
    time: Res<Time>,
    speed: Res<SimSpeed>,
    paused: Res<Paused>,
    mut q: Query<(&Spin, &mut Transform)>,
) {
    if paused.0 {
        return;
    }
    let sim_dt = (time.delta().as_secs_f64() * speed.0) as f32;
    for (spin, mut tf) in &mut q {
        tf.rotate_y(spin.0 * sim_dt);
    }
}

fn class_material(
    c: BodyClass,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Handle<StandardMaterial> {
    let (_, _, lum, _, kind) = c.props();
    if is_black_hole(c) {
        materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::BLACK,
            unlit: true,
            ..default()
        })
    } else if lum {
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: c.emissive(),
            emissive_texture: Some(crate::planetgen::make_star_texture(
                images, 3,
            )),
            ..default()
        })
    } else {
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(make_texture(images, kind, 700)),
            perceptual_roughness: 0.9,
            ..default()
        })
    }
}

fn handle_spawn(
    mut req: ResMut<SpawnRequest>,
    nbt: Res<NewBodyType>,
    selected: Res<Selected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut jet_mats: ResMut<Assets<JetMat>>,
    mut disk_mats: ResMut<Assets<DiskMat>>,
    mut comet_mats: ResMut<Assets<CometMat>>,
    grid: Query<(Entity, &Grid)>,
    gravs: Query<&Grav>,
) {
    if !req.0 {
        return;
    }
    req.0 = false;

    let Ok((grid_entity, grid)) = grid.single() else {
        return;
    };
    let Some(sel) = selected.0 else {
        return;
    };
    let Ok(target) = gravs.get(sel) else {
        return;
    };

    let class = nbt.0;
    let (mass, radius, lum, _t, kind) = class.props();
    let pos = target.pos + DVec3::new(MOON_ORBIT_M * 3.0, 0.0, 0.0);
    let v = (G * SUN_MASS / pos.length()).sqrt();
    let vel = DVec3::new(0.0, 0.0, v);
    let (cell, off) = grid.translation_to_grid(pos);

    let mesh = meshes.add(Sphere::new(radius as f32).mesh().uv(32, 20));
    let mat = class_material(class, &mut materials, &mut images);

    let body = commands
        .spawn((
            Grav { mass, pos, vel },
            Shape {
                base_radius: radius,
                radius,
            },
            Visual {
                kind,
                star: lum,
                atmo: None,
                rings: false,
                emissive: class.emissive(),
            },
            Trail::default(),
            Name::new(class.label()),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(off),
            cell,
        ))
        .id();
    commands.entity(grid_entity).add_child(body);

    if class == BodyClass::Comet {
        let reach = spawn_comet_rig(
            &mut commands,
            grid_entity,
            grid,
            body,
            pos,
            &mut meshes,
            &mut comet_mats,
            radius,
        );
        commands.entity(body).insert(FocusRadius(reach));
    } else if is_exotic(class) {
        let reach = spawn_exotic_rig(
            &mut commands,
            grid_entity,
            grid,
            body,
            pos,
            &mut meshes,
            &mut jet_mats,
            &mut disk_mats,
            class,
        );
        commands.entity(body).insert(FocusRadius(reach));
    }
    if class == BodyClass::Magnetar {
        commands.entity(body).insert(Magnetar::default());
    }
    if is_main_seq(mass, radius, lum) {
        commands.entity(body).insert(Star {
            age: 0.0,
            t_ms: ms_lifetime(mass),
            stage: 0,
        });
    }
}

fn chaos(
    mut req: ResMut<ChaosReq>,
    mut selected: ResMut<Selected>,
    mut boom: ResMut<Boom>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut sn_mats: ResMut<Assets<crate::snmat::SupernovaMat>>,
    grid: Query<(Entity, &Grid)>,
    mut q: Query<(Entity, &mut Grav, &Shape)>,
) {
    let Some(action) = req.0 else {
        return;
    };
    req.0 = None;
    boom.amt = match action {
        Chaos::Shatter
        | Chaos::GiantImpact
        | Chaos::Vaporize => 1.0,
        Chaos::Barrage | Chaos::CometSwarm => 0.65,
        Chaos::ScatterAll | Chaos::Freeze | Chaos::Reverse => {
            0.45
        }
        Chaos::RogueBh | Chaos::StarFall => 0.4,
        _ => 0.5,
    };
    boom.col = match action {
        Chaos::RogueBh => [0.55, 0.6, 1.0],
        Chaos::ScatterAll | Chaos::Freeze | Chaos::Reverse => {
            [0.6, 0.85, 1.0]
        }
        _ => [1.0, 0.72, 0.3],
    };
    let Ok((grid_e, g)) = grid.single() else {
        return;
    };
    let star = q.iter().fold(
        (0.0_f64, DVec3::ZERO),
        |a, (_, gr, _)| {
            if gr.mass > a.0 {
                (gr.mass, gr.pos)
            } else {
                a
            }
        },
    );
    let mut rng = Lcg(time.elapsed().as_nanos() as u64 | 1);
    let sel = selected.0;
    let target = sel
        .and_then(|e| q.get(e).ok())
        .map(|(_, gr, s)| (gr.pos, gr.vel, gr.mass, s.base_radius));

    match action {
        Chaos::Kick => {
            if let Some(e) = sel {
                if let Ok((_, mut gr, _)) = q.get_mut(e) {
                    let sp = gr.vel.length().max(3.0e4);
                    let dir = DVec3::new(
                        rng.u() - 0.5,
                        rng.u() - 0.5,
                        rng.u() - 0.5,
                    )
                    .normalize_or_zero();
                    gr.vel += dir * sp * 3.0;
                }
            }
        }
        Chaos::Shatter => {
            let (Some(e), Some((p, v, m, r))) = (sel, target)
            else {
                return;
            };
            commands.entity(e).despawn();
            {
                let fm = meshes.add(
                    Sphere::new((r * 2.0).max(4.0e8) as f32)
                        .mesh()
                        .ico(5)
                        .unwrap(),
                );
                let fmat = sn_mats.add(
                    crate::snmat::SupernovaMat {
                        params: crate::snmat::SnParams {
                            c0: Vec4::new(1.0, 0.92, 0.6, 1.0),
                            c1: Vec4::new(1.0, 0.32, 0.08, 1.0),
                            progress: 0.0,
                            seed: 3.0,
                            mode: 1.0,
                            _b: 0.0,
                        },
                    },
                );
                let (fc, fo) = g.translation_to_grid(p);
                commands.entity(grid_e).with_children(|pp| {
                    pp.spawn((
                        SolarScene,
                        Flash {
                            age: 0.0,
                            life: 1.6,
                            expand: 28.0,
                            c0: LinearRgba::rgb(1.0, 0.92, 0.6),
                            c1: LinearRgba::rgb(1.0, 0.32, 0.08),
                        },
                        Mesh3d(fm),
                        MeshMaterial3d(fmat),
                        Transform::from_translation(fo),
                        fc,
                    ));
                });
            }
            let nf = 18usize;
            let mf = m / nf as f64;
            let rf = (r / (nf as f64).cbrt()).max(8.0e4);
            let vesc =
                (2.0 * G * m.max(1.0) / r.max(1.0)).sqrt();
            let mesh =
                meshes.add(Sphere::new(rf as f32).mesh().uv(12, 8));
            let mat = class_material(
                BodyClass::Rocky,
                &mut materials,
                &mut images,
            );
            for _ in 0..nf {
                let d = DVec3::new(
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                )
                .normalize_or_zero();
                let fp = p + d * (r * 1.5);
                let fv = v
                    + d * vesc * (0.8 + 0.9 * rng.u());
                let (c, o) = g.translation_to_grid(fp);
                let f = commands
                    .spawn((
                        Grav {
                            mass: mf,
                            pos: fp,
                            vel: fv,
                        },
                        Shape {
                            base_radius: rf,
                            radius: rf,
                        },
                        Visual {
                            kind: PlanetKind::Rocky {
                                r: 0.55,
                                g: 0.42,
                                b: 0.36,
                            },
                            star: false,
                            atmo: None,
                            rings: false,
                            emissive: LinearRgba::BLACK,
                        },
                        Debris,
                        Trail::default(),
                        Name::new("Debris"),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(o),
                        c,
                    ))
                    .id();
                commands.entity(grid_e).add_child(f);
            }
        }
        Chaos::RogueBh => {
            let (m, radius, _, _, _) =
                BodyClass::BlackHole.props();
            let dir = DVec3::new(
                rng.u() - 0.5,
                (rng.u() - 0.5) * 0.3,
                rng.u() - 0.5,
            )
            .normalize_or_zero();
            let p = star.1 + dir * 8.0e12;
            let toward = (star.1 - p).normalize_or_zero();
            let vel = toward * 5.0e5;
            let mesh = meshes.add(
                Sphere::new(radius as f32).mesh().uv(20, 14),
            );
            let mat = class_material(
                BodyClass::BlackHole,
                &mut materials,
                &mut images,
            );
            let (c, o) = g.translation_to_grid(p);
            let b = commands
                .spawn((
                    Grav {
                        mass: m,
                        pos: p,
                        vel,
                    },
                    Shape {
                        base_radius: radius,
                        radius,
                    },
                    Visual {
                        kind: PlanetKind::Rocky {
                            r: 0.01,
                            g: 0.01,
                            b: 0.02,
                        },
                        star: false,
                        atmo: None,
                        rings: false,
                        emissive: LinearRgba::BLACK,
                    },
                    Trail::default(),
                    Name::new("Rogue black hole"),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_translation(o),
                    c,
                ))
                .id();
            commands.entity(grid_e).add_child(b);
            selected.0 = Some(b);
        }
        Chaos::Barrage => {
            let Some((tp, tv, _, _)) = target else {
                return;
            };
            let mesh =
                meshes.add(Sphere::new(5.0e5).mesh().uv(10, 8));
            let mat = class_material(
                BodyClass::Rocky,
                &mut materials,
                &mut images,
            );
            for _ in 0..12 {
                let d = DVec3::new(
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                )
                .normalize_or_zero();
                let p = tp + d * 6.0e10;
                let vel = tv
                    + (tp - p).normalize_or_zero() * 8.0e4;
                let (c, o) = g.translation_to_grid(p);
                let r = commands
                    .spawn((
                        Grav {
                            mass: 2.0e20,
                            pos: p,
                            vel,
                        },
                        Shape {
                            base_radius: 5.0e5,
                            radius: 5.0e5,
                        },
                        Visual {
                            kind: PlanetKind::Rocky {
                                r: 0.5,
                                g: 0.45,
                                b: 0.4,
                            },
                            star: false,
                            atmo: None,
                            rings: false,
                            emissive: LinearRgba::BLACK,
                        },
                        Trail::default(),
                        Name::new("Impactor"),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(o),
                        c,
                    ))
                    .id();
                commands.entity(grid_e).add_child(r);
            }
        }
        Chaos::ScatterAll => {
            for (_, mut gr, _) in &mut q {
                let sp = gr.vel.length().max(5.0e3);
                let d = DVec3::new(
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                    rng.u() - 0.5,
                )
                .normalize_or_zero();
                gr.vel += d * sp * (0.6 + 1.4 * rng.u());
            }
        }
        Chaos::Freeze => {
            for (_, mut gr, _) in &mut q {
                gr.vel = DVec3::ZERO;
            }
        }
        Chaos::Reverse => {
            for (_, mut gr, _) in &mut q {
                gr.vel = -gr.vel;
            }
        }
        Chaos::Vaporize => {
            if let Some(e) = sel {
                if let Some((p, _, _, r)) = target {
                    let fm = meshes.add(
                        Sphere::new(
                            (r * 2.0).max(4.0e8) as f32,
                        )
                        .mesh()
                        .ico(5)
                        .unwrap(),
                    );
                    let fmat = sn_mats.add(
                        crate::snmat::SupernovaMat {
                            params: crate::snmat::SnParams {
                                c0: Vec4::new(
                                    0.8, 0.95, 1.0, 1.0,
                                ),
                                c1: Vec4::new(
                                    0.4, 0.7, 1.0, 1.0,
                                ),
                                progress: 0.0,
                                seed: 5.0,
                                mode: 0.0,
                                _b: 0.0,
                            },
                        },
                    );
                    let (fc, fo) =
                        g.translation_to_grid(p);
                    commands
                        .entity(grid_e)
                        .with_children(|pp| {
                            pp.spawn((
                                SolarScene,
                                Flash {
                                    age: 0.0,
                                    life: 1.0,
                                    expand: 22.0,
                                    c0: LinearRgba::rgb(
                                        0.8, 0.95, 1.0,
                                    ),
                                    c1: LinearRgba::rgb(
                                        0.4, 0.7, 1.0,
                                    ),
                                },
                                Mesh3d(fm),
                                MeshMaterial3d(fmat),
                                Transform::from_translation(
                                    fo,
                                ),
                                fc,
                            ));
                        });
                }
                commands.entity(e).despawn();
            }
        }
        Chaos::Clone => {
            if let Some((p, v, m, r)) = target {
                let mesh = meshes
                    .add(Sphere::new(r as f32).mesh().uv(24, 16));
                let mat = class_material(
                    BodyClass::Rocky,
                    &mut materials,
                    &mut images,
                );
                let off = DVec3::new(r * 6.0, 0.0, r * 6.0);
                let (c, o) = g.translation_to_grid(p + off);
                let b = commands
                    .spawn((
                        Grav {
                            mass: m,
                            pos: p + off,
                            vel: v
                                + DVec3::new(0.0, 0.0, 1.0e3),
                        },
                        Shape {
                            base_radius: r,
                            radius: r,
                        },
                        Visual {
                            kind: PlanetKind::Rocky {
                                r: 0.55,
                                g: 0.5,
                                b: 0.46,
                            },
                            star: false,
                            atmo: None,
                            rings: false,
                            emissive: LinearRgba::BLACK,
                        },
                        Trail::default(),
                        Name::new("Clone"),
                        Mesh3d(mesh),
                        MeshMaterial3d(mat),
                        Transform::from_translation(o),
                        c,
                    ))
                    .id();
                commands.entity(grid_e).add_child(b);
                selected.0 = Some(b);
            }
        }
        Chaos::GiantImpact => {
            if let Some((tp, tv, _, tr)) = target {
                let im = 6.4e23;
                let ir = 3.4e6;
                let d = DVec3::new(
                    rng.u() - 0.5,
                    (rng.u() - 0.5) * 0.2,
                    rng.u() - 0.5,
                )
                .normalize_or_zero();
                let p = tp + d * (tr.max(ir) * 60.0);
                let vel = tv
                    + (tp - p).normalize_or_zero() * 2.0e4;
                let mesh = meshes.add(
                    Sphere::new(ir as f32).mesh().uv(24, 16),
                );
                let mat = class_material(
                    BodyClass::Rocky,
                    &mut materials,
                    &mut images,
                );
                let (c, o) = g.translation_to_grid(p);
                let b = commands
                    .spawn((
                        Grav {
                            mass: im,
                            pos: p,
                            vel,
                        },
                        Shape {
                            base_radius: ir,
                            radius: ir,
                        },
                        Visual {
                            kind: PlanetKind::Rocky {
                                r: 0.6,
                                g: 0.4,
                                b: 0.32,
                            },
                            star: false,
                            atmo: None,
                            rings: false,
                            emissive: LinearRgba::BLACK,
                        },
                        Trail::default(),
                        Name::new("Theia"),
                        Mesh3d(mesh),
                        MeshMaterial3d(mat),
                        Transform::from_translation(o),
                        c,
                    ))
                    .id();
                commands.entity(grid_e).add_child(b);
                selected.0 = Some(b);
            }
        }
        Chaos::StarFall => {
            let (m, radius, _, _, _) =
                BodyClass::SunLike.props();
            let d = DVec3::new(
                rng.u() - 0.5,
                (rng.u() - 0.5) * 0.4,
                rng.u() - 0.5,
            )
            .normalize_or_zero();
            let p = star.1 + d * 6.0e12;
            let vel =
                (star.1 - p).normalize_or_zero() * 1.5e5;
            let mesh = meshes.add(
                Sphere::new(radius as f32).mesh().uv(28, 18),
            );
            let mat = class_material(
                BodyClass::SunLike,
                &mut materials,
                &mut images,
            );
            let (c, o) = g.translation_to_grid(p);
            let b = commands
                .spawn((
                    Grav {
                        mass: m,
                        pos: p,
                        vel,
                    },
                    Shape {
                        base_radius: radius,
                        radius,
                    },
                    Visual {
                        kind: PlanetKind::Terran,
                        star: true,
                        atmo: None,
                        rings: false,
                        emissive: BodyClass::SunLike.emissive(),
                    },
                    Trail::default(),
                    Name::new("Intruder star"),
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_translation(o),
                    c,
                ))
                .id();
            commands.entity(grid_e).add_child(b);
            selected.0 = Some(b);
        }
        Chaos::CometSwarm => {
            let mesh =
                meshes.add(Sphere::new(4.0e3).mesh().uv(8, 6));
            let mat = class_material(
                BodyClass::Comet,
                &mut materials,
                &mut images,
            );
            for _ in 0..16 {
                let d = DVec3::new(
                    rng.u() - 0.5,
                    (rng.u() - 0.5) * 0.6,
                    rng.u() - 0.5,
                )
                .normalize_or_zero();
                let p = star.1 + d * 7.0e12;
                let aim = star.1
                    + DVec3::new(
                        (rng.u() - 0.5) * 4.0e11,
                        0.0,
                        (rng.u() - 0.5) * 4.0e11,
                    );
                let vel =
                    (aim - p).normalize_or_zero() * 6.0e4;
                let (c, o) = g.translation_to_grid(p);
                let b = commands
                    .spawn((
                        Grav {
                            mass: 2.2e13,
                            pos: p,
                            vel,
                        },
                        Shape {
                            base_radius: 4.0e3,
                            radius: 4.0e3,
                        },
                        Visual {
                            kind: PlanetKind::Rocky {
                                r: 0.5,
                                g: 0.6,
                                b: 0.7,
                            },
                            star: false,
                            atmo: None,
                            rings: false,
                            emissive: LinearRgba::BLACK,
                        },
                        Trail::default(),
                        Name::new("Comet"),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(o),
                        c,
                    ))
                    .id();
                commands.entity(grid_e).add_child(b);
            }
        }
    }
}

fn convert_selected(
    mut req: ResMut<ConvertReq>,
    selected: Res<Selected>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut jet_mats: ResMut<Assets<JetMat>>,
    mut disk_mats: ResMut<Assets<DiskMat>>,
    mut comet_mats: ResMut<Assets<CometMat>>,
    grid: Query<(Entity, &Grid)>,
    rigs: Query<(Entity, &JetRig)>,
    comet_rigs: Query<(Entity, &CometRig)>,
    mut q: Query<(
        &mut Grav,
        &mut Shape,
        &mut Visual,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(c) = req.0.take() else {
        return;
    };
    let Some(e) = selected.0 else {
        return;
    };
    let Ok((grid_entity, grid)) = grid.single() else {
        return;
    };
    let Ok((mut g, mut s, mut v, mut mm)) = q.get_mut(e) else {
        return;
    };
    let (mass, radius, lum, _t, kind) = c.props();
    g.mass = mass;
    s.radius = radius;
    v.kind = kind;
    v.star = lum;
    v.emissive = c.emissive();
    mm.0 = class_material(c, &mut materials, &mut images);
    let pos = g.pos;

    for (re, rig) in &rigs {
        if rig.body == e {
            commands.entity(re).despawn();
        }
    }
    for (re, rig) in &comet_rigs {
        if rig.body == e {
            commands.entity(re).despawn();
        }
    }
    if c == BodyClass::Comet {
        let reach = spawn_comet_rig(
            &mut commands,
            grid_entity,
            grid,
            e,
            pos,
            &mut meshes,
            &mut comet_mats,
            radius,
        );
        commands.entity(e).insert(FocusRadius(reach));
    } else if is_exotic(c) {
        let reach = spawn_exotic_rig(
            &mut commands,
            grid_entity,
            grid,
            e,
            pos,
            &mut meshes,
            &mut jet_mats,
            &mut disk_mats,
            c,
        );
        commands.entity(e).insert(FocusRadius(reach));
    } else {
        commands.entity(e).remove::<FocusRadius>();
    }
    if c == BodyClass::Magnetar {
        commands.entity(e).insert(Magnetar::default());
    } else {
        commands.entity(e).remove::<Magnetar>();
    }
    if is_main_seq(mass, radius, lum) {
        commands.entity(e).insert(Star {
            age: 0.0,
            t_ms: ms_lifetime(mass),
            stage: 0,
        });
    } else {
        commands.entity(e).remove::<Star>();
    }
}

fn supernova(
    mut req: ResMut<SupernovaReq>,
    mut sn_mats: ResMut<Assets<crate::snmat::SupernovaMat>>,
    mut selected: ResMut<Selected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    grid: Query<(Entity, &Grid)>,
    mut q: Query<(
        &mut Grav,
        &mut Shape,
        &mut Visual,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(e) = req.0.take() else {
        return;
    };
    let Ok((grid_e, g_grid)) = grid.single() else {
        return;
    };
    let Ok((mut g, mut s, mut v, mut mm)) = q.get_mut(e) else {
        return;
    };

    // Make sure the player is looking at the explosion.
    selected.0 = Some(e);

    let msun = g.mass / SUN_MASS;
    let r0 = (s.radius as f32).max(5.0e8);
    let star_pos = g.pos;

    if msun >= 20.0 {
        g.mass = (g.mass * 0.5).max(6.0 * SUN_MASS);
        s.radius = 2.0e5;
        v.star = false;
        v.emissive = LinearRgba::BLACK;
        v.kind = PlanetKind::Rocky { r: 0.02, g: 0.02, b: 0.03 };
        mm.0 = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            perceptual_roughness: 1.0,
            ..default()
        });
    } else if msun >= 8.0 {
        let c = BodyClass::NeutronStar;
        let (_, radius, _, _, kind) = c.props();
        g.mass = 1.4 * SUN_MASS;
        s.radius = radius;
        v.star = true;
        v.kind = kind;
        v.emissive = c.emissive();
        mm.0 = class_material(c, &mut materials, &mut images);
    } else {
        let c = BodyClass::WhiteDwarf;
        let (mass, radius, _, _, kind) = c.props();
        g.mass = mass;
        s.radius = radius;
        v.star = true;
        v.kind = kind;
        v.emissive = c.emissive();
        mm.0 = class_material(c, &mut materials, &mut images);
    }

    let (cell, off) = g_grid.translation_to_grid(star_pos);
    let mesh = meshes.add(Sphere::new(r0).mesh().ico(6).unwrap());
    // (life, expand, hot c0, cool c1, seed): core flash, shock shell, ejecta
    // (life, expand, hot c0, cool c1, seed, mode): 0 core, 1 shock, 2 ejecta
    let layers = [
        (0.9, 10.0, Vec4::new(1.0, 0.98, 0.95, 1.0), Vec4::new(0.55, 0.7, 1.0, 1.0), 1.3, 0.0),
        (3.6, 220.0, Vec4::new(0.8, 0.9, 1.0, 1.0), Vec4::new(1.0, 0.55, 0.18, 1.0), 5.7, 1.0),
        (5.5, 95.0, Vec4::new(1.0, 0.5, 0.2, 1.0), Vec4::new(0.30, 0.05, 0.04, 1.0), 9.1, 2.0),
    ];
    commands.entity(grid_e).with_children(|p| {
        for (life, expand, c0, c1, seed, mode) in layers {
            let mat = sn_mats.add(crate::snmat::SupernovaMat {
                params: crate::snmat::SnParams {
                    c0,
                    c1,
                    progress: 0.0,
                    seed,
                    mode,
                    _b: 0.0,
                },
            });
            p.spawn((
                SolarScene,
                Flash {
                    age: 0.0,
                    life,
                    expand,
                    c0: LinearRgba::rgb(c0.x, c0.y, c0.z),
                    c1: LinearRgba::rgb(c1.x, c1.y, c1.z),
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(off),
                cell,
            ));
        }
    });
}

fn stellar_evolution(
    time: Res<Time>,
    speed: Res<SimSpeed>,
    paused: Res<Paused>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut snreq: ResMut<SupernovaReq>,
    mut q: Query<(
        Entity,
        &Grav,
        &mut Shape,
        &mut Visual,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut Star,
    )>,
) {
    if paused.0 {
        return;
    }
    let sim_dt = time.delta().as_secs_f64() * speed.0;
    if sim_dt <= 0.0 {
        return;
    }
    for (e, g, mut s, mut v, mut mm, mut st) in &mut q {
        st.age += sim_dt;
        if st.stage == 0 && st.age >= st.t_ms {
            let c = BodyClass::RedGiant;
            let (_, radius, _, _, kind) = c.props();
            s.radius = radius;
            v.kind = kind;
            v.emissive = c.emissive();
            mm.0 = class_material(c, &mut materials, &mut images);
            st.stage = 1;
            st.t_ms = st.age + ms_lifetime(g.mass) * 0.1;
        } else if st.stage == 1 && st.age >= st.t_ms {
            if g.mass >= 8.0 * SUN_MASS {
                if snreq.0.is_none() {
                    snreq.0 = Some(e);
                    commands.entity(e).remove::<Star>();
                }
            } else {
                let c = BodyClass::WhiteDwarf;
                let (_, radius, _, _, kind) = c.props();
                s.radius = radius;
                v.kind = kind;
                v.star = true;
                v.emissive = c.emissive();
                mm.0 = class_material(c, &mut materials, &mut images);
                commands.entity(e).remove::<Star>();
            }
        }
    }
}

fn kilonova(
    mut req: ResMut<KilonovaReq>,
    mut sn_mats: ResMut<Assets<crate::snmat::SupernovaMat>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    grid: Query<(Entity, &Grid)>,
) {
    if req.0.is_empty() {
        return;
    }
    let Ok((grid_e, g_grid)) = grid.single() else {
        req.0.clear();
        return;
    };
    let positions: Vec<DVec3> = req.0.drain(..).collect();
    let mesh = meshes.add(Sphere::new(3.0e8).mesh().ico(6).unwrap());
    let layers = [
        (0.6, 9.0, Vec4::new(0.85, 0.92, 1.0, 1.0),
            Vec4::new(0.55, 0.75, 1.0, 1.0), 2.1, 0.0),
        (2.0, 150.0, Vec4::new(0.65, 0.82, 1.0, 1.0),
            Vec4::new(0.95, 0.45, 0.30, 1.0), 5.3, 1.0),
        (4.2, 80.0, Vec4::new(1.0, 0.40, 0.18, 1.0),
            Vec4::new(0.35, 0.04, 0.06, 1.0), 8.8, 2.0),
    ];
    for pos in positions {
        let (cell, off) = g_grid.translation_to_grid(pos);
        commands.entity(grid_e).with_children(|p| {
            for (life, expand, c0, c1, seed, mode) in layers {
                let mat = sn_mats.add(crate::snmat::SupernovaMat {
                    params: crate::snmat::SnParams {
                        c0,
                        c1,
                        progress: 0.0,
                        seed,
                        mode,
                        _b: 0.0,
                    },
                });
                p.spawn((
                    SolarScene,
                    Flash {
                        age: 0.0,
                        life,
                        expand,
                        c0: LinearRgba::rgb(c0.x, c0.y, c0.z),
                        c1: LinearRgba::rgb(c1.x, c1.y, c1.z),
                    },
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat),
                    Transform::from_translation(off),
                    cell,
                ));
            }
        });
    }
}

fn flash_fx(
    time: Res<Time>,
    mut commands: Commands,
    mut sn_mats: ResMut<Assets<crate::snmat::SupernovaMat>>,
    mut q: Query<(
        Entity,
        &mut Flash,
        &mut Transform,
        &MeshMaterial3d<crate::snmat::SupernovaMat>,
    )>,
) {
    let dt = time.delta_secs();
    for (e, mut f, mut tf, mm) in &mut q {
        f.age += dt;
        let k = (f.age / f.life).clamp(0.0, 1.0);
        // fast burst then easing growth
        let grow = 1.0 - (1.0 - k).powf(2.4);
        tf.scale = Vec3::splat(1.0 + grow * f.expand);
        if let Some(m) = sn_mats.get_mut(&mm.0) {
            m.params.progress = k;
        }
        if f.age >= f.life {
            commands.entity(e).despawn();
        }
    }
}

fn drag_create(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    cw: Res<CamWorld>,
    sel: Res<Selected>,
    nbt: Res<NewBodyType>,
    gravs: Query<&Grav>,
    grid: Query<(Entity, &Grid)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut drag: Local<Vec2>,
) {
    let over_ui = contexts
        .ctx_mut()
        .map(|c| c.wants_pointer_input())
        .unwrap_or(false);

    if buttons.pressed(MouseButton::Right) && !over_ui {
        *drag += motion.delta;
    }
    if !buttons.just_released(MouseButton::Right) {
        return;
    }
    let d = *drag;
    *drag = Vec2::ZERO;

    let Ok((groot, grid)) = grid.single() else {
        return;
    };
    let base_vel = sel
        .0
        .and_then(|e| gravs.get(e).ok())
        .map(|g| g.vel)
        .unwrap_or(DVec3::ZERO);

    let class = nbt.0;
    let (mass, radius, lum, _t, kind) = class.props();
    let dist = 2.0e8_f64;
    let pos = cw.pos + cw.fwd * dist;
    let vscale = 40.0;
    let vel = base_vel
        + cw.right * (d.x as f64 * vscale)
        + cw.up * (-d.y as f64 * vscale);
    let (cell, off) = grid.translation_to_grid(pos);
    let mat = class_material(class, &mut materials, &mut images);

    commands.entity(groot).with_children(|p| {
        p.spawn((
            Grav { mass, pos, vel },
            Name::new(class.label()),
            Shape {
                base_radius: radius,
                radius,
            },
            Visual {
                kind,
                star: lum,
                atmo: None,
                rings: false,
                emissive: class.emissive(),
            },
            Trail::default(),
            Mesh3d(meshes.add(Sphere::new(radius as f32).mesh().uv(32, 20))),
            MeshMaterial3d(mat),
            Transform::from_translation(off),
            cell,
        ));
    });
}

fn enc_atmo(a: &Option<Srgba>) -> String {
    match a {
        Some(c) => format!("{}:{}:{}", c.red, c.green, c.blue),
        None => "-".into(),
    }
}

fn dec_atmo(s: &str) -> Option<Srgba> {
    if s == "-" {
        return None;
    }
    let p: Vec<f32> = s.split(':').filter_map(|v| v.parse().ok()).collect();
    if p.len() == 3 {
        Some(Srgba::new(p[0], p[1], p[2], 1.0))
    } else {
        None
    }
}

fn save_scene(mut req: ResMut<SaveReq>, q: Query<(&Name, &Grav, &Shape, &Visual)>) {
    if !req.0 {
        return;
    }
    req.0 = false;
    let mut out = String::new();
    for (n, g, sh, v) in &q {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}:{}:{}\n",
            n.as_str(),
            v.star as u8,
            g.mass,
            g.pos.x,
            g.pos.y,
            g.pos.z,
            g.vel.x,
            g.vel.y,
            g.vel.z,
            sh.base_radius,
            v.kind.to_code(),
            v.rings as u8,
            enc_atmo(&v.atmo),
            v.emissive.red,
            v.emissive.green,
            v.emissive.blue
        ));
    }
    let _ = std::fs::write("scenario.txt", out);
}

fn load_scene(mut req: ResMut<LoadReq>, mut pending: ResMut<PendingLoad>) {
    if !req.0 {
        return;
    }
    req.0 = false;
    let Ok(txt) = std::fs::read_to_string("scenario.txt") else {
        return;
    };
    let mut specs = Vec::new();
    for line in txt.lines() {
        let f: Vec<&str> = line.split('|').collect();
        if f.len() < 14 {
            continue;
        }
        let p = |i: usize| -> f64 { f.get(i).and_then(|v| v.parse().ok()).unwrap_or(0.0) };
        let em: Vec<f32> = f[13].split(':').filter_map(|v| v.parse().ok()).collect();
        let emissive = if em.len() == 3 {
            LinearRgba::rgb(em[0], em[1], em[2])
        } else {
            LinearRgba::BLACK
        };
        specs.push(SpawnSpec {
            name: f[0].to_string(),
            kind: PlanetKind::from_code(f[10]),
            star: f[1] == "1",
            emissive,
            radius: p(9),
            mass: p(2),
            pos: DVec3::new(p(3), p(4), p(5)),
            vel: DVec3::new(p(6), p(7), p(8)),
            atmo: dec_atmo(f[12]),
            rings: f[11] == "1",
        });
    }
    if !specs.is_empty() {
        pending.0 = Some(specs);
    }
}

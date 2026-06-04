mod blackhole;
#[allow(dead_code)]
mod physics;
mod offline;
mod planetgen;
#[allow(dead_code)]
mod scene;
mod sky;
mod auroramat;
mod cometmat;
mod diskmat;
mod jetmat;
mod nebmat;
mod snmat;
mod ui;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::transform::TransformPlugin;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};
use big_space::prelude::BigSpaceDefaultPlugins;

use crate::blackhole::BlackHolePlugin;
use crate::scene::SolarPlugin;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum AppMode {
    #[default]
    Menu,
    BlackHole,
    SolarSystem,
}

#[derive(Component)]
struct MenuScene;

#[derive(Resource)]
struct ShotMode {
    path: String,
    frame: u32,
}

fn arg<T: std::str::FromStr>(args: &[String], key: &str, def: T) -> T {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(def)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--render") {
        let mut p = offline::RenderParams::default();
        if let Some(sz) = args
            .iter()
            .position(|a| a == "--size")
            .and_then(|i| args.get(i + 1))
        {
            if let Some((w, h)) = sz.split_once('x') {
                p.width = w.parse().unwrap_or(p.width);
                p.height = h.parse().unwrap_or(p.height);
            }
        }
        p.samples = arg(&args, "--samples", p.samples);
        p.spin = arg(&args, "--spin", p.spin);
        p.radius = arg(&args, "--radius", p.radius);
        p.yaw = arg(&args, "--yaw", p.yaw);
        p.pitch = arg(&args, "--pitch", p.pitch);
        p.fov_deg = arg(&args, "--fov", p.fov_deg);
        p.temp = arg(&args, "--temp", p.temp);
        p.brightness = arg(&args, "--brightness", p.brightness);
        p.r_out = arg(&args, "--rout", p.r_out);
        p.exposure = arg(&args, "--exposure", p.exposure);
        p.max_steps = arg(&args, "--steps", p.max_steps);
        let out = args
            .iter()
            .position(|a| a == "--render")
            .and_then(|i| args.get(i + 1))
            .filter(|s| !s.starts_with("--"))
            .cloned()
            .unwrap_or_else(|| "render.png".to_string());
        let t = std::time::Instant::now();
        println!(
            "rendering {}x{} x{} samples (CPU, pure Rust)...",
            p.width, p.height, p.samples
        );
        match offline::render(&p, &out) {
            Ok(()) => println!("wrote {out} in {:.1}s", t.elapsed().as_secs_f32()),
            Err(e) => eprintln!("render error: {e}"),
        }
        return;
    }

    let preset_idx: usize = arg(&args, "--preset", 0);

    let initial = if args.iter().any(|a| a == "--solar") {
        AppMode::SolarSystem
    } else if args.iter().any(|a| a == "--bh") {
        AppMode::BlackHole
    } else {
        AppMode::Menu
    };

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Sandbox".into(),
                    ..default()
                }),
                ..default()
            })
            .build()
            .disable::<TransformPlugin>(),
        BigSpaceDefaultPlugins,
    ))
    .add_plugins(FrameTimeDiagnosticsPlugin::default())
    .add_plugins(EguiPlugin::default())
    .insert_state(initial)
    .insert_resource(scene::ActivePreset(preset_idx))
    .insert_resource(scene::SizeExaggeration(arg(&args, "--bigsize", 1.0_f32)))
    .insert_resource(scene::ShowGrid(args.iter().any(|a| a == "--grid")))
    .add_plugins(BlackHolePlugin)
    .add_plugins(SolarPlugin)
    .add_plugins(crate::sky::SkyPlugin)
    .add_plugins(crate::snmat::SnMatPlugin)
    .add_plugins(crate::jetmat::JetMatPlugin)
    .add_plugins(crate::nebmat::NebMatPlugin)
    .add_plugins(crate::diskmat::DiskMatPlugin)
    .add_plugins(crate::cometmat::CometMatPlugin)
    .add_plugins(crate::auroramat::AuroraMatPlugin)
    .insert_resource(ClearColor(Color::srgb(0.004, 0.005, 0.011)))
    .add_systems(PreStartup, disable_egui_autocontext)
    .add_systems(OnEnter(AppMode::Menu), spawn_menu_cam)
    .add_systems(OnExit(AppMode::Menu), despawn_menu_cam)
    .add_systems(Update, (toggle_mode, escape_to_menu))
    .add_systems(EguiPrimaryContextPass, ui::ui_panel);

    if let Some(i) = args.iter().position(|a| a == "--shot") {
        if let Some(path) = args.get(i + 1) {
            app.insert_resource(ShotMode {
                path: path.clone(),
                frame: 0,
            })
            .add_systems(Update, shot_system);
        }
    }
    if let Some(i) = args.iter().position(|a| a == "--bh-style") {
        if let Some(name) = args.get(i + 1) {
            let mut v = crate::blackhole::BhView::default();
            match name.as_str() {
                "interstellar" => {}
                "scientific" => {
                    v.spin = 0.9;
                    v.doppler = 1.0;
                    v.procedural = 1.0;
                    v.glow = 0.18;
                    v.disk_inner = 0.0;
                    v.temp = 3600.0;
                    v.brightness = 0.30;
                    v.r_out = 22.0;
                    v.max_steps = 2600.0;
                }
                "performance" => {
                    v.doppler = 0.0;
                    v.procedural = 0.0;
                    v.glow = 0.4;
                    v.disk_on = 1.0;
                    v.max_steps = 800.0;
                    v.aa = 1.0;
                    v.render_scale = 0.4;
                }
                _ => {}
            }
            app.insert_resource(v);
        }
    }
    if args.iter().any(|a| a == "--sn") {
        app.insert_resource(AutoSn(0)).add_systems(Update, auto_sn);
    }
    if args.iter().any(|a| a == "--io") {
        app.insert_resource(AutoIo(0)).add_systems(Update, auto_io);
    }
    if args.iter().any(|a| a == "--smash") {
        app.insert_resource(AutoSmash(0))
            .add_systems(Update, auto_smash);
    }
    if args.iter().any(|a| a == "--roche") {
        app.insert_resource(AutoRoche(0))
            .add_systems(Update, auto_roche);
    }
    if args.iter().any(|a| a == "--evolve") {
        app.insert_resource(AutoEvolve(0))
            .add_systems(Update, auto_evolve);
    }
    if args.iter().any(|a| a == "--kilo") {
        app.insert_resource(AutoKilo(0)).add_systems(Update, auto_kilo);
    }
    if args.iter().any(|a| a == "--gas") {
        app.insert_resource(AutoGas(0)).add_systems(Update, auto_gas);
    }
    if args.iter().any(|a| a == "--ice") {
        app.insert_resource(AutoIce(0)).add_systems(Update, auto_ice);
    }
    if args.iter().any(|a| a == "--mars") {
        app.insert_resource(AutoMars(0))
            .add_systems(Update, auto_mars);
    }
    if args.iter().any(|a| a == "--belt") {
        app.insert_resource(AutoBelt(0))
            .add_systems(Update, auto_belt);
    }
    if args.iter().any(|a| a == "--prec") {
        app.insert_resource(AutoPrec(0))
            .add_systems(Update, auto_prec);
    }
    if args.iter().any(|a| a == "--res") {
        app.insert_resource(AutoRes(0))
            .add_systems(Update, auto_res);
    }
    if args.iter().any(|a| a == "--esc") {
        app.insert_resource(AutoEsc(0))
            .add_systems(Update, auto_esc);
    }
    if args.iter().any(|a| a == "--sat") {
        app.insert_resource(AutoSat(0))
            .add_systems(Update, auto_sat);
    }
    if args.iter().any(|a| a == "--mt") {
        app.insert_resource(AutoMt(0))
            .add_systems(Update, auto_mt);
    }
    if args.iter().any(|a| a == "--ring") {
        app.insert_resource(AutoRing(0))
            .add_systems(Update, auto_ring);
    }
    if args.iter().any(|a| a == "--star") {
        app.insert_resource(AutoStar(0))
            .add_systems(Update, auto_star);
    }
    if args.iter().any(|a| a == "--neb") {
        app.insert_resource(AutoNeb(0)).add_systems(Update, auto_neb);
    }
    if args.iter().any(|a| a == "--lock") {
        app.insert_resource(AutoLock(0))
            .add_systems(Update, auto_lock);
    }
    if args.iter().any(|a| a == "--subl") {
        app.insert_resource(AutoSubl(0))
            .add_systems(Update, auto_subl);
    }
    if args.iter().any(|a| a == "--gw") {
        app.insert_resource(AutoGw(0)).add_systems(Update, auto_gw);
    }
    if let Some(i) = args.iter().position(|a| a == "--chaos") {
        let k = match args.get(i + 1).map(|s| s.as_str()) {
            Some("rogue") => scene::Chaos::RogueBh,
            Some("barrage") => scene::Chaos::Barrage,
            Some("kick") => scene::Chaos::Kick,
            Some("vaporize") => scene::Chaos::Vaporize,
            Some("clone") => scene::Chaos::Clone,
            Some("impact") => scene::Chaos::GiantImpact,
            Some("starfall") => scene::Chaos::StarFall,
            Some("comets") => scene::Chaos::CometSwarm,
            Some("scatter") => scene::Chaos::ScatterAll,
            Some("freeze") => scene::Chaos::Freeze,
            Some("reverse") => scene::Chaos::Reverse,
            _ => scene::Chaos::Shatter,
        };
        app.insert_resource(AutoChaos(0, k))
            .add_systems(Update, auto_chaos);
    }
    if args.iter().any(|a| a == "--accrete") {
        app.insert_resource(scene::AutoAccrete(0))
            .add_systems(Update, scene::auto_accrete);
    }
    if args.iter().any(|a| a == "--atmo") {
        app.insert_resource(AutoAtmo(0))
            .add_systems(Update, auto_atmo);
    }
    if let Some(i) = args.iter().position(|a| a == "--be") {
        let c = match args.get(i + 1).map(|s| s.as_str()) {
            Some("pulsar") => scene::BodyClass::Pulsar,
            Some("magnetar") => scene::BodyClass::Magnetar,
            Some("quasar") => scene::BodyClass::Quasar,
            Some("blackhole") => scene::BodyClass::BlackHole,
            Some("comet") => scene::BodyClass::Comet,
            Some("ostar") => scene::BodyClass::OStar,
            Some("bluesg") => scene::BodyClass::BlueSupergiant,
            Some("redsg") => scene::BodyClass::RedSupergiant,
            Some("wr") => scene::BodyClass::WolfRayet,
            Some("carbon") => scene::BodyClass::CarbonStar,
            Some("bdwarf") => scene::BodyClass::BrownDwarf,
            Some("lava") => scene::BodyClass::LavaWorld,
            Some("ocean") => scene::BodyClass::OceanWorld,
            Some("desert") => scene::BodyClass::DesertWorld,
            Some("carbonp") => scene::BodyClass::CarbonPlanet,
            Some("imbh") => scene::BodyClass::IntermediateBlackHole,
            Some("smbh") => scene::BodyClass::SupermassiveBlackHole,
            _ => scene::BodyClass::Pulsar,
        };
        app.insert_resource(AutoConv(0, c))
            .add_systems(Update, auto_conv);
    }

    app.run();
}

#[derive(Resource)]
struct AutoSn(u32);

fn auto_sn(
    mut a: ResMut<AutoSn>,
    mut req: ResMut<scene::SupernovaReq>,
    bodies: Query<(Entity, &scene::Grav)>,
) {
    a.0 += 1;
    if a.0 == 50 {
        let mut best = (0.0_f64, None);
        for (e, g) in &bodies {
            if g.mass > best.0 {
                best = (g.mass, Some(e));
            }
        }
        req.0 = best.1;
    }
}

#[derive(Resource)]
struct AutoConv(u32, scene::BodyClass);

fn auto_conv(
    mut a: ResMut<AutoConv>,
    mut req: ResMut<scene::ConvertReq>,
    mut sel: ResMut<scene::Selected>,
    bodies: Query<(Entity, &scene::Grav)>,
) {
    a.0 += 1;
    if a.0 == 6 {
        let mut all: Vec<(f64, Entity)> =
            bodies.iter().map(|(e, g)| (g.mass, e)).collect();
        all.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());
        let lum = a.1.props().2;
        let pick = if !lum {
            all.get(1).or_else(|| all.first())
        } else {
            all.first()
        };
        sel.0 = pick.map(|(_, e)| *e);
        req.0 = Some(a.1);
    }
}

#[derive(Resource)]
struct AutoIo(u32);

fn auto_io(
    mut a: ResMut<AutoIo>,
    mut sel: ResMut<scene::Selected>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Shape)>,
) {
    a.0 += 1;
    if a.0 != 8 {
        return;
    }
    let mut giant = (0.0_f64, bevy::math::DVec3::ZERO,
        bevy::math::DVec3::ZERO, 1.0_f64);
    let mut moon = (f64::MAX, None);
    for (e, g, s) in &q {
        if g.mass < 1.0e29 && g.mass > giant.0 {
            giant = (g.mass, g.pos, g.vel, s.base_radius);
        }
        if g.mass < moon.0 {
            moon = (g.mass, Some(e));
        }
    }
    if let Some(me) = moon.1 {
        let r = giant.3 * 2.3;
        let vc = (6.674e-11 * giant.0 / r).sqrt();
        if let Ok((_, mut g, _)) = q.get_mut(me) {
            g.pos = giant.1 + bevy::math::DVec3::new(r, 0.0, 0.0);
            g.vel = giant.2 + bevy::math::DVec3::new(0.0, 0.0, vc);
        }
        sel.0 = Some(me);
    }
}

#[derive(Resource)]
struct AutoSmash(u32);

fn auto_smash(
    mut a: ResMut<AutoSmash>,
    mut sel: ResMut<scene::Selected>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Shape)>,
) {
    a.0 += 1;
    if a.0 != 8 {
        return;
    }
    let mut v: Vec<(Entity, f64, f64)> = q
        .iter()
        .map(|(e, g, s)| (e, g.mass, s.base_radius))
        .collect();
    v.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
    let planets: Vec<(Entity, f64, f64)> =
        v.into_iter().filter(|t| t.1 < 1.0e29).collect();
    if planets.len() < 2 {
        return;
    }
    let (te, tm, tr) = planets[0];
    let (pe, pm, pr) = planets[1];
    let tp;
    let tv;
    {
        let (_, g, _) = q.get(te).unwrap();
        tp = g.pos;
        tv = g.vel;
    }
    let sep = (tr + pr) * 0.95;
    let vimp = 4.0 * (2.0 * 6.674e-11 * (tm + pm) / (tr + pr)).sqrt();
    if let Ok((_, mut g, _)) = q.get_mut(pe) {
        g.pos = tp + bevy::math::DVec3::new(sep, 0.0, 0.0);
        g.vel = tv + bevy::math::DVec3::new(-vimp, 0.0, 0.0);
    }
    sel.0 = Some(te);
}

#[derive(Resource)]
struct AutoRoche(u32);

fn auto_roche(
    mut a: ResMut<AutoRoche>,
    mut sel: ResMut<scene::Selected>,
    mut q: Query<(Entity, &mut scene::Grav)>,
) {
    a.0 += 1;
    if a.0 != 8 {
        return;
    }
    let mut big = (0.0_f64, bevy::math::DVec3::ZERO, bevy::math::DVec3::ZERO);
    let mut small = (f64::MAX, None);
    for (e, g) in &q {
        if g.mass > big.0 {
            big = (g.mass, g.pos, g.vel);
        }
        if g.mass < small.0 {
            small = (g.mass, Some(e));
        }
    }
    if let Some(se) = small.1 {
        if let Ok((_, mut g)) = q.get_mut(se) {
            let r = 1.0e9;
            g.pos = big.1 + bevy::math::DVec3::new(r, 0.0, 0.0);
            let vc = (6.674e-11 * big.0 / r).sqrt();
            g.vel = big.2 + bevy::math::DVec3::new(0.0, 0.0, vc);
        }
    }
    let mut bm = (0.0_f64, None);
    for (e, g) in &q {
        if g.mass > bm.0 {
            bm = (g.mass, Some(e));
        }
    }
    sel.0 = bm.1;
}

#[derive(Resource)]
struct AutoEvolve(u32);

fn auto_evolve(
    mut a: ResMut<AutoEvolve>,
    mut sel: ResMut<scene::Selected>,
    mut q: Query<(Entity, &scene::Grav, &mut scene::Star)>,
) {
    a.0 += 1;
    if a.0 >= 8 {
        let mut best = (0.0_f64, None);
        for (e, g, mut st) in &mut q {
            st.age += 6.0e8;
            if g.mass > best.0 {
                best = (g.mass, Some(e));
            }
        }
        if a.0 == 8 {
            sel.0 = best.1;
        }
    }
}

#[derive(Resource)]
struct AutoKilo(u32);

fn auto_kilo(
    mut a: ResMut<AutoKilo>,
    mut kreq: ResMut<scene::KilonovaReq>,
    mut sel: ResMut<scene::Selected>,
    bodies: Query<(Entity, &scene::Grav)>,
) {
    a.0 += 1;
    if a.0 == 8 {
        let mut best = (0.0_f64, None, bevy::math::DVec3::ZERO);
        for (e, g) in &bodies {
            if g.mass > best.0 {
                best = (g.mass, Some(e), g.pos);
            }
        }
        sel.0 = best.1;
        kreq.0.push(best.2);
    }
}

#[derive(Resource)]
struct AutoMt(u32);

fn auto_mt(
    mut a: ResMut<AutoMt>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut q: Query<(Entity, &mut scene::Grav, &mut scene::Shape, &mut scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut sun = (0.0_f64, None, bevy::math::DVec3::ZERO);
    for (e, g, _, v) in &q {
        if v.star && g.mass > sun.0 {
            sun = (g.mass, Some(e), g.pos);
        }
    }
    let mut pl = (0.0_f64, None);
    for (e, g, _, v) in &q {
        if !v.star && g.mass > pl.0 {
            pl = (g.mass, Some(e));
        }
    }
    let (Some(ae), Some(de)) = (sun.1, pl.1) else {
        return;
    };
    let ma = 1.989e30_f64;
    let mb = 0.5 * ma;
    let mtot = ma + mb;
    let sep = 4.0e9_f64;
    let g_c = 6.674e-11_f64;
    let vrel = (g_c * mtot / sep).sqrt();
    let c = sun.2;
    if let Ok((_, mut g, mut s, _)) = q.get_mut(ae) {
        g.mass = ma;
        g.pos = c;
        g.vel = bevy::math::DVec3::new(0.0, 0.0, -(mb / mtot) * vrel);
        s.base_radius = 6.96e8;
        s.radius = 6.96e8;
    }
    if let Ok((_, mut g, mut s, mut v)) = q.get_mut(de) {
        v.star = true;
        g.mass = mb;
        g.pos = c + bevy::math::DVec3::new(sep, 0.0, 0.0);
        g.vel = bevy::math::DVec3::new(0.0, 0.0, (ma / mtot) * vrel);
        s.base_radius = 1.7e9;
        s.radius = 1.7e9;
    }
    sel.0 = Some(de);
    speed.0 = 5.0e5;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoSat(u32);

fn auto_sat(
    mut a: ResMut<AutoSat>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &Name)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    for (e, n) in &q {
        if n.as_str() == "Moon" {
            sel.0 = Some(e);
            a.0 = u32::MAX;
            return;
        }
    }
}

#[derive(Resource)]
struct AutoEsc(u32);

fn auto_esc(
    mut a: ResMut<AutoEsc>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut star = (0.0_f64, bevy::math::DVec3::ZERO);
    for (_, g, v) in &q {
        if v.star && g.mass > star.0 {
            star = (g.mass, g.pos);
        }
    }
    if star.0 <= 0.0 {
        return;
    }
    let mut light = (f64::MAX, None);
    for (e, g, v) in &q {
        if !v.star && g.mass < light.0 {
            light = (g.mass, Some(e));
        }
    }
    let Some(te) = light.1 else {
        return;
    };
    let mu = 6.674e-11 * star.0;
    if let Ok((_, mut g, _)) = q.get_mut(te) {
        let r = (g.pos - star.1).length().max(1.0);
        let vesc = (2.0 * mu / r).sqrt();
        let dir = if g.vel.length() > 1.0 {
            g.vel.normalize()
        } else {
            bevy::math::DVec3::new(0.0, 0.0, 1.0)
        };
        g.vel = dir * (1.4 * vesc);
    }
    sel.0 = Some(te);
    speed.0 = 2.0e5;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoRes(u32);

fn auto_res(
    mut a: ResMut<AutoRes>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut star = (0.0_f64, bevy::math::DVec3::ZERO);
    for (_, g, v) in &q {
        if v.star && g.mass > star.0 {
            star = (g.mass, g.pos);
        }
    }
    if star.0 <= 0.0 {
        return;
    }
    let mut picks: Vec<Entity> = q
        .iter()
        .filter(|(_, g, v)| !v.star && g.mass < 1.0e29)
        .map(|(e, _, _)| e)
        .take(2)
        .collect();
    if picks.len() < 2 {
        return;
    }
    let mu = 6.674e-11 * star.0;
    let au = 1.495_978_707e11;
    let a1 = au;
    let a2 = au * 2.0_f64.powf(2.0 / 3.0);
    for (k, e) in picks.drain(..).enumerate() {
        let aa = if k == 0 { a1 } else { a2 };
        let vc = (mu / aa).sqrt();
        if let Ok((_, mut g, _)) = q.get_mut(e) {
            g.pos = star.1 + bevy::math::DVec3::new(aa, 0.0, 0.0);
            g.vel = bevy::math::DVec3::new(0.0, 0.0, vc);
        }
        if k == 0 {
            sel.0 = Some(e);
        }
    }
    speed.0 = 5.0e5;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoPrec(u32);

fn auto_prec(
    mut a: ResMut<AutoPrec>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut star = (0.0_f64, bevy::math::DVec3::ZERO);
    for (_, g, v) in &q {
        if v.star && g.mass > star.0 {
            star = (g.mass, g.pos);
        }
    }
    if star.0 <= 0.0 {
        return;
    }
    let mut light = (f64::MAX, None);
    for (e, g, v) in &q {
        if !v.star && g.mass < light.0 {
            light = (g.mass, Some(e));
        }
    }
    let Some(te) = light.1 else {
        return;
    };
    let mu = 6.674e-11 * star.0;
    let au = 1.495_978_707e11;
    let sa = 0.05 * au;
    let ecc = 0.5;
    let rp = sa * (1.0 - ecc);
    let vp = (mu * (1.0 + ecc) / (sa * (1.0 - ecc))).sqrt();
    if let Ok((_, mut g, _)) = q.get_mut(te) {
        g.pos = star.1 + bevy::math::DVec3::new(rp, 0.0, 0.0);
        g.vel = bevy::math::DVec3::new(0.0, 0.0, vp);
    }
    sel.0 = Some(te);
    speed.0 = 1.0e6;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoBelt(u32);

fn auto_belt(
    mut a: ResMut<AutoBelt>,
    mut mode: ResMut<scene::CamMode>,
    mut fly: ResMut<scene::FlyState>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 8 {
        return;
    }
    let au = 1.495_978_707e11;
    fly.pos = bevy::math::DVec3::new(0.0, 6.5 * au, 0.2 * au);
    fly.yaw = std::f32::consts::FRAC_PI_2;
    fly.pitch = -1.45;
    fly.speed = 1.0e9;
    *mode = scene::CamMode::Fly;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoMars(u32);

fn auto_mars(
    mut a: ResMut<AutoMars>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &Name)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    for (e, n) in &q {
        if n.as_str() == "Mars" {
            sel.0 = Some(e);
            a.0 = u32::MAX;
            return;
        }
    }
}

#[derive(Resource)]
struct AutoIce(u32);

fn auto_ice(
    mut a: ResMut<AutoIce>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &Name)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    for (e, n) in &q {
        if n.as_str() == "Neptune" {
            sel.0 = Some(e);
            a.0 = u32::MAX;
            return;
        }
    }
}

#[derive(Resource)]
struct AutoGas(u32);

fn auto_gas(
    mut a: ResMut<AutoGas>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &Name)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    for (e, n) in &q {
        if n.as_str() == "Jupiter" {
            sel.0 = Some(e);
            a.0 = u32::MAX;
            return;
        }
    }
}

#[derive(Resource)]
struct AutoRing(u32);

fn auto_ring(
    mut a: ResMut<AutoRing>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &Name)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    for (e, n) in &q {
        if n.as_str() == "Saturn" {
            sel.0 = Some(e);
            a.0 = u32::MAX;
            return;
        }
    }
}

#[derive(Resource)]
struct AutoStar(u32);

fn auto_star(
    mut a: ResMut<AutoStar>,
    mut sel: ResMut<scene::Selected>,
    q: Query<(Entity, &scene::Grav, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut best = (0.0_f64, None);
    for (e, g, v) in &q {
        if v.star && g.mass > best.0 {
            best = (g.mass, Some(e));
        }
    }
    if best.1.is_some() {
        sel.0 = best.1;
        a.0 = u32::MAX;
    }
}

#[derive(Resource)]
struct AutoNeb(u32);

fn auto_neb(
    mut a: ResMut<AutoNeb>,
    mut mode: ResMut<scene::CamMode>,
    mut fly: ResMut<scene::FlyState>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 8 {
        return;
    }
    let n = bevy::math::DVec3::new(8.0e12, 1.5e12, -4.0e12);
    let c = n + bevy::math::DVec3::new(0.6e12, 0.4e12, 4.2e12);
    let dir = (n - c).normalize();
    fly.pos = c;
    fly.yaw = (dir.z as f32).atan2(dir.x as f32);
    fly.pitch = (dir.y as f32).clamp(-1.0, 1.0).asin();
    fly.speed = 1.0e9;
    *mode = scene::CamMode::Fly;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoLock(u32);

fn auto_lock(
    mut a: ResMut<AutoLock>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Shape, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut giant = (0.0_f64, bevy::math::DVec3::ZERO,
        bevy::math::DVec3::ZERO, 1.0_f64);
    let mut moon = (f64::MAX, None);
    for (e, g, s, v) in &q {
        if v.star {
            continue;
        }
        if g.mass < 1.0e29 && g.mass > giant.0 {
            giant = (g.mass, g.pos, g.vel, s.base_radius);
        }
        if g.mass < moon.0 {
            moon = (g.mass, Some(e));
        }
    }
    let Some(me) = moon.1 else {
        return;
    };
    let r = giant.3 * 2.3;
    let vc = (6.674e-11 * giant.0 / r).sqrt();
    if let Ok((_, mut g, _, _)) = q.get_mut(me) {
        g.pos = giant.1 + bevy::math::DVec3::new(r, 0.0, 0.0);
        g.vel = giant.2 + bevy::math::DVec3::new(0.0, 0.0, vc);
    }
    commands.entity(me).insert(scene::Spin(5.0e-3));
    sel.0 = Some(me);
    speed.0 = 1.0e7;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoSubl(u32);

fn auto_subl(
    mut a: ResMut<AutoSubl>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut mode: ResMut<scene::CamMode>,
    mut fly: ResMut<scene::FlyState>,
    rigs: Query<&scene::CometRig>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let Some(body) = rigs.iter().next().map(|r| r.body) else {
        return;
    };
    let mut star = bevy::math::DVec3::ZERO;
    let mut sm = 0.0_f64;
    for (_, g, v) in &q {
        if v.star && g.mass > sm {
            sm = g.mass;
            star = g.pos;
        }
    }
    let r = 1.2 * 1.495_978_707e11;
    let vc = (6.674e-11 * sm / r).sqrt();
    if let Ok((_, mut g, _)) = q.get_mut(body) {
        g.pos = star + bevy::math::DVec3::new(r, 0.0, 0.0);
        g.vel = bevy::math::DVec3::new(0.0, 0.0, vc);
    }
    sel.0 = Some(body);
    speed.0 = 5.0e5;
    let cpos = star + bevy::math::DVec3::new(r, 0.0, 0.0);
    let cam = cpos + bevy::math::DVec3::new(0.0, 0.7e12, 3.0e12);
    let dir = (cpos - cam).normalize();
    fly.pos = cam;
    fly.yaw = (dir.z as f32).atan2(dir.x as f32);
    fly.pitch = (dir.y as f32).clamp(-1.0, 1.0).asin();
    fly.speed = 1.0e9;
    *mode = scene::CamMode::Fly;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoGw(u32);

#[derive(Resource)]
struct AutoChaos(u32, scene::Chaos);

fn auto_chaos(
    mut a: ResMut<AutoChaos>,
    mut sel: ResMut<scene::Selected>,
    mut req: ResMut<scene::ChaosReq>,
    q: Query<(Entity, &scene::Grav, &scene::Visual)>,
) {
    a.0 += 1;
    if a.0 != 8 {
        return;
    }
    let mut best = (0.0_f64, None);
    for (e, g, v) in &q {
        if !v.star && g.mass > best.0 {
            best = (g.mass, Some(e));
        }
    }
    if best.1.is_some() {
        sel.0 = best.1;
    }
    req.0 = Some(a.1);
}

fn auto_gw(
    mut a: ResMut<AutoGw>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut scene::Grav, &mut scene::Shape, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut star = bevy::math::DVec3::ZERO;
    let mut sm = 0.0_f64;
    for (_, g, _, v) in &q {
        if v.star && g.mass > sm {
            sm = g.mass;
            star = g.pos;
        }
    }
    let picks: Vec<Entity> = q
        .iter()
        .filter(|(_, g, _, v)| !v.star && g.mass < 1.0e29)
        .map(|(e, _, _, _)| e)
        .take(2)
        .collect();
    if picks.len() < 2 {
        return;
    }
    let a0 = 8.0e6;
    let base = star + bevy::math::DVec3::new(3.0e11, 0.0, 0.0);
    for (k, e) in picks.iter().enumerate() {
        if let Ok((_, mut g, mut s, _)) = q.get_mut(*e) {
            g.mass = 1.5e30;
            let off = if k == 0 { 0.5 } else { -0.5 } * a0;
            g.pos = base + bevy::math::DVec3::new(off, 0.0, 0.0);
            g.vel = bevy::math::DVec3::ZERO;
            s.base_radius = 1.5e4;
            s.radius = 1.5e4;
        }
    }
    commands.entity(picks[0]).insert((
        scene::GwPair {
            other: picks[1],
            a: a0,
            phase: 0.0,
            e1: bevy::math::DVec3::X,
            e2: bevy::math::DVec3::Z,
            com: base,
            vcom: bevy::math::DVec3::ZERO,
        },
        scene::GwBound,
    ));
    commands.entity(picks[1]).insert(scene::GwBound);
    sel.0 = Some(picks[0]);
    speed.0 = 1.0e8;
    a.0 = u32::MAX;
}

#[derive(Resource)]
struct AutoAtmo(u32);

fn auto_atmo(
    mut a: ResMut<AutoAtmo>,
    mut sel: ResMut<scene::Selected>,
    mut speed: ResMut<scene::SimSpeed>,
    mut q: Query<(Entity, &mut scene::Grav, &scene::Shape, &scene::Visual)>,
) {
    if a.0 == u32::MAX {
        return;
    }
    a.0 += 1;
    if a.0 < 6 {
        return;
    }
    let mut star = (0.0_f64, bevy::math::DVec3::ZERO,
        bevy::math::DVec3::ZERO);
    for (_, g, _, v) in &q {
        if v.star && g.mass > star.0 {
            star = (g.mass, g.pos, g.vel);
        }
    }
    let mut target = (f64::MAX, None);
    for (e, g, _, v) in &q {
        if v.star || v.atmo.is_none() {
            continue;
        }
        let ice = matches!(
            v.kind,
            crate::planetgen::PlanetKind::GasBands { .. }
                | crate::planetgen::PlanetKind::IceGiant { .. }
        );
        if ice && g.mass < target.0 {
            target = (g.mass, Some(e));
        }
    }
    let _ = (star.1, star.2);
    if let Some(te) = target.1 {
        if let Ok((_, mut g, _, _)) = q.get_mut(te) {
            g.mass = 1.5e24;
        }
        sel.0 = Some(te);
        speed.0 = 5.0e6;
        a.0 = u32::MAX;
    }
}

fn disable_egui_autocontext(mut settings: ResMut<EguiGlobalSettings>) {
    settings.auto_create_primary_context = false;
}

fn spawn_menu_cam(mut commands: Commands) {
    commands.spawn((
        MenuScene,
        Camera2d,
        bevy_egui::PrimaryEguiContext,
    ));
}

fn despawn_menu_cam(mut commands: Commands, q: Query<Entity, With<MenuScene>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn escape_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppMode>>,
    mut next: ResMut<NextState<AppMode>>,
) {
    if keys.just_pressed(KeyCode::Escape)
        && !matches!(state.get(), AppMode::Menu)
    {
        next.set(AppMode::Menu);
    }
}

fn toggle_mode(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppMode>>,
    mut next: ResMut<NextState<AppMode>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        match state.get() {
            AppMode::BlackHole => next.set(AppMode::SolarSystem),
            AppMode::SolarSystem => next.set(AppMode::BlackHole),
            AppMode::Menu => {}
        }
    }
}

fn shot_system(
    mut commands: Commands,
    mut shot: ResMut<ShotMode>,
    mut exit: MessageWriter<AppExit>,
) {
    shot.frame += 1;
    if shot.frame == 90 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(shot.path.clone()));
    }
    if shot.frame == 130 {
        exit.write(AppExit::Success);
    }
}

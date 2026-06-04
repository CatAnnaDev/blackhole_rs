use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::AppMode;
use crate::blackhole::BhView;
use crate::physics;
use crate::scene::{G as GRAV_G};
use crate::scene::{
    ActivePreset, BodyClass, C_LIGHT, CamMode, ConvertReq, Grav, LoadReq,
    Atmosphere, NewBodyType,
    MassXfer, OrbitInfo,
    PRESETS, Paused, PrecessionInfo, ReloadReq, Relativity, SaveReq, Selected, Shape, SimClock,
    Boom, Chaos, ChaosReq, CineCam,
    SimSpeed, ShowGrid, SizeExaggeration, Spin, SpawnRequest, SupernovaReq,
    Temperature, Visual,
};
use crate::scene::SUN_MASS;

const R_SUN: f64 = 6.957e8;
const AU: f64 = 1.495_978_7e11;
const LY: f64 = 9.460_730_5e15;

fn fmt_dist(m: f64) -> String {
    if m >= 0.1 * LY {
        format!("{:.3} ly", m / LY)
    } else if m >= 0.01 * AU {
        format!("{:.4} AU", m / AU)
    } else {
        format!("{:.0} km", m / 1000.0)
    }
}

const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 196, 214);

fn theme(ctx: &egui::Context) {
    use egui::Color32;
    let mut v = egui::Visuals::dark();
    let bg = Color32::from_rgba_unmultiplied(10, 13, 19, 240);
    v.panel_fill = bg;
    v.window_fill = bg;
    v.extreme_bg_color = Color32::from_rgba_unmultiplied(4, 6, 10, 245);
    v.override_text_color = Some(Color32::from_rgb(208, 220, 230));
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT);
    v.hyperlink_color = ACCENT;
    v.widgets.noninteractive.bg_stroke.color =
        Color32::from_rgb(30, 38, 48);
    v.widgets.hovered.bg_fill = Color32::from_rgb(30, 44, 56);
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.inactive.bg_fill = Color32::from_rgb(22, 28, 36);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(18, 23, 30);
    ctx.set_visuals(v);

    let mut s = (*ctx.style()).clone();
    s.spacing.slider_width = 104.0;
    s.spacing.item_spacing = egui::vec2(8.0, 8.0);
    s.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(s);
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(
        egui::RichText::new(title.to_uppercase())
            .color(ACCENT)
            .strong()
            .size(13.0),
    );
    ui.separator();
}

fn kv(ui: &mut egui::Ui, k: &str, val: String) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{k}:")).weak());
        ui.label(egui::RichText::new(val).monospace().color(ACCENT));
    });
}

#[derive(SystemParam)]
pub struct Ctl<'w> {
    sim_speed: ResMut<'w, SimSpeed>,
    clock: Res<'w, SimClock>,
    size_ex: ResMut<'w, SizeExaggeration>,
    paused: ResMut<'w, Paused>,
    active_preset: ResMut<'w, ActivePreset>,
    reload: ResMut<'w, ReloadReq>,
    save_req: ResMut<'w, SaveReq>,
    load_req: ResMut<'w, LoadReq>,
    spawn: ResMut<'w, SpawnRequest>,
    new_type: ResMut<'w, NewBodyType>,
    convert: ResMut<'w, ConvertReq>,
    supernova: ResMut<'w, SupernovaReq>,
    chaos: ResMut<'w, ChaosReq>,
    boom: ResMut<'w, Boom>,
    cine: ResMut<'w, CineCam>,
    grid: ResMut<'w, ShowGrid>,
    relativity: ResMut<'w, Relativity>,
    prec: Res<'w, PrecessionInfo>,
    orbit: Res<'w, OrbitInfo>,
    mx: Res<'w, MassXfer>,
}

pub fn ui_panel(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut view: ResMut<BhView>,
    mut ctl: Ctl,
    mut selected: ResMut<Selected>,
    mut bodies: Query<(
        Entity,
        &Name,
        &mut Grav,
        &mut Shape,
        &mut Spin,
        &Visual,
        Option<&Temperature>,
        Option<&Atmosphere>,
    )>,
    state: Res<State<AppMode>>,
    cam_mode: Res<CamMode>,
    mut next: ResMut<NextState<AppMode>>,
    mut exit: MessageWriter<AppExit>,
) -> Result {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let list: Vec<(Entity, String)> = bodies
        .iter()
        .map(|(e, n, _, _, _, _, _, _)| (e, n.as_str().to_string()))
        .collect();
    let solar = matches!(state.get(), AppMode::SolarSystem);
    let star = bodies.iter().fold(
        (0.0_f64, bevy::math::DVec3::ZERO),
        |acc, (_, _, g, _, _, _, _, _)| {
            if g.mass > acc.0 { (g.mass, g.pos) } else { acc }
        },
    );

    let ctx = contexts.ctx_mut()?;
    theme(ctx);

    if ctl.boom.amt > 0.004 {
        let sr = ctx.content_rect();
        let c = ctl.boom.col;
        let a = (ctl.boom.amt * 0.55).clamp(0.0, 0.8);
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("boom_flash"),
        ));
        painter.rect_filled(
            sr,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                (a * 255.0) as u8,
            ),
        );
        ctl.boom.amt *= 0.84;
        ctx.request_repaint();
    } else {
        ctl.boom.amt = 0.0;
    }

    // ---------- MAIN MENU ----------
    if matches!(state.get(), AppMode::Menu) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.label(
                    egui::RichText::new("SPACE  SANDBOX")
                        .size(58.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.label(
                    egui::RichText::new(
                        "realistic gravity, relativity & black-hole sandbox",
                    )
                    .size(15.0)
                    .weak(),
                );
                ui.add_space(50.0);
                let btn = |ui: &mut egui::Ui, t: &str| {
                    ui.add_sized(
                        [300.0, 46.0],
                        egui::Button::new(egui::RichText::new(t).size(20.0)),
                    )
                    .clicked()
                };
                if btn(ui, "Solar System") {
                    ctl.active_preset.0 = 0;
                    next.set(AppMode::SolarSystem);
                }
                ui.add_space(9.0);
                if btn(ui, "TRAPPIST-1") {
                    ctl.active_preset.0 = 1;
                    next.set(AppMode::SolarSystem);
                }
                ui.add_space(9.0);
                if btn(ui, "Star Cluster") {
                    ctl.active_preset.0 = 2;
                    next.set(AppMode::SolarSystem);
                }
                ui.add_space(9.0);
                if btn(ui, "Milky Way") {
                    ctl.active_preset.0 = 3;
                    next.set(AppMode::SolarSystem);
                }
                ui.add_space(9.0);
                if btn(ui, "Black Hole") {
                    next.set(AppMode::BlackHole);
                }
                ui.add_space(9.0);
                if btn(ui, "Quit") {
                    exit.write(AppExit::Success);
                }
                ui.add_space(44.0);
                ui.label(
                    egui::RichText::new(
                        "Esc menu    Tab switch    F free-fly    \
                         click select    drag orbit/look    scroll zoom    \
                         right-drag fling",
                    )
                    .small()
                    .weak(),
                );
            });
        });
        return Ok(());
    }

    // ---------- TOP BAR ----------
    egui::TopBottomPanel::top("top").exact_height(34.0).show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new("SPACE SANDBOX").strong().color(ACCENT),
            );
            ui.separator();
            ui.label(if solar { "Solar System" } else { "Black Hole" });
            if solar && *cam_mode == CamMode::Fly {
                ui.label(
                    egui::RichText::new("FLY")
                        .strong()
                        .color(egui::Color32::from_rgb(120, 230, 140)),
                );
            }
            if ui.small_button("switch (Tab)").clicked() {
                next.set(if solar {
                    AppMode::BlackHole
                } else {
                    AppMode::SolarSystem
                });
            }
            if ui.small_button("menu (Esc)").clicked() {
                next.set(AppMode::Menu);
            }
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(format!("{fps:.0} FPS"))
                            .monospace()
                            .weak(),
                    );
                },
            );
        });
    });

    // ---------- BOTTOM TIME BAR (solar only) ----------
    if solar {
        egui::TopBottomPanel::bottom("bottom")
            .exact_height(46.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let p = if ctl.paused.0 { "▶" } else { "❚❚" };
                    if ui
                        .add_sized([38.0, 26.0], egui::Button::new(p))
                        .clicked()
                    {
                        ctl.paused.0 = !ctl.paused.0;
                    }
                    ui.add(
                        egui::Slider::new(&mut ctl.sim_speed.0, 0.0..=2.0e8)
                            .logarithmic(true)
                            .show_value(false)
                            .text("time"),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.1} d/s",
                            ctl.sim_speed.0 / 86_400.0
                        ))
                        .monospace(),
                    );
                    if ui.small_button("realtime").clicked() {
                        ctl.sim_speed.0 = 1.0;
                        ctl.paused.0 = false;
                    }
                    ui.separator();
                    let yrs = (ctl.clock.0 / 31_557_600.0).floor();
                    let days =
                        ((ctl.clock.0 % 31_557_600.0) / 86_400.0).floor();
                    ui.label(
                        egui::RichText::new(format!("T+ {yrs:.0}y {days:.0}d"))
                            .monospace()
                            .color(ACCENT),
                    );
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui.small_button("load").clicked() {
                                ctl.load_req.0 = true;
                            }
                            if ui.small_button("save").clicked() {
                                ctl.save_req.0 = true;
                            }
                            ui.separator();
                            let rl = ctl.relativity.0;
                            if ui
                                .selectable_label(rl, "relativity")
                                .clicked()
                            {
                                ctl.relativity.0 = !rl;
                            }
                            let g = ctl.grid.0;
                            if ui.selectable_label(g, "grid (G)").clicked() {
                                ctl.grid.0 = !g;
                            }
                        },
                    );
                });
            });
    }

    // ---------- LEFT PANEL ----------
    egui::SidePanel::left("left")
        .resizable(true)
        .default_width(286.0)
        .width_range(240.0..=420.0)
        .show(ctx, |ui| {
          egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
            if solar {
                section(ui, "Scenario");
                ui.horizontal_wrapped(|ui| {
                    for (i, pr) in PRESETS.iter().enumerate() {
                        let on = ctl.active_preset.0 == i;
                        if ui.selectable_label(on, pr.name).clicked() && !on {
                            ctl.active_preset.0 = i;
                            ctl.reload.0 = true;
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("🎲 Random").clicked() {
                        let n = PRESETS.len();
                        let seed = std::time::SystemTime::now()
                            .duration_since(
                                std::time::UNIX_EPOCH,
                            )
                            .map(|d| d.subsec_nanos() as usize)
                            .unwrap_or(0);
                        let mut idx = seed % n;
                        if idx == ctl.active_preset.0 {
                            idx = (idx + 1) % n;
                        }
                        ctl.active_preset.0 = idx;
                        ctl.reload.0 = true;
                    }
                    if ui.button("↺ Restart").clicked() {
                        ctl.reload.0 = true;
                    }
                    let cine = ctl.cine.0;
                    if ui
                        .selectable_label(cine, "🎥 Cinematic")
                        .clicked()
                    {
                        ctl.cine.0 = !cine;
                    }
                });
                ui.add(
                    egui::Slider::new(&mut ctl.size_ex.0, 1.0..=3000.0)
                        .logarithmic(true)
                        .text("body size ×"),
                );

                section(ui, "Create");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("newtype")
                        .selected_text(ctl.new_type.0.label())
                        .show_ui(ui, |ui| {
                            for c in BodyClass::all() {
                                ui.selectable_value(
                                    &mut ctl.new_type.0,
                                    c,
                                    c.label(),
                                );
                            }
                        });
                    if ui.button("+ add").clicked() {
                        ctl.spawn.0 = true;
                    }
                });
                ui.label(
                    egui::RichText::new("right-drag in space to fling it")
                        .small()
                        .weak(),
                );

                section(ui, "Bodies");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (e, name) in &list {
                        let on = selected.0 == Some(*e);
                        if ui
                            .selectable_label(on, name.as_str())
                            .clicked()
                        {
                            selected.0 = Some(*e);
                        }
                    }
                });
            } else {
                section(ui, "Kerr Black Hole");
                ui.add(
                    egui::Slider::new(&mut view.spin, 0.0..=0.999)
                        .text("spin a"),
                );
                let a = view.spin as f64;
                ui.label(
                    egui::RichText::new(format!(
                        "horizon {:.3}   ISCO {:.3}",
                        physics::horizon(a),
                        physics::isco(a)
                    ))
                    .small()
                    .weak(),
                );
                section(ui, "Disk");
                ui.add(
                    egui::Slider::new(&mut view.temp, 1200.0..=30000.0)
                        .logarithmic(true)
                        .text("peak T (K)"),
                );
                ui.add(
                    egui::Slider::new(&mut view.brightness, 0.01..=1.0)
                        .text("brightness"),
                );
                ui.add(
                    egui::Slider::new(&mut view.r_out, 8.0..=40.0)
                        .text("outer r"),
                );
                ui.add(
                    egui::Slider::new(&mut view.exposure, 0.1..=8.0)
                        .text("exposure"),
                );

                section(ui, "Style");
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(false, "Scientific").clicked() {
                        view.spin = 0.9;
                        view.doppler = 1.0;
                        view.procedural = 1.0;
                        view.glow = 0.18;
                        view.disk_on = 1.0;
                        view.disk_inner = 0.0;
                        view.temp = 3600.0;
                        view.brightness = 0.30;
                        view.r_out = 22.0;
                        view.max_steps = 2600.0;
                    }
                    if ui.selectable_label(false, "Interstellar").clicked() {
                        view.spin = 0.6;
                        view.doppler = 0.0;
                        view.procedural = 0.0;
                        view.glow = 0.55;
                        view.disk_on = 1.0;
                        view.disk_inner = 9.26;
                        view.temp = 4500.0;
                        view.brightness = 0.55;
                        view.r_out = 18.7;
                        view.radius = 30.0;
                        view.pitch = 0.060;
                        view.fov_deg = 50.0;
                        view.max_steps = 2600.0;
                    }
                    if ui.selectable_label(false, "Performance").clicked() {
                        view.doppler = 0.0;
                        view.procedural = 0.0;
                        view.glow = 0.4;
                        view.disk_on = 1.0;
                        view.max_steps = 800.0;
                        view.aa = 1.0;
                        view.render_scale = 0.4;
                    }
                });
                let mut dop = view.doppler;
                if ui
                    .add(
                        egui::Slider::new(&mut dop, 0.0..=1.0)
                            .text("doppler"),
                    )
                    .changed()
                {
                    view.doppler = dop;
                }
                ui.add(
                    egui::Slider::new(&mut view.glow, 0.0..=1.0).text("glow"),
                );
                let mut proc = view.procedural > 0.5;
                if ui.checkbox(&mut proc, "procedural disk").changed() {
                    view.procedural = if proc { 1.0 } else { 0.0 };
                }
                let mut don = view.disk_on > 0.5;
                if ui.checkbox(&mut don, "accretion disk").changed() {
                    view.disk_on = if don { 1.0 } else { 0.0 };
                }
                ui.add(
                    egui::Slider::new(&mut view.disk_inner, 0.0..=20.0)
                        .text("inner r (0=ISCO)"),
                );

                section(ui, "Camera");
                ui.add(
                    egui::Slider::new(&mut view.radius, 3.0..=120.0)
                        .logarithmic(true)
                        .text("distance"),
                );
                ui.add(
                    egui::Slider::new(&mut view.pitch, -1.5..=1.5)
                        .text("pitch"),
                );
                ui.add(
                    egui::Slider::new(&mut view.yaw, -3.14..=3.14)
                        .text("yaw"),
                );
                ui.add(
                    egui::Slider::new(&mut view.fov_deg, 20.0..=100.0)
                        .text("fov"),
                );
                section(ui, "Quality");
                ui.add(
                    egui::Slider::new(&mut view.max_steps, 600.0..=10000.0)
                        .text("steps"),
                );
                ui.add(
                    egui::Slider::new(&mut view.aa, 1.0..=16.0)
                        .integer()
                        .text("anti-alias"),
                );
                ui.add(
                    egui::Slider::new(&mut view.render_scale, 0.2..=1.0)
                        .text("resolution"),
                );
                ui.label(
                    egui::RichText::new("drag orbit | scroll zoom")
                        .small()
                        .weak(),
                );
            }
            });
        });

    // ---------- RIGHT PROPERTIES PANEL ----------
    if solar {
        if let Some(sel) = selected.0 {
            egui::SidePanel::right("props")
                .resizable(true)
                .default_width(300.0)
                .width_range(260.0..=440.0)
                .show(ctx, |ui| {
                  egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                    if let Ok((
                        _,
                        name,
                        mut grav,
                        mut shape,
                        mut spin,
                        vis,
                        temp,
                        atmo,
                    )) =
                        bodies.get_mut(sel)
                    {
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(name.as_str())
                                .size(20.0)
                                .strong()
                                .color(ACCENT),
                        );

                        section(ui, "Physical");
                        let mut msun = grav.mass / SUN_MASS;
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut msun,
                                    1.0e-9..=1.0e3,
                                )
                                .logarithmic(true)
                                .custom_formatter(|v, _| {
                                    format!("{v:.4} M\u{2609}")
                                })
                                .text("mass"),
                            )
                            .changed()
                        {
                            grav.mass = msun * SUN_MASS;
                        }
                        let mut rsun = shape.radius / R_SUN;
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut rsun,
                                    1.0e-4..=1.0e3,
                                )
                                .logarithmic(true)
                                .custom_formatter(|v, _| {
                                    format!("{v:.4} R\u{2609}")
                                })
                                .text("radius"),
                            )
                            .changed()
                        {
                            shape.radius = rsun * R_SUN;
                        }
                        section(ui, "Orbit & Spin");
                        let mut spd = grav.vel.length() / 1000.0;
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut spd,
                                    0.0..=2.99e5,
                                )
                                .logarithmic(true)
                                .custom_formatter(|v, _| {
                                    format!("{v:.2} km/s")
                                })
                                .text("speed"),
                            )
                            .changed()
                        {
                            let len = grav.vel.length();
                            let cap = 0.999 * C_LIGHT / 1000.0;
                            let s = spd.min(cap) * 1000.0;
                            if len > 0.0 {
                                grav.vel = grav.vel / len * s;
                            }
                        }
                        let rel = grav.pos - star.1;
                        let mut dist_au = rel.length() / AU;
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut dist_au,
                                    1.0e-4..=1.0e4,
                                )
                                .logarithmic(true)
                                .custom_formatter(|v, _| {
                                    format!("{v:.4} AU")
                                })
                                .text("orbit r"),
                            )
                            .changed()
                        {
                            let rl = rel.length();
                            if rl > 0.0 {
                                grav.pos =
                                    star.1 + rel / rl * (dist_au * AU);
                            }
                        }
                        let mut dps = spin.0.to_degrees();
                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut dps,
                                    -720.0..=720.0,
                                )
                                .custom_formatter(|v, _| {
                                    format!("{v:.1} \u{b0}/s")
                                })
                                .text("spin"),
                            )
                            .changed()
                        {
                            spin.0 = dps.to_radians();
                        }

                        kv(
                            ui,
                            "mass",
                            format!("{:.4} M\u{2609}", grav.mass / SUN_MASS),
                        );
                        kv(
                            ui,
                            "radius",
                            format!("{:.4} R\u{2609}", shape.radius / R_SUN),
                        );
                        kv(
                            ui,
                            "speed",
                            format!("{:.2} km/s", grav.vel.length() / 1000.0),
                        );
                        kv(ui, "dist", fmt_dist(grav.pos.length()));

                        if let Some(t) = temp {
                            section(ui, "Climate");
                            let k = t.0;
                            kv(
                                ui,
                                "temp",
                                format!("{k:.0} K ({:.0} \u{b0}C)", k - 273.15),
                            );
                            let zone = if k < 248.0 {
                                ("frozen", egui::Color32::from_rgb(
                                    130, 190, 255,
                                ))
                            } else if k <= 330.0 {
                                ("temperate", egui::Color32::from_rgb(
                                    120, 230, 140,
                                ))
                            } else if k < 600.0 {
                                ("hot", egui::Color32::from_rgb(
                                    255, 170, 90,
                                ))
                            } else {
                                ("molten", egui::Color32::from_rgb(
                                    255, 90, 60,
                                ))
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("zone:").weak(),
                                );
                                ui.label(
                                    egui::RichText::new(zone.0)
                                        .strong()
                                        .color(zone.1),
                                );
                            });
                            if vis.atmo.is_some() {
                                let f = atmo.map(|a| a.0).unwrap_or(1.0);
                                let c = if f > 0.66 {
                                    egui::Color32::from_rgb(120, 230, 140)
                                } else if f > 0.25 {
                                    egui::Color32::from_rgb(255, 170, 90)
                                } else {
                                    egui::Color32::from_rgb(255, 90, 60)
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("atmo:").weak(),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{:.0}%",
                                            f * 100.0
                                        ))
                                        .strong()
                                        .color(c),
                                    );
                                });
                            } else {
                                ui.label(
                                    egui::RichText::new("atmo: stripped")
                                        .weak()
                                        .color(egui::Color32::from_rgb(
                                            255, 90, 60,
                                        )),
                                );
                            }
                            let habitable = vis.atmo.is_some()
                                && (255.0..=320.0).contains(&k);
                            if habitable {
                                ui.label(
                                    egui::RichText::new("\u{2713} HABITABLE")
                                        .strong()
                                        .color(egui::Color32::from_rgb(
                                            90, 230, 130,
                                        )),
                                );
                            }
                        }

                        if ctl.orbit.valid {
                            section(ui, "Orbit (Kepler)");
                            kv(
                                ui,
                                "primary",
                                ctl.orbit.primary.clone(),
                            );
                            kv(
                                ui,
                                "class",
                                ctl.orbit.class.clone(),
                            );
                            if !ctl.orbit.sat_status.is_empty() {
                                kv(
                                    ui,
                                    "satellite",
                                    ctl.orbit
                                        .sat_status
                                        .clone(),
                                );
                            }
                            kv(
                                ui,
                                "v / v_esc",
                                format!(
                                    "{:.2} / {:.2} km/s",
                                    grav.vel.length() / 1000.0,
                                    ctl.orbit.v_esc_kms
                                ),
                            );
                            if ctl.orbit.vinf_kms > 0.0 {
                                kv(
                                    ui,
                                    "v∞",
                                    format!(
                                        "{:.2} km/s",
                                        ctl.orbit.vinf_kms
                                    ),
                                );
                            }
                            if ctl.orbit.reflex_ms > 0.0 {
                                kv(
                                    ui,
                                    "reflex K",
                                    format!(
                                        "{:.2} m/s",
                                        ctl.orbit.reflex_ms
                                    ),
                                );
                                kv(
                                    ui,
                                    "barycentre",
                                    format!(
                                        "{:.2} R☉ from star",
                                        ctl.orbit.bary_rsun
                                    ),
                                );
                            }
                            kv(
                                ui,
                                "a",
                                format!("{:.4} AU", ctl.orbit.a_au),
                            );
                            kv(
                                ui,
                                "e",
                                format!("{:.4}", ctl.orbit.e),
                            );
                            kv(
                                ui,
                                "i",
                                format!("{:.2}°", ctl.orbit.inc_deg),
                            );
                            if ctl.orbit.vinf_kms <= 0.0 {
                                kv(
                                    ui,
                                    "period",
                                    if ctl.orbit.period_days
                                        > 800.0
                                    {
                                        format!(
                                            "{:.2} yr",
                                            ctl.orbit.period_days
                                                / 365.25
                                        )
                                    } else {
                                        format!(
                                            "{:.2} d",
                                            ctl.orbit.period_days
                                        )
                                    },
                                );
                                kv(
                                    ui,
                                    "peri / apo",
                                    format!(
                                        "{:.3} / {:.3} AU",
                                        ctl.orbit.peri_au,
                                        ctl.orbit.apo_au
                                    ),
                                );
                                kv(
                                    ui,
                                    "Hill r",
                                    format!(
                                        "{:.4} AU",
                                        ctl.orbit.hill_au
                                    ),
                                );
                                kv(
                                    ui,
                                    "resonance",
                                    ctl.orbit.resonance.clone(),
                                );
                            } else {
                                kv(
                                    ui,
                                    "periapsis",
                                    format!(
                                        "{:.4} AU",
                                        ctl.orbit.peri_au
                                    ),
                                );
                            }
                            if ctl.orbit.teq_k > 0.0 {
                                kv(
                                    ui,
                                    "T_eq",
                                    format!(
                                        "{:.0} K  ({:.0} °C)",
                                        ctl.orbit.teq_k,
                                        ctl.orbit.teq_k - 273.15
                                    ),
                                );
                                kv(
                                    ui,
                                    "habitability",
                                    ctl.orbit.hz.clone(),
                                );
                            }
                            if ctl.orbit.roche_au > 0.0 {
                                kv(
                                    ui,
                                    "Roche limit",
                                    format!(
                                        "{:.5} AU · {}",
                                        ctl.orbit.roche_au,
                                        ctl.orbit.roche_status
                                    ),
                                );
                            }
                        }

                        if ctl.mx.active {
                            section(ui, "Binary mass transfer");
                            kv(
                                ui,
                                "donor → acc",
                                format!(
                                    "{} → {}",
                                    ctl.mx.donor, ctl.mx.acc
                                ),
                            );
                            kv(
                                ui,
                                "Roche fill",
                                format!("{:.3}", ctl.mx.fill),
                            );
                            kv(
                                ui,
                                "Ṁ",
                                format!(
                                    "{:.2e} M☉/yr",
                                    ctl.mx.rate_msun_yr
                                ),
                            );
                        }

                        section(ui, "Relativity");
                        let v = grav.vel.length();
                        let beta = (v / C_LIGHT).clamp(0.0, 0.999_999);
                        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
                        let r = (grav.pos - star.1).length().max(1.0);
                        let gd = (1.0
                            - 2.0 * GRAV_G * star.0
                                / (r * C_LIGHT * C_LIGHT))
                            .max(1.0e-9)
                            .sqrt();
                        kv(ui, "v / c", format!("{:.4}%", beta * 100.0));
                        kv(ui, "γ", format!("{gamma:.6}"));
                        kv(ui, "clock", format!("{:.6}", gd / gamma));
                        kv(
                            ui,
                            "E=γmc²",
                            format!(
                                "{:.2e} J",
                                gamma * grav.mass * C_LIGHT * C_LIGHT
                            ),
                        );
                        let rs2 = 2.0 * GRAV_G * grav.mass
                            / (shape.base_radius.max(1.0)
                                * C_LIGHT
                                * C_LIGHT);
                        kv(
                            ui,
                            "grav z (surf)",
                            if rs2 < 0.999 {
                                let z =
                                    1.0 / (1.0 - rs2).sqrt() - 1.0;
                                let vs = z * C_LIGHT;
                                if vs.abs() >= 1000.0 {
                                    format!(
                                        "{z:.3e} ({:.1} km/s)",
                                        vs / 1000.0
                                    )
                                } else {
                                    format!(
                                        "{z:.3e} ({vs:.0} m/s)"
                                    )
                                }
                            } else {
                                "→ ∞ (within horizon)".into()
                            },
                        );
                        let r_s = 2.0 * GRAV_G * grav.mass
                            / (C_LIGHT * C_LIGHT);
                        let compact =
                            r_s / shape.base_radius.max(1.0);
                        kv(
                            ui,
                            "Schwarzschild r_s",
                            if r_s >= 6.957e8 {
                                format!(
                                    "{:.3} R☉",
                                    r_s / 6.957e8
                                )
                            } else if r_s >= 1000.0 {
                                format!("{:.2} km", r_s / 1000.0)
                            } else {
                                format!("{r_s:.0} m")
                            },
                        );
                        kv(
                            ui,
                            "r_s / R",
                            if compact >= 1.0 {
                                format!(
                                    "{compact:.3}  ⚫ black hole"
                                )
                            } else {
                                format!("{compact:.3e}")
                            },
                        );
                        if ctl.prec.pred_arcsec > 0.0 {
                            kv(
                                ui,
                                "GR ϖ̇",
                                format!(
                                    "{:.3} ″/orbit",
                                    ctl.prec.pred_arcsec
                                ),
                            );
                            kv(
                                ui,
                                "GR ϖ̇/cy",
                                format!(
                                    "{:.2} ″",
                                    ctl.prec.pred_century
                                ),
                            );
                        }

                        section(ui, "Convert");
                        ui.horizontal_wrapped(|ui| {
                            for c in BodyClass::all() {
                                if ui.small_button(c.label()).clicked() {
                                    ctl.convert.0 = Some(c);
                                }
                            }
                        });
                        if vis.star {
                            ui.add_space(6.0);
                            if ui
                                .add_sized(
                                    [ui.available_width(), 32.0],
                                    egui::Button::new(
                                        egui::RichText::new("☢ SUPERNOVA")
                                            .strong()
                                            .color(egui::Color32::from_rgb(
                                                255, 140, 70,
                                            )),
                                    ),
                                )
                                .clicked()
                            {
                                ctl.supernova.0 = Some(sel);
                            }
                        }

                        ui.add_space(6.0);
                        section(ui, "💥 Chaos / Fun");
                        ui.horizontal_wrapped(|ui| {
                            if ui
                                .small_button("💥 Detonate")
                                .clicked()
                            {
                                ctl.supernova.0 = Some(sel);
                            }
                            if ui
                                .small_button("🎆 Shatter")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Shatter);
                            }
                            if ui
                                .small_button("☄ Barrage")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Barrage);
                            }
                            if ui
                                .small_button("🚀 Kick")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Kick);
                            }
                            if ui
                                .small_button("🕳 Rogue BH")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::RogueBh);
                            }
                            if ui
                                .small_button("✨ Vaporize")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Vaporize);
                            }
                            if ui
                                .small_button("⧉ Clone")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Clone);
                            }
                            if ui
                                .small_button("🌑 Giant impact")
                                .clicked()
                            {
                                ctl.chaos.0 =
                                    Some(Chaos::GiantImpact);
                            }
                            if ui
                                .small_button("🌟 Star fall")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::StarFall);
                            }
                            if ui
                                .small_button("☄ Comet swarm")
                                .clicked()
                            {
                                ctl.chaos.0 =
                                    Some(Chaos::CometSwarm);
                            }
                            if ui
                                .small_button("🌀 Scatter all")
                                .clicked()
                            {
                                ctl.chaos.0 =
                                    Some(Chaos::ScatterAll);
                            }
                            if ui
                                .small_button("❄ Freeze")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Freeze);
                            }
                            if ui
                                .small_button("⏪ Reverse")
                                .clicked()
                            {
                                ctl.chaos.0 = Some(Chaos::Reverse);
                            }
                        });
                    } else {
                        selected.0 = None;
                    }
                    });
                });
        }
    }

    Ok(())
}

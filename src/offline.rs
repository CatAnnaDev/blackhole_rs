use std::thread;

use bevy::math::DVec3;

use crate::physics::{horizon, isco, orbital_omega, page_thorne};

const PI: f64 = std::f64::consts::PI;

pub struct RenderParams {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub spin: f64,
    pub radius: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub fov_deg: f64,
    pub temp: f64,
    pub brightness: f64,
    pub r_out: f64,
    pub exposure: f64,
    pub max_steps: i32,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            samples: 160,
            spin: 0.9,
            radius: 19.0,
            yaw: 0.0,
            pitch: 0.55,
            fov_deg: 55.0,
            temp: 2200.0,
            brightness: 0.16,
            r_out: 15.0,
            exposure: 2.2,
            max_steps: 4000,
        }
    }
}

fn fract(x: f64) -> f64 {
    x - x.floor()
}

fn fract3(v: DVec3) -> DVec3 {
    DVec3::new(fract(v.x), fract(v.y), fract(v.z))
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn hash33(p3in: DVec3) -> DVec3 {
    let mut p3 = fract3(p3in * DVec3::new(0.1031, 0.1030, 0.0973));
    let add = p3.dot(DVec3::new(p3.y, p3.x, p3.z) + 33.33);
    p3 += DVec3::splat(add);
    let a = DVec3::new(p3.x + p3.x, p3.x, p3.y);
    let b = DVec3::new(p3.y, p3.x, p3.x);
    let c = DVec3::new(p3.z, p3.y, p3.x);
    fract3((a + b) * c)
}

fn blackbody(temp_k: f64) -> DVec3 {
    let t = temp_k.clamp(1000.0, 40000.0) / 100.0;
    let r;
    let g;
    let b;
    if t <= 66.0 {
        r = 1.0;
        g = (0.39008157 * t.ln() - 0.63184144).clamp(0.0, 1.0);
    } else {
        r = (1.29293618 * (t - 60.0).powf(-0.1332047592)).clamp(0.0, 1.0);
        g = (1.12989086 * (t - 60.0).powf(-0.0755148492)).clamp(0.0, 1.0);
    }
    if t >= 66.0 {
        b = 1.0;
    } else if t <= 19.0 {
        b = 0.0;
    } else {
        b = (0.54320679 * (t - 10.0).ln() - 1.19625408).clamp(0.0, 1.0);
    }
    DVec3::new(r, g, b)
}

fn star_layer(d: DVec3, grid: f64, density: f64, gain: f64) -> DVec3 {
    let p = d * grid;
    let cell = DVec3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = p - cell;
    let mut col = DVec3::ZERO;
    for oz in -1..=1 {
        for oy in -1..=1 {
            for ox in -1..=1 {
                let g = DVec3::new(ox as f64, oy as f64, oz as f64);
                let id = cell + g;
                let rnd = hash33(id);
                if rnd.x > density {
                    let sp = g + rnd;
                    let dist = (f - sp).length();
                    let bright = ((rnd.x - density) / (1.0 - density)).powf(4.0);
                    let core = bright * smoothstep(0.5, 0.0, dist) * gain;
                    let temp = mix(2800.0, 14000.0, rnd.z);
                    col += blackbody(temp) * core;
                }
            }
        }
    }
    col
}

fn starfield(dir: DVec3) -> DVec3 {
    let d = dir.normalize();
    let mut col = DVec3::ZERO;
    col += star_layer(d, 24.0, 0.86, 2.2);
    col += star_layer(d, 60.0, 0.93, 1.1);
    col += star_layer(d, 120.0, 0.965, 0.6);
    let band = (-(d.y * 2.5).powi(2)).exp();
    let dust = 0.5 + 0.5 * hash33((d * 16.0).floor()).x;
    col += DVec3::new(0.030, 0.038, 0.060) * band * dust;
    col += DVec3::new(0.004, 0.005, 0.009);
    col
}

fn sigma(r: f64, ct: f64, a: f64) -> f64 {
    r * r + a * a * ct * ct
}

fn delta(r: f64, a: f64) -> f64 {
    r * r - 2.0 * r + a * a
}

fn hamiltonian(r: f64, th: f64, pr: f64, pth: f64, e: f64, l: f64, a: f64) -> f64 {
    let st = th.sin().abs().max(1.0e-4);
    let ct = th.cos();
    let s2 = st * st;
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let aa = (r * r + a * a) * (r * r + a * a) - a * a * dl * s2;
    let g_rr = dl / sg;
    let g_thth = 1.0 / sg;
    let g_tt = -aa / (sg * dl);
    let g_tphi = -2.0 * a * r / (sg * dl);
    let g_phph = (dl - a * a * s2) / (sg * dl * s2);
    let pt = -e;
    let pph = l;
    0.5 * (g_tt * pt * pt
        + 2.0 * g_tphi * pt * pph
        + g_phph * pph * pph
        + g_rr * pr * pr
        + g_thth * pth * pth)
}

fn deriv(s: [f64; 4], e: f64, l: f64, a: f64) -> [f64; 4] {
    let (r, th, pr, pth) = (s[0], s[1], s[2], s[3]);
    let ct = th.cos();
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let dr = dl / sg * pr;
    let dth = pth / sg;
    let hr = 1.0e-3 * r.max(1.0);
    let hth = 1.0e-3;
    let dpr = -(hamiltonian(r + hr, th, pr, pth, e, l, a)
        - hamiltonian(r - hr, th, pr, pth, e, l, a))
        / (2.0 * hr);
    let dpth = -(hamiltonian(r, th + hth, pr, pth, e, l, a)
        - hamiltonian(r, th - hth, pr, pth, e, l, a))
        / (2.0 * hth);
    [dr, dth, dpr, dpth]
}

fn dphi(r: f64, th: f64, e: f64, l: f64, a: f64) -> f64 {
    let st = th.sin().abs().max(1.0e-4);
    let s2 = st * st;
    let ct = th.cos();
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let g_tphi = -2.0 * a * r / (sg * dl);
    let g_phph = (dl - a * a * s2) / (sg * dl * s2);
    g_tphi * (-e) + g_phph * l
}

struct Geo {
    state: [f64; 4],
    phi: f64,
    e: f64,
    l: f64,
}

fn init_geodesic(ro: DVec3, rd: DVec3, a: f64) -> Geo {
    let big_r2 = ro.dot(ro);
    let a2 = a * a;
    let r2 = 0.5
        * ((big_r2 - a2)
            + ((big_r2 - a2) * (big_r2 - a2) + 4.0 * a2 * ro.y * ro.y)
                .max(0.0)
                .sqrt());
    let r = r2.max(1.0e-6).sqrt();
    let th = (ro.y / r).clamp(-1.0, 1.0).acos();
    let phi = ro.z.atan2(ro.x);
    let rho = (r * r + a2).sqrt();
    let st = th.sin();
    let ct = th.cos();
    let cp = phi.cos();
    let sp = phi.sin();
    let er = DVec3::new((r / rho) * st * cp, ct, (r / rho) * st * sp).normalize();
    let eth = DVec3::new(rho * ct * cp, -r * st, rho * ct * sp).normalize();
    let eph = DVec3::new(-rho * st * sp, 0.0, rho * st * cp).normalize();
    let n = DVec3::new(rd.dot(er), rd.dot(eth), rd.dot(eph)).normalize();
    let s2 = (st * st).max(1.0e-6);
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let aa = (r * r + a2) * (r * r + a2) - a2 * dl * s2;
    let omega = 2.0 * a * r / aa;
    let alpha = (sg * dl / aa).max(1.0e-12).sqrt();
    let pup_t = 1.0 / alpha;
    let pup_r = (dl / sg).max(0.0).sqrt() * n.x;
    let pup_th = (1.0 / sg).max(0.0).sqrt() * n.y;
    let pup_ph = omega / alpha + (sg / aa).max(0.0).sqrt() / st.max(1.0e-4) * n.z;
    let g_tt = -(1.0 - 2.0 * r / sg);
    let g_tphi = -2.0 * a * r * s2 / sg;
    let g_phph = (aa / sg) * s2;
    let g_rr = sg / dl;
    let g_thth = sg;
    let p_t = g_tt * pup_t + g_tphi * pup_ph;
    let p_ph = g_tphi * pup_t + g_phph * pup_ph;
    let p_r = g_rr * pup_r;
    let p_th = g_thth * pup_th;
    Geo {
        state: [r, th, p_r, p_th],
        phi,
        e: -p_t,
        l: p_ph,
    }
}

fn add(s: [f64; 4], k: [f64; 4], h: f64) -> [f64; 4] {
    [s[0] + k[0] * h, s[1] + k[1] * h, s[2] + k[2] * h, s[3] + k[3] * h]
}

fn rk4(s: [f64; 4], h: f64, e: f64, l: f64, a: f64) -> [f64; 4] {
    let k1 = deriv(s, e, l, a);
    let k2 = deriv(add(s, k1, 0.5 * h), e, l, a);
    let k3 = deriv(add(s, k2, 0.5 * h), e, l, a);
    let k4 = deriv(add(s, k3, h), e, l, a);
    [
        s[0] + h / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
        s[1] + h / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
        s[2] + h / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
        s[3] + h / 6.0 * (k1[3] + 2.0 * k2[3] + 2.0 * k3[3] + k4[3]),
    ]
}

fn to_cartesian(r: f64, th: f64, phi: f64, a: f64) -> DVec3 {
    let rho = (r * r + a * a).sqrt();
    let st = th.sin();
    DVec3::new(rho * st * phi.cos(), r * th.cos(), rho * st * phi.sin())
}

fn disk_redshift(r: f64, e: f64, l: f64, a: f64) -> f64 {
    let om = orbital_omega(r, a);
    let g_tt = -(1.0 - 2.0 / r);
    let g_tphi = -2.0 * a / r;
    let g_phph = r * r + a * a + 2.0 * a * a / r;
    let denom = -(g_tt + 2.0 * om * g_tphi + om * om * g_phph);
    if denom <= 1.0e-6 {
        return 1.0;
    }
    let ut = 1.0 / denom.sqrt();
    let bottom = ut * (e - l * om);
    if bottom.abs() < 1.0e-6 {
        return 1.0;
    }
    e / bottom
}

fn disk_color(
    r: f64,
    e: f64,
    l: f64,
    a: f64,
    r_in: f64,
    r_out: f64,
    p: &RenderParams,
) -> DVec3 {
    let flux = page_thorne(r, a, r_in);
    let fref = page_thorne(r_in * 2.2, a, r_in).max(1.0e-3);
    let fnorm = (flux / fref).clamp(0.0, 24.0);
    let t_emit = p.temp * fnorm.powf(0.25);
    let g = disk_redshift(r, e, l, a).clamp(0.05, 4.0);
    let t_obs = (t_emit * g).clamp(900.0, 42000.0);
    let bb = blackbody(t_obs);
    let beaming = g * g * g;
    let radiance = fnorm.powf(0.85) * beaming * p.brightness;
    let edge = smoothstep(r_in, r_in + 0.4, r) * smoothstep(r_out, r_out - 6.0, r);
    bb * radiance * edge
}

fn trace(ro: DVec3, rd: DVec3, p: &RenderParams) -> DVec3 {
    let a = p.spin;
    let mut geo = init_geodesic(ro, rd, a);
    let rh = horizon(a) * 1.001;
    let r_escape = (ro.length() * 4.0).max(60.0);
    let r_in = isco(a);
    let r_out = p.r_out;
    let mut prev = ro;
    let mut captured = false;

    for _ in 0..p.max_steps {
        let r = geo.state[0];
        let th = geo.state[1];
        if r <= rh {
            captured = true;
            break;
        }
        if r >= r_escape {
            break;
        }
        let dist = r - rh;
        let h = (0.10 * dist.min(r)).clamp(0.012, 2.0);
        let new_state = rk4(geo.state, h, geo.e, geo.l, a);
        let dph = dphi(geo.state[0], geo.state[1], geo.e, geo.l, a) * h;

        let s_old = th - 0.5 * PI;
        let s_new = new_state[1] - 0.5 * PI;
        if s_old * s_new < 0.0 {
            let frac = s_old / (s_old - s_new);
            let r_cross = r + (new_state[0] - r) * frac;
            if r_cross >= r_in && r_cross <= r_out {
                return disk_color(r_cross, geo.e, geo.l, a, r_in, r_out, p);
            }
        }

        let nr = new_state[0];
        let mut nth = new_state[1];
        let npr = new_state[2];
        let mut npth = new_state[3];
        let mut nphi = geo.phi + dph;
        if nth < 0.0 {
            nth = -nth;
            npth = -npth;
            nphi += PI;
        } else if nth > PI {
            nth = 2.0 * PI - nth;
            npth = -npth;
            nphi += PI;
        }
        prev = to_cartesian(r, th, geo.phi, a);
        geo.state = [nr, nth, npr, npth];
        geo.phi = nphi;
    }

    if captured {
        return DVec3::ZERO;
    }
    let now = to_cartesian(geo.state[0], geo.state[1], geo.phi, a);
    starfield((now - prev).normalize())
}

fn aces(x: DVec3) -> DVec3 {
    let f = |v: f64| {
        ((v * (2.51 * v + 0.03)) / (v * (2.43 * v + 0.59) + 0.14)).clamp(0.0, 1.0)
    };
    DVec3::new(f(x.x), f(x.y), f(x.z))
}

fn lin_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn jitter(px: u32, py: u32, s: u32) -> (f64, f64) {
    let h = hash33(DVec3::new(px as f64, py as f64, s as f64 + 1.0));
    (h.x - 0.5, h.y - 0.5)
}

pub fn render(p: &RenderParams, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    let w = p.width;
    let h = p.height;

    let cp = p.pitch.cos();
    let eye = DVec3::new(
        p.radius * cp * p.yaw.cos(),
        p.radius * p.pitch.sin(),
        p.radius * cp * p.yaw.sin(),
    );
    let fwd = (-eye).normalize();
    let right = fwd.cross(DVec3::Y).normalize();
    let up = right.cross(fwd);
    let tan_half = (p.fov_deg.to_radians() * 0.5).tan();
    let aspect = w as f64 / h as f64;

    let mut buf = vec![0u8; (w * h * 3) as usize];
    let nthreads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let rows_per = h.div_ceil(nthreads as u32);

    thread::scope(|sc| {
        for (ti, chunk) in buf
            .chunks_mut((rows_per * w * 3) as usize)
            .enumerate()
        {
            let p = &p;
            sc.spawn(move || {
                let y0 = ti as u32 * rows_per;
                for ly in 0..(chunk.len() as u32 / (w * 3)) {
                    let py = y0 + ly;
                    for px in 0..w {
                        let mut acc = DVec3::ZERO;
                        for s in 0..p.samples {
                            let (jx, jy) = jitter(px, py, s);
                            let ndc_x =
                                (px as f64 + 0.5 + jx) / w as f64 * 2.0 - 1.0;
                            let ndc_y =
                                1.0 - (py as f64 + 0.5 + jy) / h as f64 * 2.0;
                            let dir = (fwd
                                + right * (ndc_x * aspect * tan_half)
                                + up * (ndc_y * tan_half))
                                .normalize();
                            acc += trace(eye, dir, p);
                        }
                        let c = aces(acc / p.samples as f64 * p.exposure);
                        let o = (ly * w + px) as usize * 3;
                        chunk[o] = (lin_to_srgb(c.x) * 255.0).round() as u8;
                        chunk[o + 1] = (lin_to_srgb(c.y) * 255.0).round() as u8;
                        chunk[o + 2] = (lin_to_srgb(c.z) * 255.0).round() as u8;
                    }
                }
            });
        }
    });

    let img = image::RgbImage::from_raw(w, h, buf).ok_or("buffer size")?;
    img.save(out)?;
    Ok(())
}

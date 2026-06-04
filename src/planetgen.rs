use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone, Copy)]
pub enum PlanetKind {
    Rocky { r: f32, g: f32, b: f32 },
    Terran,
    GasBands { warm: bool },
    IceGiant { deep: bool },
    Lava,
    Ocean,
    Desert,
    Carbon,
}

impl PlanetKind {
    pub fn to_code(&self) -> String {
        match self {
            PlanetKind::Rocky { r, g, b } => format!("rocky;{r};{g};{b}"),
            PlanetKind::Terran => "terran".into(),
            PlanetKind::GasBands { warm } => format!("gas;{}", *warm as u8),
            PlanetKind::IceGiant { deep } => format!("ice;{}", *deep as u8),
            PlanetKind::Lava => "lava".into(),
            PlanetKind::Ocean => "ocean".into(),
            PlanetKind::Desert => "desert".into(),
            PlanetKind::Carbon => "carbon".into(),
        }
    }

    pub fn from_code(s: &str) -> PlanetKind {
        let parts: Vec<&str> = s.split(';').collect();
        match parts.first().copied().unwrap_or("terran") {
            "rocky" => PlanetKind::Rocky {
                r: parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.5),
                g: parts.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.5),
                b: parts.get(3).and_then(|v| v.parse().ok()).unwrap_or(0.5),
            },
            "gas" => PlanetKind::GasBands {
                warm: parts.get(1).map(|v| *v == "1").unwrap_or(true),
            },
            "ice" => PlanetKind::IceGiant {
                deep: parts.get(1).map(|v| *v == "1").unwrap_or(false),
            },
            "lava" => PlanetKind::Lava,
            "ocean" => PlanetKind::Ocean,
            "desert" => PlanetKind::Desert,
            "carbon" => PlanetKind::Carbon,
            _ => PlanetKind::Terran,
        }
    }
}

fn hash3(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add((z as u32).wrapping_mul(2147483647))
        .wrapping_add(seed.wrapping_mul(362437));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(p: [f32; 3], seed: u32) -> f32 {
    let xi = p[0].floor() as i32;
    let yi = p[1].floor() as i32;
    let zi = p[2].floor() as i32;
    let xf = smooth(p[0] - xi as f32);
    let yf = smooth(p[1] - yi as f32);
    let zf = smooth(p[2] - zi as f32);

    let mut c = [[[0.0f32; 2]; 2]; 2];
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                c[dx][dy][dz] =
                    hash3(xi + dx as i32, yi + dy as i32, zi + dz as i32, seed);
            }
        }
    }
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(c[0][0][0], c[1][0][0], xf);
    let x10 = lerp(c[0][1][0], c[1][1][0], xf);
    let x01 = lerp(c[0][0][1], c[1][0][1], xf);
    let x11 = lerp(c[0][1][1], c[1][1][1], xf);
    let y0 = lerp(x00, x10, yf);
    let y1 = lerp(x01, x11, yf);
    lerp(y0, y1, zf)
}

fn fbm(dir: [f32; 3], seed: u32, octaves: u32, freq0: f32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = freq0;
    for _ in 0..octaves {
        sum += value_noise([dir[0] * freq, dir[1] * freq, dir[2] * freq], seed)
            * amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    sum
}

fn srgb(r: f32, g: f32, b: f32) -> [u8; 4] {
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
    [c(r), c(g), c(b), 255]
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn shade(kind: PlanetKind, dir: [f32; 3], lat: f32, seed: u32) -> [u8; 4] {
    match kind {
        PlanetKind::Rocky { r, g, b } => {
            let n = fbm(dir, seed, 5, 3.0) * 0.5 + 0.5;
            let basin = fbm(dir, seed ^ 0x5bd1, 4, 1.7) * 0.5 + 0.5;
            let mare = smoothstep(0.40, 0.66, basin);
            let highland = smoothstep(0.30, 0.85, n);
            let crat = fbm(dir, seed ^ 0x9e37, 3, 7.0);
            let pit = (1.0 - (crat.abs() * 4.0).min(1.0)) * 0.5;
            let spk = fbm(dir, seed ^ 0x33a7, 3, 22.0) * 0.12;
            let v = (0.50 + 0.42 * n + 0.22 * highland)
                * (1.0 - 0.55 * mare)
                * (1.0 - pit)
                + spk;
            let dr = 1.0 - 0.30 * mare;
            srgb(r * v * dr, g * v, b * v * (1.0 - 0.12 * mare))
        }
        PlanetKind::Terran => {
            let h = fbm(dir, seed, 6, 2.2);
            let al = lat.abs();
            let ice = (al - 0.82).max(0.0) / 0.18;
            if h < 0.02 {
                let deep = (-h * 6.0).clamp(0.0, 1.0);
                let shelf = smoothstep(0.0, 0.018, h);
                let r = 0.015 + 0.06 * (1.0 - deep);
                let g = 0.10 + 0.34 * (1.0 - deep) + 0.10 * shelf;
                let b = 0.22 + 0.34 * (1.0 - deep) + 0.06 * shelf;
                blend_ice(r, g, b, ice)
            } else {
                let v = fbm(dir, seed ^ 0x55, 4, 6.0) * 0.16;
                let vary = fbm(dir, seed ^ 0xa13, 3, 2.6) - 0.5;
                let alb = (al + 0.10 * vary).clamp(0.0, 1.0);
                let (mut r, mut g, mut b) = if alb < 0.40 {
                    (0.12 + v, 0.36 + v, 0.12)
                } else if alb < 0.55 {
                    (0.60 + v, 0.50 + v, 0.30)
                } else if alb < 0.80 {
                    (0.24 + v, 0.38 + v, 0.18)
                } else {
                    (0.34 + v, 0.34 + v, 0.27)
                };
                if h > 0.46 {
                    let m = ((h - 0.46) / 0.34).clamp(0.0, 1.0);
                    let rock = 0.40 + 0.18 * v;
                    r = r + (rock - r) * m;
                    g = g + (rock - g) * m;
                    b = b + (rock + 0.04 - b) * m;
                    let snow = ((h - 0.66) / 0.2).clamp(0.0, 1.0)
                        * (0.4 + 0.6 * al);
                    r += (0.95 - r) * snow;
                    g += (0.96 - g) * snow;
                    b += (0.98 - b) * snow;
                }
                blend_ice(r, g, b, ice)
            }
        }
        PlanetKind::GasBands { warm } => {
            let phi = dir[2].atan2(dir[0]);
            let festoon = fbm(dir, seed, 4, 4.0) * 0.10;
            let fine = fbm(dir, seed ^ 0x1234, 4, 9.0) * 0.5 + 0.5;
            let l = lat + festoon;
            let bandw = ((l * 17.0).sin()
                + 0.45 * (l * 31.0 + 1.3).sin())
                * 0.5
                + 0.5;
            let belt = (bandw - 0.5).abs() * 2.0;
            let zone = 1.0 - belt;
            let eq = (-(lat * lat) / 0.018).exp();
            let mut t = (zone * 0.78 + 0.22 * fine).clamp(0.0, 1.0);
            t = (t + 0.35 * eq).clamp(0.0, 1.0);
            let (mut r, mut g, mut b) = if warm {
                let zr = [0.86, 0.78, 0.60];
                let br = [0.52, 0.34, 0.22];
                (
                    br[0] + (zr[0] - br[0]) * t,
                    br[1] + (zr[1] - br[1]) * t,
                    br[2] + (zr[2] - br[2]) * t,
                )
            } else {
                let zr = [0.88, 0.82, 0.66];
                let br = [0.66, 0.56, 0.40];
                (
                    br[0] + (zr[0] - br[0]) * t,
                    br[1] + (zr[1] - br[1]) * t,
                    br[2] + (zr[2] - br[2]) * t,
                )
            };
            if warm {
                let lat0 = -0.37_f32;
                let phi0 = 2.1_f32;
                let mut dphi = phi - phi0;
                while dphi > std::f32::consts::PI {
                    dphi -= 2.0 * std::f32::consts::PI;
                }
                while dphi < -std::f32::consts::PI {
                    dphi += 2.0 * std::f32::consts::PI;
                }
                let e = ((dphi / 0.52).powi(2)
                    + ((lat - lat0) / 0.14).powi(2))
                .sqrt();
                if e < 1.0 {
                    let k = (1.0 - e).clamp(0.0, 1.0);
                    let swirl = fbm(dir, seed ^ 0x6a51, 4, 16.0) * 0.5
                        + 0.5;
                    let spot = [
                        0.82 + 0.14 * swirl,
                        0.34 + 0.16 * swirl,
                        0.17 + 0.10 * swirl,
                    ];
                    let m = (k * 1.5).min(1.0);
                    r = r + (spot[0] - r) * m;
                    g = g + (spot[1] - g) * m;
                    b = b + (spot[2] - b) * m;
                }
            }
            srgb(r, g, b)
        }
        PlanetKind::IceGiant { deep } => {
            if !deep {
                let band = (lat * 6.0).sin() * 0.5 + 0.5;
                let haze = fbm(dir, seed, 3, 3.0) * 0.04;
                let t = 0.94 + 0.05 * band + haze;
                return srgb(0.62 * t, 0.80 * t, 0.84 * t);
            }
            let phi = dir[2].atan2(dir[0]);
            let warp = fbm(dir, seed, 4, 3.5) * 0.06;
            let band = ((lat + warp) * 9.0).sin() * 0.5 + 0.5;
            let n = fbm(dir, seed ^ 0x77, 3, 4.0) * 0.06;
            let t = 0.82 + 0.16 * band + n;
            let cirrus = fbm(dir, seed ^ 0x2c1d, 4, 14.0);
            let streak =
                ((cirrus - 0.62).max(0.0) / 0.38).clamp(0.0, 1.0);
            let mut r = 0.13 * t;
            let mut g = 0.30 * t;
            let mut b = 0.74 * t;
            let lat0 = -0.45_f32;
            let phi0 = 1.0_f32;
            let mut dphi = phi - phi0;
            while dphi > std::f32::consts::PI {
                dphi -= 2.0 * std::f32::consts::PI;
            }
            while dphi < -std::f32::consts::PI {
                dphi += 2.0 * std::f32::consts::PI;
            }
            let e = ((dphi / 0.50).powi(2)
                + ((lat - lat0) / 0.16).powi(2))
            .sqrt();
            if e < 1.0 {
                let k = (1.0 - e).clamp(0.0, 1.0);
                let dk = (k * 1.6).min(1.0);
                r *= 1.0 - 0.62 * dk;
                g *= 1.0 - 0.62 * dk;
                b *= 1.0 - 0.55 * dk;
            }
            let comp = (((lat - (lat0 + 0.20)) / 0.07).powi(2)
                + (dphi / 0.34).powi(2))
            .sqrt();
            let bright =
                (streak * 0.6 + (1.0 - comp).max(0.0) * 0.9).min(1.0);
            r += bright * 0.55;
            g += bright * 0.60;
            b += bright * 0.55;
            srgb(r.min(1.0), g.min(1.0), b.min(1.0))
        }
        PlanetKind::Lava => {
            let crust = fbm(dir, seed, 5, 4.0);
            let crack = 1.0 - (2.0 * fbm(dir, seed ^ 0x1a7, 4, 6.0)
                - 1.0)
                .abs();
            let glow = smoothstep(0.62, 0.95, crack)
                + smoothstep(0.55, 0.85, crust) * 0.4;
            let g0 = glow.clamp(0.0, 1.0);
            let base = 0.05 + 0.05 * crust;
            srgb(
                base + g0 * 1.6,
                base * 0.7 + g0 * g0 * 0.7,
                base * 0.6 + g0 * g0 * g0 * 0.15,
            )
        }
        PlanetKind::Ocean => {
            let al = lat.abs();
            let depth = fbm(dir, seed, 5, 2.6);
            let shoal = smoothstep(0.55, 0.78, depth);
            let swirl = fbm(dir, seed ^ 0x9c, 4, 9.0) * 0.12;
            let r = 0.015 + 0.10 * shoal + swirl;
            let g = 0.16 + 0.42 * shoal + swirl;
            let b = 0.34 + 0.34 * shoal + swirl;
            let ice = ((al - 0.86).max(0.0) / 0.14).clamp(0.0, 1.0);
            blend_ice(r, g, b, ice)
        }
        PlanetKind::Desert => {
            let al = lat.abs();
            let warp = fbm(dir, seed, 4, 2.0) * 0.5;
            let dune = ((lat * 26.0 + warp).sin() * 0.5 + 0.5)
                * 0.18;
            let rock = fbm(dir, seed ^ 0x55, 4, 5.0);
            let dark = smoothstep(0.58, 0.82, rock) * 0.30;
            let v = 0.62 + dune + fbm(dir, seed ^ 0x7, 3, 11.0) * 0.10
                - dark;
            let frost = ((al - 0.90).max(0.0) / 0.10).clamp(0.0, 1.0)
                * 0.5;
            srgb(
                (v + 0.30 * frost).min(1.0),
                (v * 0.78 + 0.34 * frost).min(1.0),
                (v * 0.48 + 0.38 * frost).min(1.0),
            )
        }
        PlanetKind::Carbon => {
            let n = fbm(dir, seed, 5, 3.5);
            let tholin = smoothstep(0.5, 0.85, fbm(dir, seed ^ 0x3d, 3, 2.2));
            let sheen = smoothstep(0.7, 0.95, fbm(dir, seed ^ 0x9, 4, 13.0))
                * 0.16;
            let v = 0.07 + 0.06 * n + sheen;
            srgb(
                v + tholin * 0.10,
                v + tholin * 0.055,
                v + tholin * 0.03 + 0.012,
            )
        }
    }
}

fn blend_ice(r: f32, g: f32, b: f32, ice: f32) -> [u8; 4] {
    let k = ice.clamp(0.0, 1.0).powf(0.6);
    srgb(
        r + (0.92 - r) * k,
        g + (0.94 - g) * k,
        b + (0.97 - b) * k,
    )
}

pub fn blackbody_rgb(temp_k: f32) -> [f32; 3] {
    let t = temp_k.clamp(1000.0, 40000.0) / 100.0;
    let r;
    let g;
    let b;
    if t <= 66.0 {
        r = 1.0;
        g = (0.390_081_57 * t.ln() - 0.631_841_44).clamp(0.0, 1.0);
    } else {
        r = (1.292_936_2 * (t - 60.0).powf(-0.133_204_76)).clamp(0.0, 1.0);
        g = (1.129_890_9 * (t - 60.0).powf(-0.075_514_85)).clamp(0.0, 1.0);
    }
    if t >= 66.0 {
        b = 1.0;
    } else if t <= 19.0 {
        b = 0.0;
    } else {
        b = (0.543_206_8 * (t - 10.0).ln() - 1.196_254_1).clamp(0.0, 1.0);
    }
    [r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code_roundtrip(k: PlanetKind) -> PlanetKind {
        PlanetKind::from_code(&k.to_code())
    }

    #[test]
    fn planetkind_codec() {
        assert!(matches!(
            code_roundtrip(PlanetKind::Terran),
            PlanetKind::Terran
        ));
        assert!(matches!(
            code_roundtrip(PlanetKind::GasBands { warm: true }),
            PlanetKind::GasBands { warm: true }
        ));
        assert!(matches!(
            code_roundtrip(PlanetKind::GasBands { warm: false }),
            PlanetKind::GasBands { warm: false }
        ));
        assert!(matches!(
            code_roundtrip(PlanetKind::IceGiant { deep: true }),
            PlanetKind::IceGiant { deep: true }
        ));
        match code_roundtrip(PlanetKind::Rocky { r: 0.7, g: 0.5, b: 0.3 }) {
            PlanetKind::Rocky { r, g, b } => {
                assert!((r - 0.7).abs() < 1e-4);
                assert!((g - 0.5).abs() < 1e-4);
                assert!((b - 0.3).abs() < 1e-4);
            }
            _ => panic!("rocky roundtrip failed"),
        }
    }
}

pub fn make_cloud_texture(
    images: &mut bevy::prelude::Assets<Image>,
    seed: u32,
) -> bevy::prelude::Handle<Image> {
    let w = 256usize;
    let h = 128usize;
    let mut data = vec![0u8; w * h * 4];
    let pi = std::f32::consts::PI;
    for y in 0..h {
        let theta = ((y as f32 + 0.5) / h as f32) * pi;
        let st = theta.sin();
        let ct = theta.cos();
        for x in 0..w {
            let phi = ((x as f32 + 0.5) / w as f32) * 2.0 * pi;
            let dir = [st * phi.cos(), ct, st * phi.sin()];
            let n = fbm(dir, seed, 5, 3.0) * 0.5 + 0.5;
            let a = (((n - 0.52) / 0.30).clamp(0.0, 1.0)).powf(0.8);
            let o = (y * w + x) * 4;
            data[o] = 255;
            data[o + 1] = 255;
            data[o + 2] = 255;
            data[o + 3] = (a * 235.0) as u8;
        }
    }
    images.add(Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

pub fn make_ring_texture(
    images: &mut bevy::prelude::Assets<Image>,
    style: u8,
) -> bevy::prelude::Handle<Image> {
    let w = 1024usize;
    let h = 2usize;
    let mut data = vec![0u8; w * h * 4];
    let rn = |x: f32| -> f32 {
        let s = (x * 12.9898).sin() * 43758.547;
        s - s.floor()
    };
    let ringlet = |fr: f32, k: f32| -> f32 {
        let a = (fr * k).sin() * 0.5 + 0.5;
        let b = rn((fr * k * 0.37).floor());
        0.78 + 0.22 * a * (0.5 + 0.5 * b)
    };
    let narrow = |u: f32, c: f32, hw: f32| -> f32 {
        (1.0 - ((u - c).abs() / hw).min(1.0)).max(0.0)
    };
    for x in 0..w {
        let u = (x as f32 + 0.5) / w as f32;
        let (mut col, mut al);
        if style == 2 {
            col = [0.42, 0.32, 0.26];
            al = 0.035 * ringlet(u, 60.0);
            al += 0.10 * narrow(u, 0.82, 0.16) * ringlet(u, 220.0);
            al += 0.05 * narrow(u, 0.30, 0.30);
        } else if style == 3 {
            col = [0.24, 0.24, 0.27];
            al = 0.02;
            for &(c, hw, pk) in &[
                (0.30_f32, 0.012_f32, 0.16_f32),
                (0.46, 0.010, 0.14),
                (0.58, 0.013, 0.18),
                (0.70, 0.011, 0.16),
                (0.92, 0.022, 0.40),
            ] {
                al += pk * narrow(u, c, hw);
            }
        } else if style == 4 {
            col = [0.20, 0.23, 0.32];
            al = 0.025;
            al += 0.13 * narrow(u, 0.34, 0.018);
            al += 0.11 * narrow(u, 0.55, 0.015);
            let arc = 0.45 + 0.55 * ringlet(u, 320.0);
            al += 0.26 * narrow(u, 0.90, 0.020) * arc;
        } else {
            let fr = 1.30 + u * (2.30 - 1.30);
            if fr < 1.525 {
                col = [0.52, 0.47, 0.40];
                al = 0.16 * ringlet(fr, 520.0);
            } else if fr < 1.951 {
                col = [0.93, 0.86, 0.69];
                al = 0.94 * ringlet(fr, 900.0);
            } else if fr < 2.027 {
                col = [0.34, 0.22, 0.16];
                al = 0.05 * ringlet(fr, 260.0);
            } else if fr < 2.269 {
                col = [0.80, 0.79, 0.74];
                al = 0.64 * ringlet(fr, 760.0);
                if (fr - 2.214).abs() < 0.004 {
                    al *= 0.06;
                }
                if (fr - 2.265).abs() < 0.002 {
                    al *= 0.10;
                }
            } else {
                col = [0.6, 0.6, 0.6];
                al = (1.0 - ((fr - 2.269) / 0.03).clamp(0.0, 1.0))
                    * 0.10;
            }
        }
        let f = 0.85 + 0.15 * rn((u * 311.0).floor());
        col = [col[0] * f, col[1] * f, col[2] * f];
        for y in 0..h {
            let o = (y * w + x) * 4;
            data[o] = (col[0].clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 1] = (col[1].clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 2] = (col[2].clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 3] = (al.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    images.add(Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

pub fn make_star_texture(
    images: &mut bevy::prelude::Assets<Image>,
    seed: u32,
) -> bevy::prelude::Handle<Image> {
    let w = 384usize;
    let h = 192usize;
    let mut data = vec![0u8; w * h * 4];
    let pi = std::f32::consts::PI;
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        let theta = v * pi;
        let st = theta.sin();
        let ct = theta.cos();
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let phi = u * 2.0 * pi;
            let dir = [st * phi.cos(), ct, st * phi.sin()];
            let g = fbm(dir, seed, 5, 16.0) * 0.5 + 0.5;
            let fine = fbm(dir, seed ^ 0x51ed, 3, 42.0) * 0.5 + 0.5;
            let lane = (1.0 - (2.0 * g - 1.0).abs()).powf(1.6);
            let mut bri = 0.58 + 0.62 * g + 0.12 * fine - 0.30 * lane;
            let sp = fbm(dir, seed ^ 0x9e37, 4, 2.2) * 0.5 + 0.5;
            let spot = ((sp - 0.56) / 0.16).clamp(0.0, 1.0).powf(0.6);
            bri *= 1.0 - 0.97 * spot;
            let bri = bri.clamp(0.05, 1.25);
            let rr = bri;
            let gg = bri * (1.0 - 0.30 * spot);
            let bb = bri * (1.0 - 0.55 * spot);
            let o = (y * w + x) * 4;
            data[o] = (rr.clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 1] = (gg.clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 2] = (bb.clamp(0.0, 1.0) * 255.0) as u8;
            data[o + 3] = 255;
        }
    }
    images.add(Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

pub fn make_texture(
    images: &mut bevy::prelude::Assets<Image>,
    kind: PlanetKind,
    seed: u32,
) -> bevy::prelude::Handle<Image> {
    let w = 384usize;
    let h = 192usize;
    let mut data = vec![0u8; w * h * 4];
    let pi = std::f32::consts::PI;
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        let theta = v * pi;
        let st = theta.sin();
        let ct = theta.cos();
        let lat = ct;
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let phi = u * 2.0 * pi;
            let dir = [st * phi.cos(), ct, st * phi.sin()];
            let px = shade(kind, dir, lat, seed);
            let o = (y * w + x) * 4;
            data[o..o + 4].copy_from_slice(&px);
        }
    }
    images.add(Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

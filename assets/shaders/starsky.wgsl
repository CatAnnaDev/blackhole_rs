#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

fn hash33(p3in: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p3in * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn blackbody(temp_k: f32) -> vec3<f32> {
    let t = clamp(temp_k, 1000.0, 40000.0) / 100.0;
    var r: f32;
    var g: f32;
    var b: f32;
    if (t <= 66.0) {
        r = 1.0;
        g = clamp(0.39008157 * log(t) - 0.63184144, 0.0, 1.0);
    } else {
        r = clamp(1.29293618 * pow(t - 60.0, -0.1332047592), 0.0, 1.0);
        g = clamp(1.12989086 * pow(t - 60.0, -0.0755148492), 0.0, 1.0);
    }
    if (t >= 66.0) {
        b = 1.0;
    } else if (t <= 19.0) {
        b = 0.0;
    } else {
        b = clamp(0.54320679 * log(t - 10.0) - 1.19625408, 0.0, 1.0);
    }
    return vec3<f32>(r, g, b);
}

fn star_layer(d: vec3<f32>, grid: f32, density: f32, gain: f32) -> vec3<f32> {
    let p = d * grid;
    let cell = floor(p);
    let f = p - cell;
    var col = vec3<f32>(0.0);
    for (var oz = -1; oz <= 1; oz = oz + 1) {
        for (var oy = -1; oy <= 1; oy = oy + 1) {
            for (var ox = -1; ox <= 1; ox = ox + 1) {
                let g = vec3<f32>(f32(ox), f32(oy), f32(oz));
                let id = cell + g;
                let rnd = hash33(id);
                if (rnd.x > density) {
                    let sp = g + vec3<f32>(rnd.x, rnd.y, rnd.z);
                    let dist = length(f - sp);
                    let bright = pow((rnd.x - density) / (1.0 - density), 3.0);
                    let core = bright
                        * (smoothstep(0.09, 0.0, dist)
                            + 0.18 * smoothstep(0.35, 0.0, dist))
                        * gain;
                    let temp = mix(3200.0, 15000.0, rnd.z);
                    col += blackbody(temp) * core;
                }
            }
        }
    }
    return col;
}

fn vnoise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let n000 = hash33(i + vec3<f32>(0.0, 0.0, 0.0)).x;
    let n100 = hash33(i + vec3<f32>(1.0, 0.0, 0.0)).x;
    let n010 = hash33(i + vec3<f32>(0.0, 1.0, 0.0)).x;
    let n110 = hash33(i + vec3<f32>(1.0, 1.0, 0.0)).x;
    let n001 = hash33(i + vec3<f32>(0.0, 0.0, 1.0)).x;
    let n101 = hash33(i + vec3<f32>(1.0, 0.0, 1.0)).x;
    let n011 = hash33(i + vec3<f32>(0.0, 1.0, 1.0)).x;
    let n111 = hash33(i + vec3<f32>(1.0, 1.0, 1.0)).x;
    let x00 = mix(n000, n100, u.x);
    let x10 = mix(n010, n110, u.x);
    let x01 = mix(n001, n101, u.x);
    let x11 = mix(n011, n111, u.x);
    return mix(mix(x00, x10, u.y), mix(x01, x11, u.y), u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var s = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 5; i = i + 1) {
        s += a * vnoise(q);
        q = q * 2.02;
        a *= 0.5;
    }
    return s;
}

fn starfield(dir: vec3<f32>) -> vec3<f32> {
    let d = normalize(dir);
    var col = vec3<f32>(0.00035, 0.00045, 0.0007);

    let gp = d.y;
    let band = exp(-gp * gp * 9.0);
    let neb = fbm(d * 3.0);
    let neb2 = fbm(d * 7.0 + 11.0);
    let dust = smoothstep(0.45, 0.95, neb) * band;
    let glow = smoothstep(0.35, 0.9, neb2) * band;
    col += vec3<f32>(0.022, 0.018, 0.040) * dust;
    col += vec3<f32>(0.030, 0.024, 0.018) * glow * 0.6;
    col += vec3<f32>(0.010, 0.013, 0.020) * band * 0.5;

    col += star_layer(d, 18.0, 0.90, 3.0);
    col += star_layer(d, 40.0, 0.93, 1.6);
    col += star_layer(d, 90.0, 0.96, 0.9);
    col += star_layer(d, 170.0, 0.978, 0.5);
    return col;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(mesh.world_position.xyz - view.world_position);
    return vec4<f32>(starfield(dir), 1.0);
}

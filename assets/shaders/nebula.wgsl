#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Neb {
    col_a: vec4<f32>,
    col_b: vec4<f32>,
    p: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> neb: Neb;

fn h3(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

fn vn3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let c000 = h3(i + vec3<f32>(0.0, 0.0, 0.0));
    let c100 = h3(i + vec3<f32>(1.0, 0.0, 0.0));
    let c010 = h3(i + vec3<f32>(0.0, 1.0, 0.0));
    let c110 = h3(i + vec3<f32>(1.0, 1.0, 0.0));
    let c001 = h3(i + vec3<f32>(0.0, 0.0, 1.0));
    let c101 = h3(i + vec3<f32>(1.0, 0.0, 1.0));
    let c011 = h3(i + vec3<f32>(0.0, 1.0, 1.0));
    let c111 = h3(i + vec3<f32>(1.0, 1.0, 1.0));
    let x00 = mix(c000, c100, u.x);
    let x10 = mix(c010, c110, u.x);
    let x01 = mix(c001, c101, u.x);
    let x11 = mix(c011, c111, u.x);
    return mix(mix(x00, x10, u.y), mix(x01, x11, u.y), u.z);
}

fn fbm(p0: vec3<f32>) -> f32 {
    var s = 0.0;
    var a = 0.5;
    var q = p0;
    for (var i = 0; i < 5; i = i + 1) {
        s += a * vn3(q);
        q = q * 2.07 + vec3<f32>(3.1, 1.7, 5.3);
        a *= 0.5;
    }
    return s;
}

fn ridged(p0: vec3<f32>) -> f32 {
    var s = 0.0;
    var a = 0.5;
    var q = p0;
    for (var i = 0; i < 5; i = i + 1) {
        let r = 1.0 - abs(2.0 * vn3(q) - 1.0);
        s += a * r * r;
        q = q * 2.13 + vec3<f32>(1.9, 4.3, 2.7);
        a *= 0.5;
    }
    return s;
}

fn warp3(p: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        fbm(p + s),
        fbm(p + s + 19.3),
        fbm(p + s + 41.7),
    ) - 0.5;
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let t = neb.p.x;
    let scale = neb.p.y;
    let seed = neb.p.z;
    let dens = neb.p.w;
    let s3 = vec3<f32>(seed * 1.7, seed * 0.6 + 5.0, seed * 1.1 + 2.0);
    let drift = vec3<f32>(t * 0.006, t * -0.004, t * 0.005);

    let entry = normalize(m.world_normal);
    let rd = normalize(m.world_position.xyz - view.world_position);

    let jit = fract(sin(dot(m.position.xy, vec2<f32>(12.99, 78.23))) * 43758.55);
    let steps = 38;
    let span = 2.05;
    let dl = span / f32(steps);

    var col = vec3<f32>(0.0);
    var trans = 1.0;

    for (var i = 0; i < steps; i = i + 1) {
        let tt = (f32(i) + jit) * dl;
        let pos = entry + rd * tt;
        let r = length(pos);
        if (r > 1.06) {
            continue;
        }

        let w = warp3(pos * (scale * 0.42) + s3, s3);
        let q = pos * scale + s3 + drift + w * 2.6;

        let presence = pow(smoothstep(0.22, 0.62, fbm(pos * 0.95 + s3 * 0.5 + drift)), 0.85);
        if (presence <= 0.003) {
            continue;
        }

        let cloud = pow(smoothstep(0.16, 0.82, fbm(q)), 1.1);
        let fil = pow(clamp(ridged(q * 1.4 + 7.0), 0.0, 1.0), 3.2);
        let fine = fbm(q * 4.4 + 13.0);
        let lane = smoothstep(0.46, 0.80, fbm(q * 1.7 + 27.0));

        var d = (cloud * 1.05 + fil * 2.2 + fine * 0.15) * presence;
        d = d * (1.0 - 0.92 * lane);
        let edge = smoothstep(1.06, 0.48, r);
        d = clamp(d * edge, 0.0, 1.0);
        if (d <= 0.004) {
            continue;
        }

        let cmix = pow(clamp(fbm(q * 0.7 + 3.0) * 1.6, 0.0, 1.0), 1.3);
        var tint = mix(neb.col_a.rgb, neb.col_b.rgb, cmix);
        let core = pow(clamp(fil * cloud, 0.0, 1.0), 1.5);
        tint = mix(tint, vec3<f32>(0.40, 1.0, 0.82), core * 0.45);
        tint = mix(tint, vec3<f32>(0.45, 0.55, 1.0), pow(1.0 - edge, 3.0) * 0.30);
        let emis = tint * (0.7 + cloud * 1.8 + core * 6.5);

        let dd = d * dl * 11.0;
        col += trans * emis * dd;
        trans *= exp(-dd * (0.85 + lane * 3.0));
        if (trans < 0.015) {
            break;
        }
    }

    let lum = col.r + col.g + col.b;
    if (lum <= 0.002) {
        discard;
    }
    let alpha = clamp((1.0 - trans) * 0.95, 0.0, 1.0);
    return vec4<f32>(col * dens * 6.5, alpha);
}

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Sn {
    c0: vec4<f32>,
    c1: vec4<f32>,
    progress: f32,
    seed: f32,
    mode: f32,
    _b: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sn: Sn;

fn h33(p3in: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p3in * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn vnoise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let n000 = h33(i + vec3<f32>(0.0, 0.0, 0.0)).x;
    let n100 = h33(i + vec3<f32>(1.0, 0.0, 0.0)).x;
    let n010 = h33(i + vec3<f32>(0.0, 1.0, 0.0)).x;
    let n110 = h33(i + vec3<f32>(1.0, 1.0, 0.0)).x;
    let n001 = h33(i + vec3<f32>(0.0, 0.0, 1.0)).x;
    let n101 = h33(i + vec3<f32>(1.0, 0.0, 1.0)).x;
    let n011 = h33(i + vec3<f32>(0.0, 1.0, 1.0)).x;
    let n111 = h33(i + vec3<f32>(1.0, 1.0, 1.0)).x;
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
        q = q * 2.03 + 7.0;
        a *= 0.5;
    }
    return s;
}

// blackbody-ish temperature ramp: 1=blue-white hot .. 0=deep red
fn temp_ramp(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.30, 0.04, 0.03);   // cold ember
    let b = vec3<f32>(1.00, 0.42, 0.10);   // orange
    let c = vec3<f32>(1.00, 0.92, 0.70);   // yellow-white
    let d = vec3<f32>(0.75, 0.85, 1.00);   // blue-white
    let x = clamp(t, 0.0, 1.0);
    if (x < 0.5) {
        return mix(a, b, smoothstep(0.0, 0.5, x));
    } else if (x < 0.8) {
        return mix(b, c, smoothstep(0.5, 0.8, x));
    }
    return mix(c, d, smoothstep(0.8, 1.0, x));
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(m.world_normal);
    let vd = normalize(view.world_position - m.world_position.xyz);
    let fres = pow(1.0 - clamp(abs(dot(n, vd)), 0.0, 1.0), 2.0);
    let p = clamp(sn.progress, 0.0, 1.0);

    // domain-warped turbulence -> stretched filaments
    let s = vec3<f32>(sn.seed);
    let w = vec3<f32>(
        fbm(n * 2.0 + s),
        fbm(n * 2.0 + s + 19.0),
        fbm(n * 2.0 + s + 41.0),
    ) - 0.5;
    let q = n * 5.0 + s + w * 2.5 + n * p * 2.5;
    let turb = fbm(q);
    let fine = fbm(q * 3.3 + 7.0);
    let fil = pow(smoothstep(0.40, 0.80, turb), 2.0);
    let detail = smoothstep(0.5, 0.95, fine);
    let sparks = pow(smoothstep(0.86, 1.0, fbm(q * 6.0 + 31.0)), 3.0);

    // bipolar anisotropy (real SNe are asymmetric)
    let axis = normalize(vec3<f32>(0.3, 1.0, -0.2));
    let aniso = 0.55 + 0.45 * pow(abs(dot(n, axis)), 1.5);

    var col: vec3<f32>;
    var intensity: f32;
    var a: f32;

    if (sn.mode < 0.5) {
        // CORE : blinding flash, fast bright spike then decay
        let flash = pow(1.0 - p, 4.0) + 0.5 * exp(-p * 14.0);
        col = mix(vec3<f32>(1.0), sn.c1.rgb, p);
        intensity = flash * 9.0 * mix(0.6, 1.0, fres);
        a = clamp(flash * 1.3, 0.0, 1.0);
    } else if (sn.mode < 1.5) {
        // SHOCK : thin bright expanding blast front, see-through center
        let edge = pow(fres, 1.4);
        let temp = temp_ramp(1.0 - p * 0.85);
        let crackle = 0.6 + 0.4 * fil;
        col = mix(temp, sn.c1.rgb, p * 0.6);
        intensity = edge * (3.0 + 4.0 * (1.0 - p)) * crackle * aniso;
        a = clamp(edge * (1.0 - p * p) * crackle, 0.0, 1.0);
    } else {
        // EJECTA : turbulent filaments + sparks, cooling, gaps transparent
        let heat = clamp(0.25 + fil * 0.9 + sparks * 1.2 - p * 0.5, 0.0, 1.0);
        col = temp_ramp(heat) * mix(vec3<f32>(1.0), sn.c0.rgb, 0.3);
        let body = (fil * (1.4 + 2.2 * detail) + sparks * 6.0) * aniso;
        intensity = body * mix(0.25, 1.0, fres) * (1.0 - 0.4 * p);
        a = clamp(
            (fil * 0.8 * (0.4 + detail) + sparks)
                * mix(0.3, 1.0, fres)
                * (1.0 - p * p * p),
            0.0,
            1.0,
        );
    }

    return vec4<f32>(col * intensity, a);
}

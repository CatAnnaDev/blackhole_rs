#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Aur {
    col: vec4<f32>,
    p: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> aur: Aur;

fn hash1(x: f32) -> f32 {
    return fract(sin(x * 127.1) * 43758.5453);
}

fn n1(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(hash1(i), hash1(i + 1.0), u);
}

fn fb1(x0: f32) -> f32 {
    var s = 0.0;
    var a = 0.5;
    var x = x0;
    for (var i = 0; i < 5; i = i + 1) {
        s += a * n1(x);
        x = x * 2.03 + 1.7;
        a *= 0.5;
    }
    return s;
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let t = aur.p.x;
    let act = clamp(aur.p.y, 0.0, 1.0);
    let seed = aur.p.w;

    let n = normalize(m.world_normal);
    let lat = abs(asin(clamp(n.y, -1.0, 1.0)));
    let lon = atan2(n.z, n.x);

    let center = 1.21 - 0.30 * act;
    let width = 0.055 + 0.11 * act;
    let dl = (lat - center) / width;
    let oval = exp(-dl * dl);
    if (oval <= 0.01 || act <= 0.001) {
        discard;
    }

    let curtain = lon * 9.0 + seed + sin(t * 0.6 + lon * 3.0) * 0.4;
    let rays = pow(fb1(curtain), 1.8);
    let flick = 0.55 + 0.45 * fb1(curtain * 0.5 + t * 1.7);
    let drape = 0.30 + 0.70 * fb1(lon * 3.5 + seed * 2.0 + t * 0.25);

    let band = clamp(dl * 0.5 + 0.5, 0.0, 1.0);
    let green = aur.col.rgb;
    let red = vec3<f32>(1.0, 0.18, 0.30);
    let violet = vec3<f32>(0.40, 0.20, 1.0);
    var tint = green;
    tint = mix(tint, red, smoothstep(0.55, 1.0, band) * 0.8);
    tint = mix(tint, violet, smoothstep(0.45, 0.0, band) * 0.6);

    let vd = normalize(view.world_position - m.world_position.xyz);
    let limb = pow(1.0 - clamp(abs(dot(n, vd)), 0.0, 1.0), 1.4);

    let intensity = oval * (0.45 + rays * 1.7) * flick * drape
        * (0.55 + 0.9 * act) * (0.6 + 0.9 * limb) * 4.8;
    let alpha = clamp(oval * (0.3 + rays) * (0.4 + act), 0.0, 1.0);
    return vec4<f32>(tint * intensity, alpha);
}

#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Jet {
    color: vec4<f32>,
    p: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> jet: Jet;

fn h2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vn(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);
    let a = h2(i);
    let b = h2(i + vec2<f32>(1.0, 0.0));
    let c = h2(i + vec2<f32>(0.0, 1.0));
    let d = h2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var s = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 5; i = i + 1) {
        s += a * vn(q);
        q = q * 2.07 + 3.1;
        a *= 0.5;
    }
    return s;
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let t = jet.p.x;
    let kind = jet.p.z;
    let gain = jet.p.w;

    let along = clamp(m.uv.y, 0.0, 1.0);
    let ang = m.uv.x;
    let a2 = ang * 6.28318;

    let n = normalize(m.world_normal);
    let vd = normalize(view.world_position - m.world_position.xyz);
    let face = clamp(abs(dot(n, vd)), 0.0, 1.0);
    let limb = pow(1.0 - face, 2.2);

    let speed = mix(2.4, 1.4, kind);

    let s = along * mix(7.0, 4.0, kind) - t * speed;
    let kp = fract(s);
    let kn = h2(vec2<f32>(floor(s), 3.0));
    let knot = pow(1.0 - abs(2.0 * kp - 1.0), 7.0)
        * (0.45 + 0.55 * kn);

    let helix = sin(a2 * 2.0 + along * 26.0 - t * speed * 1.7);
    let strand = fbm(vec2<f32>(
        ang * mix(9.0, 5.0, kind) + helix * 0.4,
        along * 6.0 - t * speed * 0.5,
    ));
    let filaments = pow(smoothstep(0.52, 0.92, strand), 2.4);

    let collimate = 0.35 + 0.65 * exp(-along * mix(1.5, 1.0, kind));
    let base_hot = smoothstep(0.10, 0.0, along);
    let lobe = smoothstep(0.80, 0.97, along)
        * (1.0 - smoothstep(0.97, 1.0, along))
        * (0.6 + 0.8 * fbm(vec2<f32>(a2, t * 0.5)));
    let envelope = collimate * (1.0 - smoothstep(0.93, 1.0, along));

    let spine = limb * envelope;
    let glow = spine * (0.14 + filaments * 1.3 + knot * 4.4)
        + base_hot * 2.4 + lobe * 2.8;

    let heat = clamp(base_hot + knot * 0.6 + lobe * 0.5
        + (1.0 - along) * 0.3, 0.0, 1.0);
    let tint = mix(jet.color.rgb, vec3<f32>(1.0), heat * 0.7);

    let intensity = glow * gain;
    let alpha = clamp(
        spine * (filaments * 0.45 + knot * 1.5)
            + base_hot * 0.8 + lobe * 0.9,
        0.0,
        1.0,
    );
    if (alpha < 0.010) {
        discard;
    }
    return vec4<f32>(tint * intensity, alpha);
}

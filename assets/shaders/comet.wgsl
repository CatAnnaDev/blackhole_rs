#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Comet {
    color: vec4<f32>,
    p: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> comet: Comet;

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
    for (var i = 0; i < 4; i = i + 1) {
        s += a * vn(q);
        q = q * 2.06 + 2.7;
        a *= 0.5;
    }
    return s;
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let t = comet.p.x;
    let kind = comet.p.z;
    let gain = comet.p.w;

    if (kind < 1.5) {
        let ion = kind < 0.5;
        let pd = m.uv - vec2<f32>(0.5, 0.5);
        let d = clamp(length(pd) * 2.0, 0.0, 1.0);
        let head = d;
        let tip = 1.0 - d;
        let aa = atan2(pd.y, pd.x);

        let fade = pow(head, select(0.8, 1.25, ion));
        let freq = select(5.0, 15.0, ion);
        let str = fbm(vec2<f32>(aa * freq, tip * 6.0 - t * 0.6));
        let streak = pow(smoothstep(0.42, 0.92, str),
            select(1.5, 2.6, ion));
        let broad = 0.4 + 0.6 * fbm(vec2<f32>(aa * 3.0, tip * 2.0));
        let dens = select(fade * broad, fade * (0.22 + streak), ion);
        if (dens < 0.010) {
            discard;
        }
        let intensity = dens * gain;
        return vec4<f32>(comet.color.rgb * intensity,
            clamp(dens * select(1.3, 1.0, ion), 0.0, 1.0));
    }

    if (kind > 2.5) {
        let pd = m.uv - vec2<f32>(0.5, 0.5);
        let d = clamp(length(pd) * 2.0, 0.0, 1.0);
        let head = d;
        let tip = 1.0 - d;
        let aa = atan2(pd.y, pd.x);
        let fade = pow(head, 0.65);
        let broad = 0.30 + 0.70 * fbm(vec2<f32>(aa * 2.2, tip * 1.5 - t * 0.12));
        let dens = fade * broad * smoothstep(0.0, 0.18, head);
        if (dens < 0.008) {
            discard;
        }
        let intensity = dens * gain * 0.55;
        return vec4<f32>(comet.color.rgb * intensity,
            clamp(dens * 0.6, 0.0, 1.0));
    }

    let n = normalize(m.world_normal);
    let vd = normalize(view.world_position - m.world_position.xyz);
    let rim = 1.0 - clamp(abs(dot(n, vd)), 0.0, 1.0);
    let body = pow(clamp(1.0 - rim, 0.0, 1.0), 1.4);
    let glow = body * (0.6 + 0.4 * fbm(vec2<f32>(m.uv.x * 4.0, t * 0.2)));
    let intensity = glow * gain;
    return vec4<f32>(comet.color.rgb * intensity,
        clamp(glow, 0.0, 1.0));
}

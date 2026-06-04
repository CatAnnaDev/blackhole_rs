#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct Disk {
    hot: vec4<f32>,
    cool: vec4<f32>,
    p: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> disk: Disk;

const TAU: f32 = 6.28318530718;

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
        q = q * 2.05 + 4.3;
        a *= 0.5;
    }
    return s;
}

@fragment
fn fragment(m: VertexOutput) -> @location(0) vec4<f32> {
    let t = disk.p.x;
    let speed = disk.p.y;
    let seed = disk.p.z;
    let gain = disk.p.w;

    let rad = clamp(m.uv.x, 0.0, 1.0);
    let ang = m.uv.y * TAU;

    let omega = 1.0 / pow(rad + 0.10, 1.5);
    let phase = ang - omega * t * speed * 0.15;
    let spiral = 0.5 + 0.5 * sin(2.0 * phase + 9.0 * (rad + 0.05));

    let n1 = fbm(vec2<f32>(phase * 1.6 + seed, rad * 5.0));
    let n2 = fbm(vec2<f32>(phase * 4.0 - seed, rad * 9.0 + 3.0));
    let band = mix(0.45, 1.3, spiral) * (0.6 + 0.7 * n1);
    let clumps = pow(smoothstep(0.55, 0.95, n2), 2.0);

    let radial = pow(1.0 - rad, 1.4) + 0.15;
    let inner = smoothstep(0.0, 0.06, rad);
    let outer = smoothstep(1.0, 0.78, rad);
    let env = inner * outer;

    let heat = clamp(
        pow(0.10 / (rad + 0.02), 0.75) * 0.85 * (0.85 + 0.3 * n1),
        0.0,
        1.0,
    );
    let col0 = mix(disk.cool.rgb, disk.hot.rgb, heat);

    let nrm = normalize(m.world_normal);
    let vd = view.world_position - m.world_position.xyz;
    let vp = vd - nrm * dot(vd, nrm);
    var e1 = cross(nrm, vec3<f32>(0.0, 0.0, 1.0));
    if (length(e1) < 1.0e-4) {
        e1 = cross(nrm, vec3<f32>(1.0, 0.0, 0.0));
    }
    e1 = normalize(e1);
    let e2 = cross(nrm, e1);
    let rdir = cos(ang) * e1 + sin(ang) * e2;
    let tang = normalize(cross(nrm, rdir));
    let vpn = normalize(vp + vec3<f32>(1.0e-6));
    let dop = clamp(dot(tang, vpn), -1.0, 1.0)
        * (0.55 + 0.45 * (1.0 - rad));
    let boost = clamp(1.0 + 1.7 * dop, 0.15, 3.2);
    let beam = boost * boost * boost;
    let blue = vec3<f32>(0.55, 0.7, 1.0);
    let red = vec3<f32>(1.0, 0.45, 0.22);
    let col = col0 * mix(red, blue, clamp(0.5 + 0.5 * dop, 0.0, 1.0));

    let bright = (radial * band + clumps * 1.3) * env;
    let intensity = bright * gain * beam;
    let alpha = clamp(bright * 1.4, 0.0, 1.0);
    return vec4<f32>(col * intensity, alpha);
}

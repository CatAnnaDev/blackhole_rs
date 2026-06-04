#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// claude --resume 2b427873-ae3f-416a-8fd8-a4a56c49203a
struct BhParams {
    cam_pos: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    fwd: vec4<f32>,
    screen: vec4<f32>,
    disk: vec4<f32>,
    misc: vec4<f32>,
    opts: vec4<f32>,
    cfg: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> bh: BhParams;

const PI: f32 = 3.14159265358979;

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
                    let bright = pow((rnd.x - density) / (1.0 - density), 4.0);
                    let core = bright * smoothstep(0.5, 0.0, dist) * gain;
                    let temp = mix(2800.0, 14000.0, rnd.z);
                    col += blackbody(temp) * core;
                }
            }
        }
    }
    return col;
}

fn starfield(dir: vec3<f32>) -> vec3<f32> {
    let d = normalize(dir);
    var col = vec3<f32>(0.0);
    col += star_layer(d, 24.0, 0.86, 2.2);
    col += star_layer(d, 60.0, 0.93, 1.1);
    col += star_layer(d, 120.0, 0.965, 0.6);
    let band = exp(-pow(d.y * 2.5, 2.0));
    let dust = 0.5 + 0.5 * hash33(floor(d * 16.0)).x;
    col += vec3<f32>(0.030, 0.038, 0.060) * band * dust;
    col += vec3<f32>(0.004, 0.005, 0.009);
    return col;
}

fn sigma(r: f32, ct: f32, a: f32) -> f32 { return r * r + a * a * ct * ct; }
fn delta(r: f32, a: f32) -> f32 { return r * r - 2.0 * r + a * a; }
fn horizon(a: f32) -> f32 { return 1.0 + sqrt(max(1.0 - a * a, 0.0)); }

fn isco(a: f32) -> f32 {
    let z1 = 1.0 + pow(1.0 - a * a, 1.0 / 3.0)
        * (pow(1.0 + a, 1.0 / 3.0) + pow(1.0 - a, 1.0 / 3.0));
    let z2 = sqrt(3.0 * a * a + z1 * z1);
    return 3.0 + z2 - sqrt(max((3.0 - z1) * (3.0 + z1 + 2.0 * z2), 0.0));
}

fn page_thorne(r: f32, a: f32, r_ms: f32) -> f32 {
    let y = sqrt(r);
    let y_ms = sqrt(r_ms);
    let ac = acos(clamp(a, -1.0, 1.0)) / 3.0;
    let y1 = 2.0 * cos(ac - PI / 3.0);
    let y2 = 2.0 * cos(ac + PI / 3.0);
    let y3 = -2.0 * cos(ac);
    let eps = 1.0e-4;
    let lg = log(abs(y / y_ms) + eps);
    let l1 = log(abs((y - y1) / (y_ms - y1)) + eps);
    let l2 = log(abs((y - y2) / (y_ms - y2)) + eps);
    let l3 = log(abs((y - y3) / (y_ms - y3)) + eps);
    let c1 = 3.0 * (y1 - a) * (y1 - a) / (y1 * (y1 - y2) * (y1 - y3));
    let c2 = 3.0 * (y2 - a) * (y2 - a) / (y2 * (y2 - y1) * (y2 - y3));
    let c3 = 3.0 * (y3 - a) * (y3 - a) / (y3 * (y3 - y1) * (y3 - y2));
    let bracket = y - y_ms - 1.5 * a * lg - c1 * l1 - c2 * l2 - c3 * l3;
    let denom = y * (y * y * y - 3.0 * y + 2.0 * a);
    let flux = (3.0 / (2.0 * r)) * bracket / max(denom, eps);
    return max(flux, 0.0);
}

fn disk_redshift(r: f32, e: f32, l: f32, a: f32) -> f32 {
    let om = 1.0 / (pow(r, 1.5) + a);
    let g_tt = -(1.0 - 2.0 / r);
    let g_tphi = -2.0 * a / r;
    let g_phph = r * r + a * a + 2.0 * a * a / r;
    let denom = -(g_tt + 2.0 * om * g_tphi + om * om * g_phph);
    if (denom <= 1.0e-6) { return 1.0; }
    let ut = 1.0 / sqrt(denom);
    let bottom = ut * (e - l * om);
    if (abs(bottom) < 1.0e-6) { return 1.0; }
    return e / bottom;
}

fn hamiltonian(r: f32, th: f32, pr: f32, pth: f32, e: f32, l: f32, a: f32) -> f32 {
    let st = max(abs(sin(th)), 1.0e-4);
    let ct = cos(th);
    let s2 = st * st;
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let aa = (r * r + a * a) * (r * r + a * a) - a * a * dl * s2;
    let inv_sg = 1.0 / sg;
    let inv_dl = 1.0 / dl;
    let inv_sgdl = inv_sg * inv_dl;
    let inv_s2 = 1.0 / s2;
    let g_rr = dl * inv_sg;
    let g_thth = inv_sg;
    let g_tt = -aa * inv_sgdl;
    let g_tphi = -2.0 * a * r * inv_sgdl;
    let g_phph = (dl - a * a * s2) * inv_sgdl * inv_s2;
    let pt = -e;
    let pph = l;
    return 0.5 * (g_tt * pt * pt + 2.0 * g_tphi * pt * pph
        + g_phph * pph * pph + g_rr * pr * pr + g_thth * pth * pth);
}

fn deriv(state: vec4<f32>, e: f32, l: f32, a: f32) -> vec4<f32> {
    let r = state.x;
    let th = state.y;
    let pr = state.z;
    let pth = state.w;
    let ct = cos(th);
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let inv_sg = 1.0 / sg;
    let dr = dl * inv_sg * pr;
    let dth = inv_sg * pth;
    let hr = 1.0e-3 * max(r, 1.0);
    let hth = 1.0e-3;
    let inv_2hr = 0.5 / hr;
    let inv_2hth = 0.5 / hth;
    let dpr = -(hamiltonian(r + hr, th, pr, pth, e, l, a)
        - hamiltonian(r - hr, th, pr, pth, e, l, a)) * inv_2hr;
    let dpth = -(hamiltonian(r, th + hth, pr, pth, e, l, a)
        - hamiltonian(r, th - hth, pr, pth, e, l, a)) * inv_2hth;
    return vec4<f32>(dr, dth, dpr, dpth);
}

fn dphi(r: f32, th: f32, e: f32, l: f32, a: f32) -> f32 {
    let st = max(abs(sin(th)), 1.0e-4);
    let s2 = st * st;
    let ct = cos(th);
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let inv_sgdl = 1.0 / (sg * dl);
    let g_tphi = -2.0 * a * r * inv_sgdl;
    let g_phph = (dl - a * a * s2) * inv_sgdl / s2;
    return g_tphi * (-e) + g_phph * l;
}

struct Geo { state: vec4<f32>, phi: f32, e: f32, l: f32 };

fn init_geodesic(ro: vec3<f32>, rd: vec3<f32>, a: f32) -> Geo {
    let big_r2 = dot(ro, ro);
    let a2 = a * a;
    let r2 = 0.5 * ((big_r2 - a2) + sqrt(max((big_r2 - a2) * (big_r2 - a2)
        + 4.0 * a2 * ro.y * ro.y, 0.0)));
    let r = sqrt(max(r2, 1.0e-6));
    let th = acos(clamp(ro.y / r, -1.0, 1.0));
    let phi = atan2(ro.z, ro.x);
    let rho = sqrt(r * r + a2);
    let st = sin(th);
    let ct = cos(th);
    let cp = cos(phi);
    let sp = sin(phi);
    let er = normalize(vec3<f32>((r / rho) * st * cp, ct, (r / rho) * st * sp));
    let eth = normalize(vec3<f32>(rho * ct * cp, -r * st, rho * ct * sp));
    let eph = normalize(vec3<f32>(-rho * st * sp, 0.0, rho * st * cp));
    var n = normalize(vec3<f32>(dot(rd, er), dot(rd, eth), dot(rd, eph)));
    let s2 = max(st * st, 1.0e-6);
    let sg = sigma(r, ct, a);
    let dl = delta(r, a);
    let aa = (r * r + a2) * (r * r + a2) - a2 * dl * s2;
    let omega = 2.0 * a * r / aa;
    let alpha = sqrt(max(sg * dl / aa, 1.0e-12));
    let pup_t = 1.0 / alpha;
    let pup_r = sqrt(max(dl / sg, 0.0)) * n.x;
    let pup_th = sqrt(max(1.0 / sg, 0.0)) * n.y;
    let pup_ph = omega / alpha + sqrt(max(sg / aa, 0.0)) / max(st, 1.0e-4) * n.z;
    let g_tt = -(1.0 - 2.0 * r / sg);
    let g_tphi = -2.0 * a * r * s2 / sg;
    let g_phph = (aa / sg) * s2;
    let g_rr = sg / dl;
    let g_thth = sg;
    let p_t = g_tt * pup_t + g_tphi * pup_ph;
    let p_ph = g_tphi * pup_t + g_phph * pup_ph;
    let p_r = g_rr * pup_r;
    let p_th = g_thth * pup_th;
    var geo: Geo;
    geo.state = vec4<f32>(r, th, p_r, p_th);
    geo.phi = phi;
    geo.e = -p_t;
    geo.l = p_ph;
    return geo;
}

fn rk4(state: vec4<f32>, h: f32, e: f32, l: f32, a: f32) -> vec4<f32> {
    let k1 = deriv(state, e, l, a);
    let k2 = deriv(state + 0.5 * h * k1, e, l, a);
    let k3 = deriv(state + 0.5 * h * k2, e, l, a);
    let k4 = deriv(state + h * k3, e, l, a);
    return state + (h / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
}

fn to_cartesian(r: f32, th: f32, phi: f32, a: f32) -> vec3<f32> {
    let rho = sqrt(r * r + a * a);
    let st = sin(th);
    return vec3<f32>(rho * st * cos(phi), r * cos(th), rho * st * sin(phi));
}

fn vn2(x: f32, y: f32) -> f32 {
    let i = floor(vec2<f32>(x, y));
    let f = fract(vec2<f32>(x, y));
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash33(vec3<f32>(i, 1.0)).x;
    let b = hash33(vec3<f32>(i + vec2<f32>(1.0, 0.0), 1.0)).x;
    let c = hash33(vec3<f32>(i + vec2<f32>(0.0, 1.0), 1.0)).x;
    let d = hash33(vec3<f32>(i + vec2<f32>(1.0, 1.0), 1.0)).x;
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn disk_color(
    r: f32, e: f32, l: f32, a: f32, r_in: f32, r_out: f32, phi: f32,
) -> vec3<f32> {
    let flux = page_thorne(r, a, r_in);
    let fref = max(page_thorne(r_in * 2.2, a, r_in), 1.0e-3);
    let fnorm = clamp(flux / fref, 0.0, 24.0);
    let g = clamp(disk_redshift(r, e, l, a), 0.05, 4.0);

    let dop = clamp(bh.opts.x, 0.0, 1.0);
    let proc = clamp(bh.opts.y, 0.0, 1.0);
    let glow = max(bh.opts.z, 0.0);
    let gg = mix(1.0, g, dop);

    let t = bh.misc.w;
    let omega = 1.0 / (pow(r, 1.5) + a);
    let ang = phi - omega * t * 4.0e3;

    let spiral = 0.5 + 0.5 * sin(2.0 * ang + 6.5 * log(r));
    let n1 = vn2(ang * 4.0, r * 1.3);
    let n2 = vn2(ang * 9.0 - 3.0, r * 2.7 + 5.0);
    let pat = mix(0.45, 1.35, spiral) * (0.65 + 0.7 * n1);
    let pattern = mix(1.0, pat, proc);
    let clumps = pow(smoothstep(0.55, 0.95, n2), 2.0) * proc;
    let grain = mix(1.0, 0.9 + 0.25 * n1, proc);

    let t_emit = bh.disk.y * pow(fnorm, 0.25) * grain;
    let t_obs = clamp(t_emit * gg, 900.0, 42000.0);
    let bb = blackbody(t_obs);
    let beaming = pow(gg, 3.0);
    let base = pow(fnorm, 0.85) * beaming * bh.disk.x;
    let radiance = base * pattern + base * clumps * 1.6 + base * glow * 0.7;
    let soft = mix(6.0, 16.0, clamp(glow, 0.0, 1.0));
    let edge =
        smoothstep(r_in, r_in + 0.4, r) * smoothstep(r_out, r_out - soft, r);
    return bb * radiance * edge;
}

fn trace(ro: vec3<f32>, rd: vec3<f32>, a: f32) -> vec3<f32> {
    var geo = init_geodesic(ro, rd, a);
    let rh = horizon(a) * 1.001;
    let r_escape = max(length(ro) * 4.0, 60.0);
    let r_in = max(isco(a), bh.cfg.x);
    let r_out = bh.disk.z;
    let max_steps = i32(bh.misc.y);
    var prev = ro;
    var captured = false;
    for (var i = 0; i < max_steps; i = i + 1) {
        let r = geo.state.x;
        let th = geo.state.y;
        if (r <= rh) { captured = true; break; }
        if (r >= r_escape) { break; }
        let dist = r - rh;
        let h = clamp(0.10 * min(dist, r), 0.012, 2.0);
        let new_state = rk4(geo.state, h, geo.e, geo.l, a);
        let dph = dphi(geo.state.x, geo.state.y, geo.e, geo.l, a) * h;
        let s_old = th - 0.5 * PI;
        let s_new = new_state.y - 0.5 * PI;
        if (bh.opts.w > 0.5 && s_old * s_new < 0.0) {
            let frac = s_old / (s_old - s_new);
            let r_cross = mix(r, new_state.x, frac);
            if (r_cross >= r_in && r_cross <= r_out) {
                let phi_c = geo.phi + dph * frac;
                return disk_color(
                    r_cross, geo.e, geo.l, a, r_in, r_out, phi_c,
                );
            }
        }
        var nr = new_state.x;
        var nth = new_state.y;
        var npr = new_state.z;
        var npth = new_state.w;
        var nphi = geo.phi + dph;
        if (nth < 0.0) { nth = -nth; npth = -npth; nphi += PI; }
        else if (nth > PI) { nth = 2.0 * PI - nth; npth = -npth; nphi += PI; }
        prev = to_cartesian(r, th, geo.phi, a);
        geo.state = vec4<f32>(nr, nth, npr, npth);
        geo.phi = nphi;
    }
    if (captured) { return vec3<f32>(0.0); }
    let now = to_cartesian(geo.state.x, geo.state.y, geo.phi, a);
    return starfield(normalize(now - prev));
}

fn hash2(p: vec2<f32>, s: f32) -> vec2<f32> {
    let q = vec3<f32>(p, s);
    return fract(sin(vec2<f32>(
        dot(q, vec3<f32>(127.1, 311.7, 74.7)),
        dot(q, vec3<f32>(269.5, 183.3, 246.1))
    )) * 43758.5453);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tan_half = bh.screen.x;
    let aspect = bh.screen.y;
    let px = vec2<f32>(bh.screen.z, bh.screen.w);
    let a = clamp(bh.misc.x, 0.0, 0.999);
    let samples = max(i32(bh.misc.z), 1);

    var acc = vec3<f32>(0.0);
    for (var s = 0; s < samples; s = s + 1) {
        var uv = mesh.uv;
        if (samples > 1) {
            let j = hash2(mesh.position.xy, f32(s)) - 0.5;
            uv = uv + j * px;
        }
        let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
        let dir = normalize(
            bh.fwd.xyz
            + bh.right.xyz * (ndc.x * aspect * tan_half)
            + bh.up.xyz * (ndc.y * tan_half)
        );
        acc += trace(bh.cam_pos.xyz, dir, a);
    }

    let color = acc / f32(samples) * bh.disk.w;
    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}

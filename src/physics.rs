pub fn horizon(a: f64) -> f64 {
    1.0 + (1.0 - a * a).max(0.0).sqrt()
}

pub fn isco(a: f64) -> f64 {
    let z1 = 1.0
        + (1.0 - a * a).powf(1.0 / 3.0)
            * ((1.0 + a).powf(1.0 / 3.0) + (1.0 - a).powf(1.0 / 3.0));
    let z2 = (3.0 * a * a + z1 * z1).sqrt();
    3.0 + z2 - ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).max(0.0).sqrt()
}

pub fn photon_sphere_eq(a: f64) -> f64 {
    2.0 * (1.0 + ((2.0 / 3.0) * (-a).clamp(-1.0, 1.0).acos()).cos())
}

pub fn orbital_omega(r: f64, a: f64) -> f64 {
    1.0 / (r.powf(1.5) + a)
}

pub fn page_thorne(r: f64, a: f64, r_ms: f64) -> f64 {
    let y = r.sqrt();
    let y_ms = r_ms.sqrt();
    let ac = a.clamp(-1.0, 1.0).acos() / 3.0;
    let y1 = 2.0 * (ac - std::f64::consts::FRAC_PI_3).cos();
    let y2 = 2.0 * (ac + std::f64::consts::FRAC_PI_3).cos();
    let y3 = -2.0 * ac.cos();

    let eps = 1.0e-4;
    let lg = ((y / y_ms).abs() + eps).ln();
    let l1 = (((y - y1) / (y_ms - y1)).abs() + eps).ln();
    let l2 = (((y - y2) / (y_ms - y2)).abs() + eps).ln();
    let l3 = (((y - y3) / (y_ms - y3)).abs() + eps).ln();

    let c1 = 3.0 * (y1 - a) * (y1 - a) / (y1 * (y1 - y2) * (y1 - y3));
    let c2 = 3.0 * (y2 - a) * (y2 - a) / (y2 * (y2 - y1) * (y2 - y3));
    let c3 = 3.0 * (y3 - a) * (y3 - a) / (y3 * (y3 - y1) * (y3 - y2));

    let bracket = y - y_ms - 1.5 * a * lg - c1 * l1 - c2 * l2 - c3 * l3;
    let denom = y * (y * y * y - 3.0 * y + 2.0 * a);
    let flux = (3.0 / (2.0 * r)) * bracket / denom.max(eps);
    flux.max(0.0)
}

pub fn flux_peak(a: f64) -> (f64, f64) {
    let r_in = isco(a);
    let mut best = (r_in, 0.0);
    let mut r = r_in;
    while r < 60.0 {
        let f = page_thorne(r, a, r_in);
        if f > best.1 {
            best = (r, f);
        }
        r += 0.01;
    }
    best
}

pub fn merge_bodies(
    m1: f64,
    v1: [f64; 3],
    r1: f64,
    m2: f64,
    v2: [f64; 3],
    r2: f64,
) -> (f64, [f64; 3], f64) {
    let m = m1 + m2;
    let v = [
        (v1[0] * m1 + v2[0] * m2) / m,
        (v1[1] * m1 + v2[1] * m2) / m,
        (v1[2] * m1 + v2[2] * m2) / m,
    ];
    let r = (r1 * r1 * r1 + r2 * r2 * r2).cbrt();
    (m, v, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn merge_conserves_momentum_and_volume() {
        let (m, v, r) = merge_bodies(
            2.0,
            [10.0, 0.0, 0.0],
            1.0,
            2.0,
            [-10.0, 0.0, 0.0],
            1.0,
        );
        assert!(close(m, 4.0, 1e-9));
        assert!(close(v[0], 0.0, 1e-9));
        assert!(close(r, 2.0_f64.cbrt(), 1e-9));

        let (m2, v2, _) =
            merge_bodies(9.0, [0.0, 4.0, 0.0], 2.0, 1.0, [0.0, -6.0, 0.0], 1.0);
        assert!(close(m2, 10.0, 1e-9));
        assert!(close(v2[1], (9.0 * 4.0 - 6.0) / 10.0, 1e-9));
    }

    #[test]
    fn horizon_limits() {
        assert!(close(horizon(0.0), 2.0, 1e-9));
        assert!(close(horizon(1.0), 1.0, 1e-9));
        assert!(close(horizon(0.9), 1.0 + 0.19_f64.sqrt(), 1e-9));
    }

    #[test]
    fn isco_limits() {
        assert!(close(isco(0.0), 6.0, 1e-6));
        assert!(close(isco(1.0), 1.0, 1e-6));
        assert!(isco(0.9) > 2.0 && isco(0.9) < 2.6);
        assert!(isco(0.5) > isco(0.9));
    }

    #[test]
    fn photon_sphere_limits() {
        assert!(close(photon_sphere_eq(0.0), 3.0, 1e-6));
        assert!(close(photon_sphere_eq(1.0), 1.0, 1e-6));
    }

    #[test]
    fn flux_vanishes_at_isco_and_is_positive_outside() {
        for &a in &[0.0, 0.5, 0.9, 0.998] {
            let r_in = isco(a);
            assert!(page_thorne(r_in, a, r_in) < 1e-3, "a={a}");
            assert!(page_thorne(r_in * 1.5, a, r_in) > 0.0, "a={a}");
            assert!(
                page_thorne(r_in * 1.5, a, r_in) > page_thorne(40.0, a, r_in),
                "a={a}"
            );
        }
    }

    #[test]
    fn print_profiles() {
        for &a in &[0.0, 0.9, 0.998] {
            let r_in = isco(a);
            let (rp, fp) = flux_peak(a);
            println!(
                "a={a:.3} horizon={:.4} isco={:.4} peak_r={rp:.3} peak_F={fp:.5}",
                horizon(a),
                r_in
            );
            for k in 1..=8 {
                let r = r_in * (1.0 + 0.5 * k as f64);
                println!("  r={r:7.3}  F={:.6}", page_thorne(r, a, r_in));
            }
        }
    }
}

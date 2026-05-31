#![allow(dead_code)]

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    pub bounce: f32,
    pub duration: f32,
}

impl SpringParams {
    pub const fn new(bounce: f32, duration: f32) -> Self {
        Self { bounce, duration }
    }
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            bounce: 0.0,
            duration: 0.4,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Integrated RK4 with substeps
#[derive(Debug, Clone)]
pub struct Spring {
    stiffness: f32, // k = ω_n²
    damping: f32,   // c = 2·ζ·ω_n
    pub position: f32,
    velocity: f32,
    pub target: f32,
}

impl Spring {
    pub fn new(params: SpringParams, initial: f32) -> Self {
        let zeta = (1.0 - params.bounce.clamp(0.0, 0.99)).max(0.01);
        let omega_n = 2.0 * PI / params.duration.max(0.01);

        Self {
            stiffness: omega_n * omega_n,
            damping: 2.0 * zeta * omega_n,
            position: initial,
            velocity: 0.0,
            target: initial,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.min(1.0 / 20.0);
        let sub = dt * 0.25;
        for _ in 0..4 {
            let (dp, dv) = self.rk4(sub);
            self.position += dp;
            self.velocity += dv;
        }
    }

    pub fn is_settled(&self) -> bool {
        (self.position - self.target).abs() < 5e-4 && self.velocity.abs() < 1e-3
    }

    pub fn snap(&mut self) {
        self.position = self.target;
        self.velocity = 0.0;
    }

    // ── RK4 ──────────────────────────────────────────────────────────────────

    #[inline(always)]
    fn deriv(&self, p: f32, v: f32) -> (f32, f32) {
        let acc = -self.stiffness * (p - self.target) - self.damping * v;
        (v, acc) // (ṗ, v̇)
    }

    fn rk4(&self, h: f32) -> (f32, f32) {
        let h2 = h * 0.5;
        let (k1p, k1v) = self.deriv(self.position, self.velocity);
        let (k2p, k2v) = self.deriv(self.position + k1p * h2, self.velocity + k1v * h2);
        let (k3p, k3v) = self.deriv(self.position + k2p * h2, self.velocity + k2v * h2);
        let (k4p, k4v) = self.deriv(self.position + k3p * h, self.velocity + k3v * h);
        let w = h / 6.0;
        (
            (k1p + 2.0 * k2p + 2.0 * k3p + k4p) * w,
            (k1v + 2.0 * k2v + 2.0 * k3v + k4v) * w,
        )
    }
}

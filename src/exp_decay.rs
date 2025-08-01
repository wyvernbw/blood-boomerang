use bevy::prelude::*;

pub trait ExpDecay: Sized {
    fn exp_decay(self, rhs: Self, decay: f32, dt: f32) -> Self;
}

impl ExpDecay for f32 {
    fn exp_decay(self, rhs: Self, decay: f32, dt: f32) -> f32 {
        rhs + (self - rhs) * (-decay * dt).exp()
    }
}

impl ExpDecay for Quat {
    fn exp_decay(self, rhs: Self, decay: f32, dt: f32) -> Self {
        let t = 1.0 - (-decay * dt).exp(); // Exponential smoothing factor
        self.slerp(rhs, t)
    }
}

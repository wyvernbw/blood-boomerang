pub trait ExpDecay: Sized {
    fn exp_decay(self, rhs: Self, decay: Self, dt: Self) -> Self;
}

impl ExpDecay for f32 {
    fn exp_decay(self, rhs: Self, decay: Self, dt: Self) -> f32 {
        rhs + (self - rhs) * (-decay * dt).exp()
    }
}

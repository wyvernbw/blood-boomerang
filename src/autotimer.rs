use std::{marker::PhantomData, time::Duration};

use bevy::prelude::*;

pub mod prelude {
    pub use super::{AutoTimer, TimerOnce, TimerRepeating};
}

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct TimerOnce;
#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct TimerRepeating;

#[derive(Resource, Component, Debug, Clone)]
pub struct AutoTimer<const MILLIS: u64, W = TimerOnce> {
    inner: Timer,
    checked_finished: bool,
    _w: PhantomData<W>,
}

impl<const T: u64> Default for AutoTimer<T, TimerOnce> {
    fn default() -> Self {
        Self {
            inner: Timer::new(Duration::from_millis(T), TimerMode::Once),
            checked_finished: false,
            _w: PhantomData,
        }
    }
}

impl<const T: u64> Default for AutoTimer<T, TimerRepeating> {
    fn default() -> Self {
        Self {
            inner: Timer::new(Duration::from_millis(T), TimerMode::Repeating),
            checked_finished: false,
            _w: PhantomData,
        }
    }
}

impl<const T: u64, W> std::ops::Deref for AutoTimer<T, W> {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const T: u64, W> std::ops::DerefMut for AutoTimer<T, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

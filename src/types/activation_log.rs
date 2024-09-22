#![allow(unsafe_code)]

use core::cell::UnsafeCell;

use rtic_monotonics::Monotonic;

use crate::app::Mono;

use super::generic::TimeInstant;

const MOD: u8 = 100;

pub struct ActivationLog {
    entry: UnsafeCell<Result<(u8, TimeInstant), &'static str>>,
}

impl ActivationLog {
    pub fn write(&self, _cs: &cortex_m::interrupt::CriticalSection) {
        unsafe {
            *self.entry.get() = match *self.entry.get() {
                Ok(entry) => Ok(((entry.0 + 1) % MOD, Mono::now())),
                Err(_) => Ok((1, Mono::now())),
            }
        }
    }

    pub fn read(&self) -> Result<(u8, TimeInstant), &str> {
        unsafe { *self.entry.get() }
    }

    pub fn build() -> Self {
        ActivationLog {
            entry: UnsafeCell::new(Err("no external event server activation logged")),
        }
    }
}

unsafe impl Sync for ActivationLog {}

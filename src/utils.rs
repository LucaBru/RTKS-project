use crate::types::time_instant::TimeInstant;
use rtic_monotonics::Monotonic;

use crate::app::Mono;

pub fn get_instant() -> TimeInstant {
    cortex_m::interrupt::free(|_| Mono::now())
}

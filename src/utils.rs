#[allow(dead_code)]
use crate::types::time_instant::TimeInstant;
use rtic_monotonics::Monotonic;

use crate::app::Mono;

pub mod activation_condition {
    use core::sync::atomic::AtomicI32;
    use core::sync::atomic::Ordering;

    const ON_CALL_PROD_MOD: i32 = 5;
    const ACTIVATION_LOG_MOD: i32 = 1000;

    static ON_CALL_PROD_ACTIVATION_REQUEST: AtomicI32 = AtomicI32::new(0);
    static ACTIVATION_LOG_READ_REQUEST: AtomicI32 = AtomicI32::new(0);

    pub fn on_call_prod_activation_criterion() -> bool {
        ON_CALL_PROD_ACTIVATION_REQUEST.fetch_add(1, Ordering::Relaxed);
        ON_CALL_PROD_ACTIVATION_REQUEST.load(Ordering::Relaxed) % ON_CALL_PROD_MOD == 2
    }

    pub fn activation_log_reader_criterion() -> bool {
        ACTIVATION_LOG_READ_REQUEST.fetch_add(1, Ordering::Relaxed);
        ACTIVATION_LOG_READ_REQUEST.load(Ordering::Relaxed) % ACTIVATION_LOG_MOD == 0
    }
}

pub fn get_instant() -> TimeInstant {
    cortex_m::interrupt::free(|_| Mono::now())
}

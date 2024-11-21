#[allow(dead_code)]
use crate::types::generic::TimeInstant;
use rtic_monotonics::Monotonic;

use crate::app::Mono;

pub mod activation_condition {
    use core::sync::atomic::AtomicI32;
    use core::sync::atomic::Ordering;

    const ON_CALL_PROD_MOD: i32 = 5;
    const LOG_READER_MOD: i32 = 1000;
    const LOG_READER_ACTV_RATIO: i32 = 3;

    static ON_CALL_PROD_ACTV_REQUEST: AtomicI32 = AtomicI32::new(0);
    static LOG_READER_ACTV_REQUEST: AtomicI32 = AtomicI32::new(0);

    pub fn on_call_prod_activation_criterion() -> bool {
        ON_CALL_PROD_ACTV_REQUEST.fetch_add(1, Ordering::Relaxed);
        ON_CALL_PROD_ACTV_REQUEST.load(Ordering::Relaxed) % ON_CALL_PROD_MOD == 2
    }

    pub fn activation_log_reader_criterion() -> bool {
        LOG_READER_ACTV_REQUEST.fetch_add(1, Ordering::Relaxed);
        LOG_READER_ACTV_REQUEST.load(Ordering::Relaxed) % LOG_READER_MOD % LOG_READER_ACTV_RATIO
            == 0
    }
}

pub fn get_instant() -> TimeInstant {
    cortex_m::interrupt::free(|_| Mono::now())
}

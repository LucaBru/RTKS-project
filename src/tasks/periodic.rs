use crate::{
    constant::BUFFER_CAPACITY, types::production_workload::ProductionWorkload, utils::get_instant,
};
use cortex_m_semihosting::hprintln;
use lm3s6965::Interrupt;
use rtic_monotonics::{fugit::MillisDurationU32, Monotonic};
use rtic_sync::channel::Sender;

use crate::app::{self, Mono};

pub async fn regular_producer(
    cx: app::regular_producer::Context<'_>,
    mut sender: Sender<'static, u32, BUFFER_CAPACITY>,
) {
    const REGULAR_PRODUCER_WORKLOAD: i32 = 756;
    const ON_CALL_PRODUCER_WORKLOAD: i32 = 278;
    const PERIOD: MillisDurationU32 = 1000.millis();
    loop {
        let instant = get_instant();
        hprintln!("regular producer starts at { }", instant);
        let mut production_workload: ProductionWorkload = Default::default();
        production_workload.small_whetstone(REGULAR_PRODUCER_WORKLOAD);
        if let Err(_) = sender.try_send(ON_CALL_PRODUCER_WORKLOAD) {
            hprintln!("on call producer activation failed due to full buffer")
        }

        Mono::delay_until(instant + PERIOD).await;
    }
}

// needed to emit interrupt for UART2 which emulates the push of the button
// peripherals method is unimplemented in lm3s6965 hal create, so this is a copy paste of the mechanism used from previous colleagues
pub async fn emit_hardware_interrupt(cx: app::emit_hardware_interrupt::Context<'_>) {
    const PERIOD: MillisDurationU32 = 5000.millis();
    loop {
        let instant = get_instant();
        rtic::pend(Interrupt::UART2);
        Mono::delay_until(instant + PERIOD).await;
    }
}

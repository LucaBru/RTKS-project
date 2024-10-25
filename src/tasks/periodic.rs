use crate::{
    constant::BUFFER_CAPACITY,
    types::{generic::TimeInstant, production_workload::ProductionWorkload},
    utils::{activation_condition, get_instant},
};
use cortex_m_semihosting::hprintln;
use lm3s6965::Interrupt;
use rtic_monotonics::{fugit::MillisDurationU32, Monotonic};
use rtic_sync::channel::Sender;

use crate::app::{self, Mono};

pub async fn regular_producer(
    cx: app::regular_producer::Context<'_>,
    mut on_call_prod_sender: Sender<'static, u32, BUFFER_CAPACITY>,
    mut on_call_prod_time_sender: Sender<'static, TimeInstant, 1>,
    mut activation_log_reader_sender: Sender<'static, u32, 1>,
) {
    const REGULAR_PRODUCER_WORKLOAD: u32 = 756;
    const ON_CALL_PRODUCER_WORKLOAD: u32 = 278;
    const PERIOD: MillisDurationU32 = MillisDurationU32::millis(1000);
    let mut production_workload: ProductionWorkload = Default::default();
    
    let mut next_time = cx.shared.task_activation_time.clone();
    Mono::delay_until(next_time).await;

    loop {
        hprintln!("regular producer started at {}", next_time);
        next_time += PERIOD;
        production_workload.small_whetstone(REGULAR_PRODUCER_WORKLOAD);

        // This must be non-preempted in order to pass the correct time to on call producer and avoid time drifting
        cortex_m::interrupt::free(|_| if activation_condition::on_call_prod_activation_criterion()
            && let Err(_) = on_call_prod_sender.try_send(ON_CALL_PRODUCER_WORKLOAD,) && let Err(_) = on_call_prod_time_sender.try_send(get_instant())
            {
                hprintln!("on call producer activation failed due to full buffer")
            }
        );

        if activation_condition::activation_log_reader_criterion()
            && let Err(_) = activation_log_reader_sender.try_send(0)
        {
            hprintln!("activation log reader failed due to full buffer")
        }

        let final_instant = get_instant();
        hprintln!("regular producer finished at { }", final_instant);

        Mono::delay_until(next_time).await;
    }
}

// needed to emit interrupt for UART2 which emulates the push of the button
// peripherals method is unimplemented in lm3s6965 hal create, so this is a copy paste of the mechanism used from previous colleagues
pub async fn emit_hardware_interrupt(_: app::emit_hardware_interrupt::Context<'_>) {
    const PERIOD: MillisDurationU32 = MillisDurationU32::millis(5000);

    loop {
        let instant = get_instant();
        rtic::pend(Interrupt::UART2);
        Mono::delay_until(instant + PERIOD).await;
    }
}

use crate::constant::CAPACITY;
use cortex_m_semihosting::hprintln;
use lm3s6965::Interrupt;
use rtic_monotonics::Monotonic;
use rtic_sync::channel::Sender;

use crate::app::{self, Mono};

pub async fn regular_producer(
    cx: app::regular_producer::Context<'_>,
    mut sender: Sender<'static, u32, CAPACITY>,
) {
    loop {
        let instant = Mono::now(); //p
        hprintln!("regular producer starts at { }", instant);
        if let Err(_) = sender.try_send(cx.local.aux_work.clone()) {
            hprintln!("on call producer activation failed due to full buffer")
        }

        Mono::delay_until(instant + cx.local.period.clone()).await; //p
    }
}

// needed to emit interrupt for UART2 which emulates the push of the button
// peripherals method is unimplemented in lm3s6965 hal create, so this is a copy paste of the mechanism used from previous colleagues
pub async fn emit_hardware_interrupt(cx: app::emit_hardware_interrupt::Context<'_>) {
    let instant = Mono::now();
    rtic::pend(Interrupt::UART2);
    Mono::delay_until(instant + cx.local.push_btn_period.clone()).await;
}

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![feature(proc_macro_hygiene)]
#![feature(let_chains)]
//#![deny(missing_docs)]

mod constant;
mod tasks;
mod types;
mod utils;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2, TIMER_0A, TIMER_0B])]
mod app {

    use crate::constant::BUFFER_CAPACITY;
    use crate::tasks::periodic::*;
    use crate::tasks::sporadic::*;
    use crate::types::generic::ActivationEntry;

    use cortex_m_semihosting::hprintln;
    use panic_semihosting as _;
    use rtic_monotonics::systick::prelude::*;
    use rtic_sync::{
        channel::{Receiver, Sender},
        make_channel,
    };

    systick_monotonic!(Mono, 1000); // Mono is a monotonic timer that interrupts with rate 1khz, a.k.a 1 ms

    // shared resources
    #[shared]
    struct Shared {}

    // local resources
    #[local]
    struct Local {}

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");
        loop {}
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // 12 MHz is the clock rate (in QEMU) associated with Systick timer
        Mono::start(cx.core.SYST, 12_000_000);

        let (on_call_prod_sender, on_call_prod_recv) = make_channel!(u32, 5);
        let (actv_log_reader_sender, actv_log_reader_recv) = make_channel!(u32, 1);
        let (actv_log_sender, actv_log_recv) = make_channel!(ActivationEntry, 1);

        regular_producer::spawn(on_call_prod_sender, actv_log_reader_sender).ok();
        on_call_producer::spawn(on_call_prod_recv).ok();
        external_event_server::spawn(actv_log_sender).ok();
        activation_log_reader::spawn(actv_log_reader_recv, actv_log_recv).ok();

        (Shared {}, Local {})
    }

    extern "Rust" {

        #[task(priority = 6)]
        async fn regular_producer(
            cx: regular_producer::Context,
            mut send1: Sender<'static, u32, BUFFER_CAPACITY>,
            mut send2: Sender<'static, u32, 1>,
        );

        #[task(priority = 4)]
        async fn on_call_producer(
            cx: on_call_producer::Context,
            mut recv: Receiver<'static, u32, BUFFER_CAPACITY>,
        );

        // this task is a sporadic task that serve an aperiodic (hardware) interrupt
        #[task(priority = 7, local = [actv_counter: u8 = 0])]
        async fn external_event_server(
            mut cx: external_event_server::Context,
            mut send: Sender<'static, ActivationEntry, 1>,
        );

        #[task(priority = 8)]
        async fn emit_hardware_interrupt(cx: emit_hardware_interrupt::Context);

        #[task(priority = 2)]
        async fn activation_log_reader(
            mut cx: activation_log_reader::Context,
            mut recv1: Receiver<'static, u32, 1>,
            mut recv2: Receiver<'static, ActivationEntry, 1>,
        );
    }
}

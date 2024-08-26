#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![feature(proc_macro_hygiene)]
//#![deny(missing_docs)]

mod constant;
mod tasks;
mod types;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2, TIMER_0A, TIMER_0B], peripherals = true)]
mod app {

    use crate::constant::CAPACITY;
    use crate::tasks::periodic::*;
    use crate::tasks::sporadic::*;
    use crate::types::activation_log::ActivationLog;

    use cortex_m_semihosting::hprintln;
    use panic_semihosting as _;
    use rtic_monotonics::{fugit::MillisDurationU32, systick::prelude::*};
    use rtic_sync::{
        channel::{Receiver, Sender},
        make_channel,
    };

    systick_monotonic!(Mono, 1000);

    // shared resources
    #[shared]
    struct Shared {
        activation_log: ActivationLog,
    }

    // local resources
    #[local]
    struct Local {
        // regular producer
        aux_work: u32,
        period: MillisDurationU32,

        // on call producer
        on_call_prod_min_sep: MillisDurationU32,
        push_butt_min_sep: MillisDurationU32,

        // hardware interrupt
        push_btn_period: MillisDurationU32,

        log_reader_min_sep: MillisDurationU32,
    }

    // runs before any other task and returns resources
    // this function disable preemption
    // `#[init]` cannot access locals from the `#[local]` struct as they are initialized here.

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");
        loop {}
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        Mono::start(cx.core.SYST, 12_000_000);

        let (s, r) = make_channel!(u32, 5);

        regular_producer::spawn(s).ok();
        on_call_producer::spawn(r).ok();
        push_button_server::spawn().ok();
        log_reader::spawn().ok();

        (
            Shared {
                activation_log: ActivationLog {
                    counter: 0,
                    time: Mono::now(),
                },
            },
            // initial values for the `#[local]` resources
            Local {
                aux_work: 100,
                period: 1000.millis(),

                on_call_prod_min_sep: 3000.millis(),
                push_butt_min_sep: 5000.millis(),

                push_btn_period: 5000.millis(),

                log_reader_min_sep: 3000.millis(),
            },
        )
    }

    extern "Rust" {

        #[task(priority = 6, local = [aux_work, period])]
        async fn regular_producer(
            cx: regular_producer::Context,
            mut sender: Sender<'static, u32, CAPACITY>,
        );

        #[task(priority = 4, local = [on_call_prod_min_sep])]
        async fn on_call_producer(
            cx: on_call_producer::Context,
            mut receiver: Receiver<'static, u32, CAPACITY>,
        );

        // this task is a sporadic task that serve an aperiodic (hardware) interrupt
        #[task(priority = 7, local = [push_butt_min_sep], shared = [activation_log])]
        async fn push_button_server(mut cx: push_button_server::Context);

        #[task(priority = 8, local = [push_btn_period])]
        async fn emit_hardware_interrupt(cx: emit_hardware_interrupt::Context);

        #[task(priority = 2, shared = [activation_log], local = [log_reader_min_sep])]
        async fn log_reader(mut cx: log_reader::Context);
    }
}

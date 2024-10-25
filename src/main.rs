#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![feature(proc_macro_hygiene)]
#![feature(let_chains)]

mod constant;
mod tasks;
mod types;
mod utils;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2, TIMER_0A, TIMER_0B])]
mod app {

    use crate::constant::BUFFER_CAPACITY;
    use crate::tasks::periodic::*;
    use crate::tasks::sporadic::*;
    use crate::types::activation_log::ActivationLog;
    use crate::types::generic::TimeInstant;

    use cortex_m_semihosting::hprintln;
    use fugit::MillisDurationU32;
    use panic_semihosting as _;
    use rtic_monotonics::systick::prelude::*;
    use rtic_sync::channel::Sender;
    use rtic_sync::{
        channel::Receiver,
        make_channel,
    };

    systick_monotonic!(Mono, 1000); // Mono is a monotonic timer that interrupts with rate 1khz, a.k.a 1 ms

    // shared resources
    #[shared]
    struct Shared {
        actv_log: ActivationLog,
        task_activation_time: TimeInstant,
    }

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
        let (on_call_prod_time_sender, on_call_prod_time_receiver) = make_channel!(TimeInstant, 1);
        let (actv_log_reader_sender, actv_log_reader_recv) = make_channel!(u32, 1);

        regular_producer::spawn(on_call_prod_sender, on_call_prod_time_sender, actv_log_reader_sender).ok();
        on_call_producer::spawn(on_call_prod_recv, on_call_prod_time_receiver).ok();
        external_event_server::spawn().ok();
        activation_log_reader::spawn(actv_log_reader_recv).ok();

        let system_activation_time: MillisDurationU32 = MillisDurationU32::millis(151);
        let task_activation_time: TimeInstant = Mono::now() + system_activation_time;
        (
            Shared {
                actv_log: ActivationLog::build(),
                task_activation_time,
            },
            Local {},
        )
    }

    extern "Rust" {

        #[task(priority = 6, shared = [&task_activation_time])]
        async fn regular_producer(
            cx: regular_producer::Context,
            mut send1: Sender<'static, u32, BUFFER_CAPACITY>,
            mut send2: Sender<'static, TimeInstant, 1>,
            mut send3: Sender<'static, u32, 1>,
        );

        #[task(priority = 4, shared = [&task_activation_time])]
        async fn on_call_producer(
            cx: on_call_producer::Context,
            mut recv: Receiver<'static, u32, BUFFER_CAPACITY>,
            mut time_receiver: Receiver<'static, TimeInstant, 1>
        );

        // this task is a sporadic task that serve an aperiodic (hardware) interrupt
        #[task(priority = 7, shared = [&actv_log, &task_activation_time])]
        async fn external_event_server(mut cx: external_event_server::Context);

        #[task(priority = 8, shared = [&task_activation_time])]
        async fn emit_hardware_interrupt(cx: emit_hardware_interrupt::Context);

        #[task(priority = 2, shared = [&actv_log, &task_activation_time])]
        async fn activation_log_reader(
            mut cx: activation_log_reader::Context,
            mut recv1: Receiver<'static, u32, 1>,
        );
    }
}

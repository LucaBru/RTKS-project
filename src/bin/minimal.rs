#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
//#![deny(missing_docs)]

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2, TIMER_0A, TIMER_0B], peripherals = true)]
mod app {

    use cortex_m_semihosting::hprintln;
    use fugit::Instant;
    use lm3s6965::Interrupt;
    use panic_semihosting as _;
    use rtic_monotonics::{fugit::MillisDurationU32, systick::prelude::*};
    use rtic_sync::{
        channel::{Receiver, Sender},
        make_channel,
    };

    systick_monotonic!(Mono, 1000);

    const CAPACITY: usize = 5;

    pub struct ActivationLog {
        counter: u8, // is mod 100
        time: Instant<u32, 1, 1000>,
    }

    impl ActivationLog {
        fn write(&mut self) -> () {
            self.counter += 1;
        }

        fn read(&self) -> (u8, Instant<u32, 1, 1000>) {
            (self.counter, self.time)
        }
    }

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
    }

    // runs before any other task and returns resources
    // this function disable preemption
    // `#[init]` cannot access locals from the `#[local]` struct as they are initialized here.
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
            },
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");
        loop {}
    }

    #[task(priority = 6, local = [aux_work, period])]
    async fn regular_producer(
        cx: regular_producer::Context,
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

    #[task(priority = 4, local = [on_call_prod_min_sep])]
    async fn on_call_producer(
        cx: on_call_producer::Context,
        mut receiver: Receiver<'static, u32, CAPACITY>,
    ) {
        while let Ok(work) = receiver.recv().await {
            let instant = Mono::now(); //p
            hprintln!("on call producer starts at { }", instant);
            hprintln!("on call producer executes { } work", work);
            Mono::delay_until(instant + cx.local.on_call_prod_min_sep.clone()).await
            //p
        }
    }

    // this task is a sporadic task that serve an aperiodic (hardware) interrupt
    #[task(priority = 7, local = [push_butt_min_sep], shared = [activation_log])]
    async fn push_button_server(mut cx: push_button_server::Context) {
        loop {
            let instant = Mono::now();
            hprintln!("push button server starts at { }", instant);
            cx.shared
                .activation_log
                .lock(|activation_log| activation_log.write());
            Mono::delay_until(instant + cx.local.push_butt_min_sep.clone()).await;
        }
    }

    #[task(priority = 2, shared = [activation_log])]
    async fn log_reader(mut cx: log_reader::Context) {
        loop {
            let instant = Mono::now();
            hprintln!("log reader starts at { }", instant);
            // activation_log must be mut due to rtic, since reads don't require the object to be mutable
            // one possibility is to implements the write operation using locks and then shared only & references
            cx.shared.activation_log.lock(|activation_log| {
                let (reads, time) = activation_log.read();
                hprintln!(
                    "Reader reads activation number { } at time { }",
                    reads,
                    time
                );
            })
        }
    }

    // needed to emit interrupt for UART2 which emulates the push of the button
    // peripherals method is unimplemented in lm3s6965 hal create, so this is a copy paste of the mechanism used from previous colleagues
    #[task(priority = 8)]
    async fn emit_hardware_interrupt(_: emit_hardware_interrupt::Context) {
        let instant = Mono::now();
        rtic::pend(Interrupt::UART2);
        Mono::delay_until(instant + 5000.millis()).await;
    }
}

// note that priority = theoretical priority - 1 due to priorities value that can be in [0, 8]

//p comments at the end of a line means that the task can be preempted there, but we don't want to

// THOUGHTS
//
// periodic task must be scheduled due to interrupt emitted by a timer, so read chapter 9.4 of lm3s6965 datasheet to configure a periodic timer
// note that this is out of rtic scope.
//
// release time for sporadic task on call producer must guarantee time constraints => when a message is received the current instant must be returned

/*
    What's next:
        - creates the share resource 'request_buffer' which is used by regular_producer to deposit work and by on_call_producer to fetch its.
        - makes regular_producer able to deposit work to request_buffer
        - makes on_call_producer sporadic to retrieve works from request_buffer

General guidelines:
    - tasks priority are sorted in ascending order (if no priority if specified => 0 is the default value)
    - all tasks (except init and idle) run as interrupt handlers
    - tasks bounded to interrupts are hardware task
*/

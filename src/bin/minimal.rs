#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
//#![deny(missing_docs)]

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2], peripherals = true)]
mod app {

    use cortex_m_semihosting::hprintln;
    use panic_semihosting as _;
    use rtic_monotonics::{fugit::MillisDurationU32, systick::prelude::*};
    use rtic_sync::{
        channel::{Receiver, Sender},
        make_channel,
    };

    systick_monotonic!(Mono, 1000);

    const CAPACITY: usize = 5;

    // shared resources
    #[shared]
    struct Shared {}

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
    // `#[init]` cannot access locals from the `#[local]` struct as they are initialized here.
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        Mono::start(cx.core.SYST, 12_000_000);

        let (s, r) = make_channel!(u32, 5);

        regular_producer::spawn(s).ok();
        on_call_producer::spawn(r).ok();
        push_button::spawn().ok();

        (
            Shared {},
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

    #[task(priority = 7, local = [aux_work, period])]
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

    #[task(priority = 5, local = [on_call_prod_min_sep])]
    async fn on_call_producer(
        cx: on_call_producer::Context,
        mut receiver: Receiver<'static, u32, CAPACITY>,
    ) {
        while let Ok(work) = receiver.recv().await {
            let instant = Mono::now(); //p
            hprintln!("on call producer starts at { }", instant);
            hprintln!("on call producer execute { } work", work);
            Mono::delay_until(instant + cx.local.on_call_prod_min_sep.clone()).await
            //p
        }
    }
}

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

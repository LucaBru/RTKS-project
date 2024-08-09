#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
//#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1, UART2], peripherals = true)]
mod app {

    use cortex_m_semihosting::hprintln;
    use rtic_monotonics::{fugit::MillisDurationU32, systick::prelude::*};

    systick_monotonic!(Mono, 1000);

    // shared resources
    #[shared]
    struct Shared {}

    // local resources
    #[local]
    struct Local {}

    // runs before any other task and returns resources
    // `#[init]` cannot access locals from the `#[local]` struct as they are initialized here.
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        Mono::start(cx.core.SYST, 12_000_000);

        regular_producer::spawn().ok();

        (
            Shared {},
            // initial values for the `#[local]` resources
            Local {},
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");
        loop {}
    }

    #[task(priority = 7)]
    async fn regular_producer(_: regular_producer::Context) {
        let period: MillisDurationU32 = 1000.millis();
        loop {
            hprintln!("regular producer");
            Mono::delay_until(Mono::now() + period).await;
        }
    }
}

/*
General guidelines:
    - tasks priority are sorted in ascending order (if no priority if specified => 0 is the default value)
    - all tasks (except init and idle) run as interrupt handlers
    - tasks bounded to interrupts are hardware task
*/

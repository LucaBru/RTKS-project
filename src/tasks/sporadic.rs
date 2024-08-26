use crate::{constant::CAPACITY, utils::get_instant};
use cortex_m_semihosting::hprintln;
use rtic::Mutex;
use rtic_monotonics::Monotonic;
use rtic_sync::channel::Receiver;

use crate::app::{self, Mono};

pub async fn on_call_producer(
    cx: app::on_call_producer::Context<'_>,
    mut receiver: Receiver<'static, u32, CAPACITY>,
) {
    while let Ok(work) = receiver.recv().await {
        // here task can be preempted, in that case the task suffers jitter
        let instant = get_instant();
        hprintln!("on call producer starts at { }", instant);
        hprintln!("on call producer executes { } work", work);
        Mono::delay_until(instant + cx.local.on_call_prod_min_sep.clone()).await
    }
}

pub async fn push_button_server(mut cx: app::push_button_server::Context<'_>) {
    loop {
        let instant = get_instant();
        hprintln!("push button server starts at { }", instant);
        cx.shared
            .activation_log
            .lock(|activation_log| activation_log.write());
        Mono::delay_until(instant + cx.local.push_butt_min_sep.clone()).await;
    }
}

pub async fn log_reader(mut cx: app::log_reader::Context<'_>) {
    loop {
        let instant = get_instant();
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
        });
        Mono::delay_until(instant + cx.local.log_reader_min_sep.clone()).await;
    }
}

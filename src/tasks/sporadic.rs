use crate::{constant::BUFFER_CAPACITY, utils::get_instant};
use cortex_m_semihosting::hprintln;
use rtic::Mutex;
use rtic_monotonics::{fugit::MillisDurationU32, Monotonic};
use rtic_sync::channel::Receiver;

use crate::app::{self, Mono};

pub async fn on_call_producer(
    _: app::on_call_producer::Context<'_>,
    mut receiver: Receiver<'static, u32, BUFFER_CAPACITY>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(3000);
    while let Ok(work) = receiver.recv().await {
        // here task can be preempted, in that case it suffers jitter
        let instant = get_instant();
        hprintln!("on call producer starts at { }", instant);
        hprintln!("on call producer executes { } work", work);
        Mono::delay_until(instant + MIN_SEP).await
    }
}

pub async fn push_button_server(mut cx: app::push_button_server::Context<'_>) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(5000);
    loop {
        let instant = get_instant();
        hprintln!("push button server starts at { }", instant);
        cx.shared
            .activation_log
            .lock(|activation_log| activation_log.write());
        Mono::delay_until(instant + MIN_SEP).await;
    }
}

pub async fn activation_log_reader(
    mut cx: app::activation_log_reader::Context<'_>,
    mut recv: Receiver<'static, u32, 1>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(3000);
    while let Ok(_) = recv.recv().await {
        // as on_call_producer here task can be preempted
        let instant = get_instant();
        hprintln!("activation log reader starts at { }", instant);
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
        Mono::delay_until(instant + MIN_SEP).await;
    }
}

use crate::{
    constant::BUFFER_CAPACITY, types::production_workload::ProductionWorkload, utils::get_instant,
};
use cortex_m_semihosting::hprintln;
use rtic_monotonics::{fugit::MillisDurationU32, Monotonic};
use rtic_sync::channel::Receiver;

use crate::app::{self, Mono};

pub async fn on_call_producer(
    _: app::on_call_producer::Context<'_>,
    mut receiver: Receiver<'static, u32, BUFFER_CAPACITY>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(3000);
    let mut production_workload: ProductionWorkload = Default::default();

    while let Ok(work) = receiver.recv().await {
        // here task can be preempted, in that case it suffers jitter
        let instant = get_instant();
        hprintln!("on call producer started at {}", instant);

        production_workload.small_whetstone(work);

        let final_instant = get_instant();
        hprintln!("on call producer finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await
    }
}

pub async fn external_event_server(cx: app::external_event_server::Context<'_>) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(5000);
    loop {
        let instant = get_instant();
        hprintln!("push button server started at {}", instant);

        cortex_m::interrupt::free(|cs| cx.shared.actv_log.write(cs));

        let final_instant = get_instant();
        hprintln!("push button server finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await;
    }
}

pub async fn activation_log_reader(
    cx: app::activation_log_reader::Context<'_>,
    mut actv_recv: Receiver<'static, u32, 1>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(3000);
    const WORKLOAD: u32 = 139;
    let mut production_workload: ProductionWorkload = Default::default();

    while let Ok(_) = actv_recv.recv().await {
        // as on_call_producer here task can be preempted
        let instant = get_instant();
        hprintln!("activation log reader started at {}", instant);

        production_workload.small_whetstone(WORKLOAD);

        match cx.shared.actv_log.read() {
            Ok((last_actv_counter, last_actv_instant)) => hprintln!(
                "Read activation number {} logged at time {}",
                last_actv_counter,
                last_actv_instant
            ),
            Err(err) => hprintln!("{}", err),
        }

        let final_instant = get_instant();
        hprintln!("activation log reader finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await;
    }
}

use crate::{
    constant::BUFFER_CAPACITY,
    types::{generic::ActivationEntry, production_workload::ProductionWorkload},
    utils::get_instant,
};
use cortex_m_semihosting::hprintln;
use rtic_monotonics::{fugit::MillisDurationU32, Monotonic};
use rtic_sync::channel::{Receiver, Sender};

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
        hprintln!("on call producer has executed {} kilo whets of work", work);

        let final_instant = get_instant();
        hprintln!("on call producer finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await
    }
}

pub async fn external_event_server(
    cx: app::external_event_server::Context<'_>,
    mut actv_log_sender: Sender<'static, ActivationEntry, 1>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(5000);
    const ACTV_COUNTER_MOD: u8 = 100;

    loop {
        let instant = get_instant();
        hprintln!("push button server started at {}", instant);

        *cx.local.actv_counter = (*cx.local.actv_counter + 1) % ACTV_COUNTER_MOD;
        let _ = actv_log_sender.try_send((*cx.local.actv_counter, Mono::now()));

        let final_instant = get_instant();
        hprintln!("push button server finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await;
    }
}

pub async fn activation_log_reader(
    _: app::activation_log_reader::Context<'_>,
    mut actv_recv: Receiver<'static, u32, 1>,
    mut actv_log_recv: Receiver<'static, ActivationEntry, 1>,
) {
    const MIN_SEP: MillisDurationU32 = MillisDurationU32::millis(3000);

    while let Ok(_) = actv_recv.recv().await {
        // as on_call_producer here task can be preempted
        let instant = get_instant();
        hprintln!("activation log reader started at {}", instant);

        match actv_log_recv.try_recv() {
            Ok(activation_entry) => hprintln!("Activation entry: {:?}", activation_entry),
            _ => hprintln!("activation log is empty, read failed"),
        }

        let final_instant = get_instant();
        hprintln!("activation log reader finished at {}", final_instant);

        Mono::delay_until(instant + MIN_SEP).await;
    }
}

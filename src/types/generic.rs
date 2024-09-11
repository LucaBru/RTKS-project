use rtic_monotonics::fugit::Instant;

pub type TimeInstant = Instant<u32, 1, 1000>;
pub type ActivationEntry = (u8, TimeInstant);

use rtic_monotonics::fugit::Instant;

pub type TimeInstant = Instant<u32, 1, 1000>;

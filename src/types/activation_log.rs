use rtic_monotonics::fugit::Instant;

pub struct ActivationLog {
    pub counter: u8, // is mod 100
    pub time: Instant<u32, 1, 1000>,
}

impl ActivationLog {
    pub fn write(&mut self) -> () {
        self.counter = (self.counter + 1) % 100;
    }

    pub fn read(&self) -> (u8, Instant<u32, 1, 1000>) {
        (self.counter, self.time)
    }
}

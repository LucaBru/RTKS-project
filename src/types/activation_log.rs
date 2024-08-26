use crate::types::time_instant::TimeInstant;



pub struct ActivationLog {
    pub counter: u8, // is mod 100
    pub time: TimeInstant,
}

impl ActivationLog {
    pub fn write(&mut self) -> () {
        self.counter = (self.counter + 1) % 100;
    }

    pub fn read(&self) -> (u8, TimeInstant) {
        (self.counter, self.time)
    }
}

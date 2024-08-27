#![deny(dead_code)]

use cortex_m_semihosting::hprintln;
use libm;

pub struct ProductionWorkload {
        pub t: f64,
        pub t1: f64,
        pub t2: f64, 
        pub n8: i32,
        pub n9: usize, 
        pub value: f64,
        pub tolerance: f64, 
        pub _i: i32,
        pub ij: i32,
        pub ik: i32,
        pub il: i32, 
        pub y: f64,
        pub z: f64,
        pub sum: f64,
        pub e1: [f64; 7],
}

impl Default for ProductionWorkload {
    fn default() -> Self {
        Self { 
            t: 0.499975,
            t1: 0.50025,
            t2: 2.0, 
            n8: 10,
            n9: 7,
            value: 0.941377,
            tolerance: 0.00001, 
            _i: 0,
            ij: 1,
            ik: 2,
            il: 3, 
            y: 1.0,
            z: 0.0,
            sum: 0.0,
            e1: [0.0; 7],
         }
    }
}

impl ProductionWorkload {
    fn clear_array(&mut self) {
        for element in self.e1.iter_mut() {
            *element = 0.0;
        }
    }

    fn _p0(&mut self) {
        if self.ij < 1 || self.ik < 1 || self.il < 1 {
            hprintln!("Parameter error 1");
            self.ij = 1;
            self.ik = 1;
            self.il = 1;
        } else if self.ij > self.n9.try_into().unwrap() || self.ik > self.n9.try_into().unwrap() || self.il > self.n9.try_into().unwrap() {
            hprintln!("Parameter error 2");
            self.ij = self.n9.try_into().unwrap();
            self.ik = self.n9.try_into().unwrap();
            self.il = self.n9.try_into().unwrap();
        }

        self.e1[self.ij as usize] = self.e1[self.ik as usize];
        self.e1[self.ik as usize] = self.e1[self.il as usize];
        self.e1[self._i as usize] = self.e1[self.ij as usize];
    }

    fn p3(&mut self, x: f64, y: f64, z: f64) -> f64 {
        let x_temp: f64 = self.t * (z + x);
        let y_temp: f64 = self.t * (x_temp + y);
        return (x_temp + y_temp) / self.t2;
    }

    pub fn small_whetstone(&mut self, kilo_whets: i32) {

        for _outer_loop_var in 1..kilo_whets {
            self.clear_array();
            self.ij = (self.ik - self.ij) * (self.il - self.ik);
            self.ik = self.il - (self.ik - self.ij);
            self.il = (self.il - self.ik) * (self.ik + self.il);
            if (self.ik - 1) < 1 || (self.il - 1) < 1 {
                hprintln!("Parameter error 3");
            } else if (self.ik - 1) > self.n9.try_into().unwrap() || (self.il - 1) > self.n9.try_into().unwrap() {
                hprintln!("Parameter error 4");
            } else {
                self.e1[(self.il - 1) as usize] = (self.ij + self.ik + self.il) as f64;
                self.e1[(self.ik - 1) as usize] = libm::sin(self.il as f64) as f64;
            }

            self.z = self.e1[4];

            for inner_loop_var in 1..self.n8 {
                self.z = self.p3(self.y * (inner_loop_var as f64), self.y + self.z, self.z);
            }

            self.ij = self.il - (self.il - 3) * self.ik;
            self.il = (self.il - self.ik) * (self.ik - self.ij);
            self.ik = (self.il - self.ik) * self.ik;

            if (self.il - 1) < 1 {
                hprintln!("Parameter error 5");
            } else if (self.il - 1) > self.n9 as i32 {
                hprintln!("Parameter error 6");
            } else {
                self.e1[(self.il - 1) as usize] = (self.ij + self.ik + self.il) as f64;
            }

            if (self.ik + 1) > self.n9 as i32 {
                hprintln!("Parameter error 7");
            } else if (self.ik + 1) < 1 {
                hprintln!("Parameter error 8");
            } else {
                self.e1[(self.ik +1) as usize] = libm::fabs(libm::cos(self.z));
            }

            self.z = libm::sqrt(libm::exp(libm::log(self.e1[self.n9 - 1]) / self.t1));

            self.sum += self.sum + self.z;

            if libm::fabs(self.z - self.value) > self.tolerance {
                self.sum = 2.0 * self.sum;
            }
        }
    }
}
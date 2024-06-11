pub trait Cooler: Clone {
    fn cool(&mut self);
    fn reset(&mut self);
    fn get_temp(&self) -> f64;
}

#[derive(Clone, Copy)]
pub struct QuadraticCooler {
    initial_temp: f64,
    temperature: f64,
    multiplier: f64,
}

impl Cooler for QuadraticCooler {
    fn cool(&mut self) {
        self.temperature *= self.multiplier;
    }
    fn get_temp(&self) -> f64 {
        self.temperature
    }

    fn reset(&mut self) {
        self.temperature = self.initial_temp;
    }
}

impl QuadraticCooler {
    pub fn new(temp: f64, multiplier: f64) -> Self {
        Self {
            temperature: temp,
            initial_temp: temp,
            multiplier,
        }
    }
}

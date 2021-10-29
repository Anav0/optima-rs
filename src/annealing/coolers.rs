pub trait Cooler {
    fn cool(&mut self);
    fn get_temp(&self) -> f64;
}
pub struct QuadriaticCooler {
    temperature: f64,
    multiplier: f64,
}

impl Cooler for QuadriaticCooler {
    fn cool(&mut self) {
        self.temperature *= self.multiplier;
    }
    fn get_temp(&self) -> f64 {
        self.temperature
    }
}

impl QuadriaticCooler {
    pub fn new(temp: f64, multiplier: f64) -> Self {
        Self {
            temperature: temp,
            multiplier,
        }
    }
}

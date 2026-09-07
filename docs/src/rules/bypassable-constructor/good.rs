pub struct Percentage(f64);

impl Percentage {
    pub fn new(n: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&n) {
            return Err(ValidationError::OutOfRange(n));
        }
        Ok(Self(n))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

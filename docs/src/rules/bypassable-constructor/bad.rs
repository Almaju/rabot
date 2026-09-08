pub struct Percentage(pub f64);

impl Percentage {
    pub fn new(n: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&n) {
            return Err(ValidationError::OutOfRange(n));
        }
        Ok(Self(n))
    }
}

impl Cart {
    fn apply_discount(&mut self) {
        self.discount = Percentage(250.0); // never went through the door
    }
}

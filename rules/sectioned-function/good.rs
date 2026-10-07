impl Orders {
    fn process(&self, order: &mut Order) -> Result<(), ProcessError> {
        order.validate()?;
        let total = order.total();
        self.store.save(order, total)
    }
}

impl Order {
    fn total(&self) -> Money {
        self.items.iter().map(Item::price).sum()
    }

    fn validate(&self) -> Result<(), ProcessError> {
        if self.items.is_empty() {
            return Err(ProcessError::Empty);
        }
        Ok(())
    }
}

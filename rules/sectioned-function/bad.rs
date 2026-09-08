impl Orders {
    fn process(&self, order: &mut Order) -> Result<(), ProcessError> {
        // step 1: validate
        if order.items.is_empty() {
            return Err(ProcessError::Empty);
        }
        // step 2: transform
        let total = order.items.iter().map(Item::price).sum();
        // step 3: persist
        self.store.save(order, total)
    }
}

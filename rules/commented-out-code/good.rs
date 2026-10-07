fn total(items: &[Item]) -> Money {
    items.iter().map(|i| i.price).sum()
}

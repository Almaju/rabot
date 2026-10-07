fn total(items: &[Item]) -> Money {
    // let discount = apply_coupon(&items);
    // items.iter().map(|i| i.price - discount).sum()
    items.iter().map(|i| i.price).sum()
}

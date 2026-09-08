enum OrderStatus {
    Approved,
    Pending,
    Rejected,
}

struct Order {
    status: OrderStatus,
}

impl Order {
    fn ship_if_approved(self, warehouse: &Warehouse) {
        match self.status {
            OrderStatus::Approved => warehouse.ship(self),
            OrderStatus::Pending | OrderStatus::Rejected => {}
        }
    }
}

struct Order {
    status: String, // "pending" | "approved" | "rejected"
}

impl Order {
    fn ship_if_approved(self, warehouse: &Warehouse) {
        if self.status == "aproved" {
            warehouse.ship(self); // never ships
        }
    }
}

struct User {
    billing: BillingProfile,
    events: Vec<UserEvent>,
    notifications: NotificationSettings,
}

impl User {
    fn ban(&mut self) {
        self.events.push(UserEvent::Banned);
    }

    fn promote(&mut self) {
        self.events.push(UserEvent::Promoted);
    }
}

struct BillingProfile {
    ledger: Vec<Charge>,
}

impl BillingProfile {
    fn charge(&mut self, amount: Money) {
        self.ledger.push(Charge::Debit(amount));
    }

    fn invoice(&self) -> Invoice {
        Invoice::from_ledger(&self.ledger)
    }

    fn refund(&mut self, amount: Money) {
        self.ledger.push(Charge::Credit(amount));
    }
}

struct NotificationSettings {
    channels: Vec<Channel>,
}

impl NotificationSettings {
    fn notify(&self, message: Message) {
        for channel in &self.channels {
            channel.send(&message);
        }
    }

    fn subscribe(&mut self, channel: Channel) {
        self.channels.push(channel);
    }

    fn unsubscribe(&mut self, channel: &Channel) {
        self.channels.retain(|existing| existing != channel);
    }
}

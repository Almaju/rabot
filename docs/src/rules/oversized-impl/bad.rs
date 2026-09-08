struct User {
    events: Vec<UserEvent>,
}

impl User {
    fn activate(&mut self) {
        self.events.push(UserEvent::Activate);
    }

    fn archive(&mut self) {
        self.events.push(UserEvent::Archive);
    }

    fn ban(&mut self) {
        self.events.push(UserEvent::Ban);
    }

    fn charge(&mut self, amount: Money) {
        self.events.push(UserEvent::Charge(amount));
    }

    fn deactivate(&mut self) {
        self.events.push(UserEvent::Deactivate);
    }

    fn delete(&mut self) {
        self.events.push(UserEvent::Delete);
    }

    fn export(&mut self) {
        self.events.push(UserEvent::Export);
    }

    fn invite(&mut self) {
        self.events.push(UserEvent::Invite);
    }

    fn invoice(&mut self, amount: Money) {
        self.events.push(UserEvent::Invoice(amount));
    }

    fn lock(&mut self) {
        self.events.push(UserEvent::Lock);
    }

    fn notify(&mut self, message: Message) {
        self.events.push(UserEvent::Notified(message));
    }

    fn promote(&mut self) {
        self.events.push(UserEvent::Promote);
    }

    fn refund(&mut self, amount: Money) {
        self.events.push(UserEvent::Refund(amount));
    }

    fn rename(&mut self, name: UserName) {
        self.events.push(UserEvent::Renamed(name));
    }

    fn restore(&mut self) {
        self.events.push(UserEvent::Restore);
    }

    fn subscribe(&mut self) {
        self.events.push(UserEvent::Subscribe);
    }

    fn suspend(&mut self) {
        self.events.push(UserEvent::Suspend);
    }

    fn unban(&mut self) {
        self.events.push(UserEvent::Unban);
    }

    fn unlock(&mut self) {
        self.events.push(UserEvent::Unlock);
    }

    fn unsubscribe(&mut self) {
        self.events.push(UserEvent::Unsubscribe);
    }

    fn upgrade(&mut self) {
        self.events.push(UserEvent::Upgrade);
    }
}

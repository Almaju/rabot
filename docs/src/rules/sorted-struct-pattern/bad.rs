impl User {
    fn greeting(&self) -> String {
        let User { name, email, .. } = self;
        format!("{name} <{email}>")
    }
}

impl Event {
    fn describe(&self) -> String {
        match self {
            Event::Moved { to, from, at } => format!("{from} -> {to} at {at}"),
        }
    }
}

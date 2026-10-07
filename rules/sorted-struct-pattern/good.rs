impl User {
    fn greeting(&self) -> String {
        let User { email, name, .. } = self;
        format!("{name} <{email}>")
    }
}

impl Event {
    fn describe(&self) -> String {
        match self {
            Event::Moved { at, from, to } => format!("{from} -> {to} at {at}"),
        }
    }
}

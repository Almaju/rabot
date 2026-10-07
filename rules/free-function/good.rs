struct User {
    banned: bool,
    name: UserName,
}

impl User {
    fn ban(&mut self) {
        self.banned = true;
    }

    fn display_name(&self) -> String {
        format!("{}", self.name)
    }
}

struct Url(String);

impl Url {
    fn parse(s: &str) -> Result<Self, ParseError> {
        Ok(Self(s.to_string()))
    }
}

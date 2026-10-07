struct User {
    banned: bool,
    name: UserName,
}

struct Url(String);

fn ban(user: &mut User) {
    user.banned = true;
}

fn parse_url(s: &str) -> Result<Url, ParseError> {
    Ok(Url(s.to_string()))
}

fn format_user(user: &User) -> String {
    format!("{}", user.name)
}

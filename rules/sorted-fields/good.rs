struct User {
    created_at: DateTime, // metadata at the end
    email: Email,
    id: UserId, // primary key first, obviously
    last_login_at: Option<DateTime>,
    name: UserName,
    phone_number: Option<PhoneNumber>, // where does this go?
    updated_at: DateTime,
}

struct User {
    id: UserId, // primary key first, obviously
    email: Email,
    name: UserName,
    created_at: DateTime, // metadata at the end
    updated_at: DateTime,
    last_login_at: Option<DateTime>,
    phone_number: Option<PhoneNumber>, // where does this go?
}

use std::fmt;

#[derive(Serialize, Debug, Clone)]
pub struct User {
    id: String,           // Primary key goes first, obviously
    email: String,
    name: String,
    /// When the account was created.
    created_at: u64,
    #[serde(default)]
    updated_at: u64,
    last_login_at: Option<u64>,
}

pub enum UserRole {
    Member,
    Admin,
    Guest { since: u64, invited_by: String },
}

// rabot: allow(sorted-variants) severity order is used in comparisons
pub enum Severity {
    Low,
    High,
}

#[derive(PartialOrd, PartialEq)]
pub enum Ordered {
    Second,
    First,
}

#[repr(C)]
pub struct Layout {
    z: u8,
    a: u8,
}

pub struct Store {
    users: Vec<User>,
}

impl Store {
    fn find(&self, id: &str) -> Option<&User> {
        let User { name, email, .. } = &self.users[0];
        match self.users.first() {
            Some(User { id: found, created_at, .. }) if found == id => None,
            _ => None,
        }
    }

    /// Removes a user.
    pub fn delete_user(&mut self, id: &str) {}

    pub fn create_user(&mut self, user: User) {
        let stored = User { role: user.role, name: user.name, id: user.id, email: user.email, created_at: 0, updated_at: 0, last_login_at: None };
        self.users.push(stored);
    }

    // The constructor.
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    fn effects(&self) -> User {
        User { name: fetch_name(), id: fetch_id(), email: String::new(), created_at: 0, updated_at: 0, last_login_at: None }
    }
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

trait Persist {
    fn save(&self);
    type Error;
    fn load(&self);
    const TABLE: &'static str;
}

fn fetch_name() -> String {
    String::new()
}

fn fetch_id() -> String {
    String::new()
}

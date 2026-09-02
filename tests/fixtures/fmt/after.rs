use std::fmt;

#[derive(Clone, Debug, Serialize)]
pub struct User {
    /// When the account was created.
    created_at: u64,
    email: String,
    id: String, // Primary key goes first, obviously
    last_login_at: Option<u64>,
    name: String,
    #[serde(default)]
    updated_at: u64,
}

pub enum UserRole {
    Admin,
    Guest { invited_by: String, since: u64 },
    Member,
}

// rabot: allow(sorted-variants) severity order is used in comparisons
pub enum Severity {
    Low,
    High,
}

#[derive(PartialEq, PartialOrd)]
pub enum Ordered {
    Second,
    First,
}

#[repr(C)]
pub struct Layout {
    z: u8,
    a: u8,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Store {
    users: Vec<User>,
}

impl Store {
    // The constructor.
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }

    pub fn create_user(&mut self, user: User) {
        let stored = User { created_at: 0, email: user.email, id: user.id, last_login_at: None, name: user.name, role: user.role, updated_at: 0 };
        self.users.push(stored);
    }

    /// Removes a user.
    pub fn delete_user(&mut self, id: &str) {}

    fn effects(&self) -> User {
        User { name: fetch_name(), id: fetch_id(), email: String::new(), created_at: 0, updated_at: 0, last_login_at: None }
    }

    fn find(&self, id: &str) -> Option<&User> {
        let User { email, name, .. } = &self.users[0];
        match self.users.first() {
            Some(User { created_at, id: found, .. }) if found == id => None,
            _ => None,
        }
    }
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

trait Persist {
    const TABLE: &'static str;
    type Error;
    fn load(&self);
    fn save(&self);
}

fn fetch_name() -> String {
    String::new()
}

fn fetch_id() -> String {
    String::new()
}

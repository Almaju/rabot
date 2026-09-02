//! One violation per rule, so the integration test can pin each one to a line.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

mod utils;

static DATABASE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static LOGGER: OnceLock<String> = OnceLock::new();

#[derive(Serialize, Debug, Clone)]
pub struct User {
    id: String,
    email: String,
}

pub struct CreateUserRequest {
    email: String,
    user_id: String,
}

pub struct Email(String);

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

// rabot: allow(sorted-fields)
pub struct Undocumented {
    b: u8,
    a: u8,
}

// rabot: allow(no-such-rule) whatever
pub struct UserService {
    db: String,
}

impl UserService {
    fn validate(&self) -> bool {
        true
    }
    pub fn delete_user(&self, id: &str) {}
    pub fn new(db: String) -> Self {
        Self { db }
    }
}

pub fn send_invoice(user_id: String, email: String, invoice_id: String) {
    let x = std::env::var("X").unwrap();
    // let old = compute(5);
    // send_email(&user_id, &invoice_id);
    // TODO: refactor this
    // TODO: this linear scan works at current scale but will need an index once we hit the enterprise tier. See PERF-112.
    panic!("nope");
}

pub fn ban_user(user: &mut User) {}

pub fn parse_user(s: &str) -> Result<User, String> {
    todo!()
}

fn build(id: String) -> User {
    User { id: id.clone(), email: id }
}

pub fn load() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn many(a: u8, b: u16, c: u32, d: u64, e: i8, f: i16, g: i32, h: i64) {}

trait Store {
    fn save(&self);
    type Item;
}

fn main() {
    let config = std::env::var("CONFIG").expect("Config file required for startup");
}

fn destructure(user: &User) {
    let User { id, email } = user;
}

impl serde::Serializer for User {
    fn serialize_unit_variant(self, name: &str, variant: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn process(order: &mut User) {
    // step 1: validate
    let valid = true;
    // step 2: transform
    let transformed = valid; // trailing comments do not count
    // step 3: persist
    let _ = transformed;
}

#[cfg(test)]
mod tests {
    use mockall::automock;

    #[test]
    fn unwrap_is_fine_in_tests() {
        let x: Option<u8> = None;
        x.unwrap();
    }
}

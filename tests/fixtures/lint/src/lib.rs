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

pub struct Percentage(pub f64);

impl Percentage {
    pub fn new(value: f64) -> Result<Self, String> {
        Ok(Self(value))
    }
}

pub struct Order {
    status: String,
}

pub enum PaymentError {
    Declined,
    Other(String),
}

pub fn validate_email(s: &str) -> bool {
    s.contains('@')
}

pub fn ambient() -> u64 {
    let now = std::time::SystemTime::now();
    let port = std::env::var("PORT");
    let n: u64 = rand::random();
    let _ = std::fs::read_to_string("x").map_err(|_| PaymentError::Declined);
    n
}

impl Config {
    pub fn from_env() -> Result<Self, PaymentError> {
        let _ = std::env::var("PORT");
        Ok(Config)
    }
}

pub struct Config;

pub fn swallow(order: Order) {
    std::fs::remove_file("x").ok();
    if let Err(_) = std::fs::remove_file("y") {}
    match std::fs::remove_file("z") {
        Ok(()) => {}
        Err(_) => {}
    }
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
fn test_helper(user_id: String, email: String) -> User {
    // arrange
    let x: Option<u8> = None;
    // act
    x.unwrap();
    // assert
    todo!()
}

#[cfg(any(test, feature = "test-utils"))]
pub struct MockService {
    email: String,
}

#[cfg(test)]
mod tests {
    use mockall::automock;

    #[ignore]
    #[test]
    fn skipped_for_no_reason() {}

    #[ignore = "waits on PERF-112"]
    #[test]
    fn skipped_for_a_reason() {}

    #[test]
    fn waits_for_luck() {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    #[test]
    fn unwrap_is_fine_in_tests() {
        let x: Option<u8> = None;
        x.unwrap();
    }
}

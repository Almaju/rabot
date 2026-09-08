struct UserId(String);
struct Email(String);
struct InvoiceId(String);

impl Mailer {
    fn send_invoice(&self, user_id: UserId, email: Email, invoice_id: InvoiceId) {
        self.send_email(&email, &invoice_id);
        self.log_access(&user_id, &invoice_id);
    }
}

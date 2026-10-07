impl Mailer {
    fn send_invoice(&self, user_id: String, email: String, invoice_id: String) {
        self.send_email(&user_id, &invoice_id); // swapped
        self.log_access(&invoice_id, &email); // wrong order
    }
}

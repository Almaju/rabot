enum PaymentError {
    CardDeclined { reason: DeclineReason },
    Other(String),
}

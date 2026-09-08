enum PaymentError {
    CardDeclined {
        reason: DeclineReason,
    },
    CurrencyMismatch {
        expected: Currency,
        got: Currency,
    },
    Provider {
        retry_after: Option<Duration>,
        #[source]
        source: ProviderError,
    },
}

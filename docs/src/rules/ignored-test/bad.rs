#[ignore]
#[test]
fn transfers_between_accounts() {
    let bank = Bank::sandbox();
    bank.transfer(Account::Alice, Account::Bob, Money::cents(500));
    assert_eq!(bank.balance(Account::Bob), Money::cents(500));
}

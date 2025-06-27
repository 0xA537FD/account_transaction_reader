use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;

use crate::data_structures::{Account, Transaction, TransactionType};

pub struct AccountService {
    pub accounts: HashMap<u16, Account>,
    /// Key: transaction id
    pub disputable_transactions: HashMap<u32, Transaction>,
    pub disputed_transaction_ids: HashSet<u32>,
    pub resolved_dispute_ids: HashSet<u32>,
}

impl AccountService {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            disputable_transactions: HashMap::new(),
            disputed_transaction_ids: HashSet::new(),
            resolved_dispute_ids: HashSet::new(),
        }
    }

    /// Record a transaction for an account. This operates on good-will meaning that we don't
    /// return an error if the transaction is invalid. Instead, we just don't perform any operations
    /// on the account.
    pub fn record_transaction(&mut self, transaction: Transaction) {
        if !self.accounts.contains_key(&transaction.client) {
            self.accounts.insert(
                transaction.client,
                Account {
                    client: transaction.client,
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    total: Decimal::ZERO,
                    locked: false,
                },
            );
        }

        let account = self
            .accounts
            .get_mut(&transaction.client)
            .expect("to have an account for the client in our map");
        // the referenced account is locked so we don't perform any operations on it
        if account.locked {
            return;
        }

        match transaction.r#type {
            TransactionType::Deposit => {
                // deposit transactions must specify an amount. if they don't, it looks like an error on the partners side
                if transaction.amount.is_none() {
                    return;
                }

                account.available += transaction.amount.unwrap();
                account.total += transaction.amount.unwrap();
                self.disputable_transactions
                    .insert(transaction.tx, transaction);
            }
            TransactionType::Withdrawal => {
                // withdrawal transactions must specify an amount. if they don't, it looks like an error on the partners side
                if transaction.amount.is_none() {
                    return;
                }

                let amount = transaction.amount.unwrap();
                if amount > account.available {
                    // the account doesn't have enough funds to withdraw so we don't perform any operations on it
                    return;
                }

                account.available -= amount;
                account.total -= amount;
                self.disputable_transactions
                    .insert(transaction.tx, transaction);
            }
            TransactionType::Dispute => {
                let disputed_transaction = self.disputable_transactions.get(&transaction.tx);
                // we don't have a transaction for this dispute so it looks like an error on the partners side
                if disputed_transaction.is_none() {
                    return;
                }

                let disputed_transaction = disputed_transaction.unwrap();
                // the client of the disputed transaction must be the same as the account we're recording the dispute for
                if disputed_transaction.client != transaction.client {
                    return;
                }
                let amount = if let Some(amount) = disputed_transaction.amount {
                    amount
                } else {
                    // disputable transactions must have an amount. if they don't, it looks like an error on the partners side
                    return;
                };
                account.available -= amount;
                account.held += amount;
                self.disputed_transaction_ids.insert(transaction.tx);
            }
            TransactionType::Resolve => {
                // the transaction is not under dispute so it looks like an error on the partners side
                if !self.disputed_transaction_ids.contains(&transaction.tx) {
                    return;
                }

                // the transaction is already resolved so it looks like an error on the partners side
                if self.resolved_dispute_ids.contains(&transaction.tx) {
                    return;
                }

                let resolved_transaction = self.disputable_transactions.get(&transaction.tx);
                // we don't have a transaction for this resolve so it looks like an error on the partners side
                if resolved_transaction.is_none() {
                    return;
                }

                let resolved_transaction = resolved_transaction.unwrap();
                // the client of the resolved transaction must be the same as the account we're recording the resolve for
                if resolved_transaction.client != transaction.client {
                    return;
                }

                let amount = if let Some(amount) = resolved_transaction.amount {
                    amount
                } else {
                    // disputable transactions must have an amount. if they don't, it looks like an error on the partners side
                    return;
                };
                account.held -= amount;
                account.available += amount;
                self.resolved_dispute_ids.insert(transaction.tx);
            }
            TransactionType::Chargeback => {
                // the transaction is not under dispute so it looks like an error on the partners side
                if !self.disputed_transaction_ids.contains(&transaction.tx) {
                    return;
                }
                let disputed_transaction = self.disputable_transactions.get(&transaction.tx);
                if disputed_transaction.is_none() {
                    return;
                }

                let disputed_transaction = disputed_transaction.unwrap();
                // the client of the disputed transaction must be the same as the account we're recording the dispute for
                if disputed_transaction.client != transaction.client {
                    return;
                }

                let amount = if let Some(amount) = disputed_transaction.amount {
                    amount
                } else {
                    // disputable transactions must have an amount. if they don't, it looks like an error on the partners side
                    return;
                };

                // the transaction is resolved but it's now being chargedback so it... to be safe, we undo the resolve and perforom the chargeback
                if self.resolved_dispute_ids.contains(&transaction.tx) {
                    account.held += amount;
                    account.available -= amount;
                    self.resolved_dispute_ids.remove(&transaction.tx);
                }

                account.held -= amount;
                account.total -= amount;
                account.locked = true;
            }
            _ => (),
        }
    }

    pub fn summary(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_withdrawal_with_insufficient_funds() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(Decimal::from(100)),
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_dispute() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::from(50));
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_dispute_with_invalid_tx() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 2,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());
        let account = account.unwrap();
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_dispute_with_resolve() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_dispute_with_resolve_on_invalid_tx() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Resolve,
            client: 1,
            tx: 2,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::from(50));
        assert_eq!(account.locked, false);
    }

    #[test]
    fn test_chargeback() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, true);
    }

    #[test]
    fn test_chargeback_on_disputed_transaction_only() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some(Decimal::from(10)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::from(10));
        assert_eq!(account.total, Decimal::from(10));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, true);
    }

    #[test]
    fn test_revert_of_resolve_on_chargeback() {
        let mut service = AccountService::new();
        service.record_transaction(Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from(50)),
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        });
        service.record_transaction(Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        });

        let account = service.summary().get(&1);
        assert!(account.is_some());

        let account = account.unwrap();
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.locked, true);
    }
}

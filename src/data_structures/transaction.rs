use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(default, with = "super::utils::serde::high_precision_decimal_option")]
    pub amount: Option<Decimal>,
}

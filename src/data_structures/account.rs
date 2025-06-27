use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Account {
    pub client: u16,
    #[serde(with = "super::utils::serde::high_precision_decimal")]
    pub available: Decimal,
    #[serde(with = "super::utils::serde::high_precision_decimal")]
    pub held: Decimal,
    #[serde(with = "super::utils::serde::high_precision_decimal")]
    pub total: Decimal,
    pub locked: bool,
}

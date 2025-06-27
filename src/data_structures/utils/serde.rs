pub mod high_precision_decimal {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        let formatted_decimal = format!("{:.4}", decimal);
        // Remove trailing zeros after decimal point
        let trimmed = formatted_decimal
            .trim_end_matches('0')
            .trim_end_matches('.');
        serializer.serialize_str(trimmed)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        let string = String::deserialize(deserializer)?;
        let decimal = Decimal::from_str(&string).map_err(serde::de::Error::custom)?;
        // Limit to 4 fractional digits
        let limited_decimal = decimal.round_dp(4);

        Ok(limited_decimal)
    }
}

pub mod high_precision_decimal_option {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::data_structures::utils::serde::high_precision_decimal;

    pub fn serialize<S: Serializer>(
        decimal_opt: &Option<Decimal>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if let Some(decimal) = decimal_opt {
            high_precision_decimal::serialize(&decimal, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Decimal>, D::Error> {
        let string_opt = Option::<String>::deserialize(deserializer)?;
        if let Some(string) = string_opt {
            // Use a temporary deserializer to reuse the existing logic
            let decimal = high_precision_decimal::deserialize(
                serde::de::value::StringDeserializer::new(string),
            )?;
            Ok(decimal.into())
        } else {
            Ok(None)
        }
    }
}

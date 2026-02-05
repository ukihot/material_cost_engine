use crate::domain::value_objects::*;
use color_eyre::{Result, eyre::eyre};

/// 運賃マスタエンティティ
#[derive(Debug, Clone)]
pub struct FreightMaster {
    pub freight_code: String,
    pub pattern_name: PatternName,
    pub kg_unit_price: Amount,
    pub valid_from: TransactionDate,
    pub valid_to: Option<TransactionDate>,
}

impl FreightMaster {
    pub fn new(
        freight_code: String,
        pattern_name: PatternName,
        kg_unit_price: Amount,
        valid_from: TransactionDate,
        valid_to: Option<TransactionDate>,
    ) -> Result<Self> {
        // 運賃コードのバリデーション（T0001形式）
        if !freight_code.starts_with('T') || freight_code.len() != 5 {
            return Err(eyre!(
                "運賃コードの形式が不正です: '{}' (T0001～T9999の形式が必要)",
                freight_code
            ));
        }
        let digits = &freight_code[1..];
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(eyre!(
                "運賃コードの形式が不正です: '{}' (T0001～T9999の形式が必要)",
                freight_code
            ));
        }

        Ok(Self {
            freight_code,
            pattern_name,
            kg_unit_price,
            valid_from,
            valid_to,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freight_master_valid() {
        let pattern_name = PatternName::new("標準運賃".to_string()).unwrap();
        let kg_unit_price = Amount::new(150.0).unwrap();
        let valid_from = TransactionDate::new("2024-01-01".to_string()).unwrap();

        let freight_master = FreightMaster::new(
            "T0001".to_string(),
            pattern_name,
            kg_unit_price,
            valid_from,
            None,
        )
        .unwrap();

        assert_eq!(freight_master.freight_code, "T0001");
        assert_eq!(freight_master.kg_unit_price.value(), 150.0);
    }

    #[test]
    fn test_freight_master_with_valid_to() {
        let pattern_name = PatternName::new("期間限定運賃".to_string()).unwrap();
        let kg_unit_price = Amount::new(120.0).unwrap();
        let valid_from = TransactionDate::new("2024-01-01".to_string()).unwrap();
        let valid_to = TransactionDate::new("2024-12-31".to_string()).unwrap();

        let freight_master = FreightMaster::new(
            "T0002".to_string(),
            pattern_name,
            kg_unit_price,
            valid_from,
            Some(valid_to),
        )
        .unwrap();

        assert!(freight_master.valid_to.is_some());
    }

    #[test]
    fn test_freight_master_invalid_code_format() {
        let pattern_name = PatternName::new("標準運賃".to_string()).unwrap();
        let kg_unit_price = Amount::new(150.0).unwrap();
        let valid_from = TransactionDate::new("2024-01-01".to_string()).unwrap();

        let result = FreightMaster::new(
            "A0001".to_string(),
            pattern_name,
            kg_unit_price,
            valid_from,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_freight_master_invalid_code_length() {
        let pattern_name = PatternName::new("標準運賃".to_string()).unwrap();
        let kg_unit_price = Amount::new(150.0).unwrap();
        let valid_from = TransactionDate::new("2024-01-01".to_string()).unwrap();

        let result = FreightMaster::new(
            "T001".to_string(),
            pattern_name,
            kg_unit_price,
            valid_from,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_freight_master_non_numeric_digits() {
        let pattern_name = PatternName::new("標準運賃".to_string()).unwrap();
        let kg_unit_price = Amount::new(150.0).unwrap();
        let valid_from = TransactionDate::new("2024-01-01".to_string()).unwrap();

        let result = FreightMaster::new(
            "T00A1".to_string(),
            pattern_name,
            kg_unit_price,
            valid_from,
            None,
        );

        assert!(result.is_err());
    }
}

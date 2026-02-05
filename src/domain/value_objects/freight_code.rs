use color_eyre::{Result, eyre::eyre};

/// 運賃コード（T0001形式）または直接のKg単価（数値）
#[derive(Debug, Clone)]
pub enum FreightCode {
    Code(String),     // T0001形式のコード
    DirectPrice(f64), // 直接指定されたKg単価
}

impl FreightCode {
    pub fn new(value: String) -> Result<Self> {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(eyre!("運賃コードまたは単価が空です"));
        }

        // 数値かどうかチェック
        if let Ok(price) = trimmed.parse::<f64>() {
            if price < 0.0 {
                return Err(eyre!("運賃単価が負の値です: {}", price));
            }
            return Ok(FreightCode::DirectPrice(price));
        }

        // T0001形式かチェック
        if trimmed.starts_with('T') && trimmed.len() == 5 {
            let digits = &trimmed[1..];
            if digits.chars().all(|c| c.is_ascii_digit()) {
                return Ok(FreightCode::Code(trimmed.to_string()));
            }
        }

        Err(eyre!(
            "運賃コードの形式が不正です: '{}'\n  有効な形式: T0001～T9999 または数値 (例: T0001, 150.5)",
            trimmed
        ))
    }

    pub fn is_code(&self) -> bool {
        matches!(self, FreightCode::Code(_))
    }

    pub fn is_direct_price(&self) -> bool {
        matches!(self, FreightCode::DirectPrice(_))
    }

    pub fn as_code(&self) -> Option<&str> {
        match self {
            FreightCode::Code(code) => Some(code),
            _ => None,
        }
    }

    pub fn as_direct_price(&self) -> Option<f64> {
        match self {
            FreightCode::DirectPrice(price) => Some(*price),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freight_code_valid_code() {
        let code = FreightCode::new("T0001".to_string()).unwrap();
        assert!(code.is_code());
        assert_eq!(code.as_code(), Some("T0001"));
    }

    #[test]
    fn test_freight_code_valid_direct_price() {
        let code = FreightCode::new("150.5".to_string()).unwrap();
        assert!(code.is_direct_price());
        assert_eq!(code.as_direct_price(), Some(150.5));
    }

    #[test]
    fn test_freight_code_zero_price() {
        let code = FreightCode::new("0".to_string()).unwrap();
        assert!(code.is_direct_price());
        assert_eq!(code.as_direct_price(), Some(0.0));
    }

    #[test]
    fn test_freight_code_negative_price() {
        let result = FreightCode::new("-10".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_freight_code_invalid_format() {
        let result = FreightCode::new("ABC123".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_freight_code_invalid_length() {
        let result = FreightCode::new("T001".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_freight_code_invalid_prefix() {
        let result = FreightCode::new("A0001".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_freight_code_empty() {
        let result = FreightCode::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_freight_code_with_whitespace() {
        let code = FreightCode::new("  T0001  ".to_string()).unwrap();
        assert!(code.is_code());
        assert_eq!(code.as_code(), Some("T0001"));
    }
}

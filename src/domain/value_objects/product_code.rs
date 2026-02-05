use color_eyre::{Result, eyre::eyre};

/// 商品コード
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductCode(String);

impl ProductCode {
    pub fn new(code: String) -> Result<Self> {
        if code.trim().is_empty() {
            return Err(eyre!("商品コードが空です"));
        }
        Ok(Self(code.trim().to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_code_valid() {
        let code = ProductCode::new("P001".to_string()).unwrap();
        assert_eq!(code.value(), "P001");
    }

    #[test]
    fn test_product_code_with_whitespace() {
        let code = ProductCode::new("  P001  ".to_string()).unwrap();
        assert_eq!(code.value(), "P001");
    }

    #[test]
    fn test_product_code_empty() {
        let result = ProductCode::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_product_code_whitespace_only() {
        let result = ProductCode::new("   ".to_string());
        assert!(result.is_err());
    }
}

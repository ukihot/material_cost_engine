use color_eyre::{Result, eyre::eyre};

/// パターン名（重複を許さない）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternName(String);

impl PatternName {
    pub fn new(name: String) -> Result<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(eyre!("パターン名が空です"));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_name_valid() {
        let name = PatternName::new("標準運賃".to_string()).unwrap();
        assert_eq!(name.value(), "標準運賃");
    }

    #[test]
    fn test_pattern_name_with_whitespace() {
        let name = PatternName::new("  標準運賃  ".to_string()).unwrap();
        assert_eq!(name.value(), "標準運賃");
    }

    #[test]
    fn test_pattern_name_empty() {
        let result = PatternName::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_name_whitespace_only() {
        let result = PatternName::new("   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_name_equality() {
        let name1 = PatternName::new("標準運賃".to_string()).unwrap();
        let name2 = PatternName::new("標準運賃".to_string()).unwrap();
        assert_eq!(name1, name2);
    }
}

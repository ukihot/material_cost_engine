use color_eyre::{Result, eyre::eyre};

/// 日付（文字列として保持）
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionDate(String);

impl TransactionDate {
    pub fn new(date: String) -> Result<Self> {
        let trimmed = date.trim();

        if trimmed.is_empty() {
            return Err(eyre!("日付が空です"));
        }

        // 日付形式のバリデーション
        // YYYY-MM-DD, YYYY/MM/DD, YYYY.MM.DD などの形式をサポート
        if !Self::is_valid_date_format(trimmed) {
            return Err(eyre!(
                "日付の形式が不正です: '{}'\n  有効な形式: YYYY-MM-DD, YYYY/MM/DD, YYYY.MM.DD (例: 2024-01-15)",
                trimmed
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    fn is_valid_date_format(date_str: &str) -> bool {
        // 区切り文字を検出
        let separator = if date_str.contains('-') {
            '-'
        } else if date_str.contains('/') {
            '/'
        } else if date_str.contains('.') {
            '.'
        } else {
            return false;
        };

        let parts: Vec<&str> = date_str.split(separator).collect();
        if parts.len() != 3 {
            return false;
        }

        // 年月日を解析
        let year: i32 = match parts[0].parse() {
            Ok(y) => y,
            Err(_) => return false,
        };
        let month: u32 = match parts[1].parse() {
            Ok(m) => m,
            Err(_) => return false,
        };
        let day: u32 = match parts[2].parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        // 年の範囲チェック（1900-2100）
        if !(1900..=2100).contains(&year) {
            return false;
        }

        // 月の範囲チェック
        if !(1..=12).contains(&month) {
            return false;
        }

        // 日の範囲チェック
        if day < 1 {
            return false;
        }

        // 月ごとの最大日数チェック
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                // うるう年判定
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };

        day <= max_day
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_date_valid_hyphen() {
        let date = TransactionDate::new("2024-01-15".to_string()).unwrap();
        assert_eq!(date.value(), "2024-01-15");
    }

    #[test]
    fn test_transaction_date_valid_slash() {
        let date = TransactionDate::new("2024/01/15".to_string()).unwrap();
        assert_eq!(date.value(), "2024/01/15");
    }

    #[test]
    fn test_transaction_date_valid_dot() {
        let date = TransactionDate::new("2024.01.15".to_string()).unwrap();
        assert_eq!(date.value(), "2024.01.15");
    }

    #[test]
    fn test_transaction_date_leap_year() {
        let date = TransactionDate::new("2024-02-29".to_string()).unwrap();
        assert_eq!(date.value(), "2024-02-29");
    }

    #[test]
    fn test_transaction_date_non_leap_year() {
        let result = TransactionDate::new("2023-02-29".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_date_empty() {
        let result = TransactionDate::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_date_invalid_format() {
        let result = TransactionDate::new("20240115".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_date_invalid_month() {
        let result = TransactionDate::new("2024-13-01".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_date_invalid_day() {
        let result = TransactionDate::new("2024-01-32".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_date_ordering() {
        let date1 = TransactionDate::new("2024-01-15".to_string()).unwrap();
        let date2 = TransactionDate::new("2024-02-20".to_string()).unwrap();
        assert!(date1 < date2);
    }
}

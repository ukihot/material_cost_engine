use color_eyre::{Result, eyre::eyre};

/// 歩留率
#[derive(Debug, Clone, Copy)]
pub struct YieldRate(f64);

impl YieldRate {
    pub fn new(value: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(eyre!(
                "歩留率は0.0から1.0の範囲である必要があります: {}",
                value
            ));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_rate_valid() {
        let rate = YieldRate::new(0.95).unwrap();
        assert_eq!(rate.value(), 0.95);
    }

    #[test]
    fn test_yield_rate_zero() {
        let rate = YieldRate::new(0.0).unwrap();
        assert_eq!(rate.value(), 0.0);
    }

    #[test]
    fn test_yield_rate_one() {
        let rate = YieldRate::new(1.0).unwrap();
        assert_eq!(rate.value(), 1.0);
    }

    #[test]
    fn test_yield_rate_negative() {
        let result = YieldRate::new(-0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_yield_rate_over_one() {
        let result = YieldRate::new(1.5);
        assert!(result.is_err());
    }
}

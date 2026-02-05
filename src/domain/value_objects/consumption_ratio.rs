use color_eyre::{Result, eyre::eyre};

/// 消費比率
#[derive(Debug, Clone, Copy)]
pub struct ConsumptionRatio(f64);

impl ConsumptionRatio {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("消費比率が負の値です: {}", value));
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
    fn test_consumption_ratio_valid() {
        let ratio = ConsumptionRatio::new(0.5).unwrap();
        assert_eq!(ratio.value(), 0.5);
    }

    #[test]
    fn test_consumption_ratio_zero() {
        let ratio = ConsumptionRatio::new(0.0).unwrap();
        assert_eq!(ratio.value(), 0.0);
    }

    #[test]
    fn test_consumption_ratio_large() {
        let ratio = ConsumptionRatio::new(10.5).unwrap();
        assert_eq!(ratio.value(), 10.5);
    }

    #[test]
    fn test_consumption_ratio_negative() {
        let result = ConsumptionRatio::new(-0.5);
        assert!(result.is_err());
    }
}

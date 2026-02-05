use color_eyre::{Result, eyre::eyre};

/// 金額（小計）
#[derive(Debug, Clone, Copy)]
pub struct Amount(f64);

impl Amount {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("金額が負の値です: {}", value));
        }
        Ok(Self(value))
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn add(&self, other: &Amount) -> Amount {
        Amount(self.0 + other.0)
    }

    pub fn multiply(&self, ratio: f64) -> Amount {
        Amount(self.0 * ratio)
    }

    pub fn divide_by(&self, divisor: f64) -> Amount {
        Amount(self.0 / divisor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_valid() {
        let amount = Amount::new(1000.0).unwrap();
        assert_eq!(amount.value(), 1000.0);
    }

    #[test]
    fn test_amount_zero() {
        let amount = Amount::zero();
        assert_eq!(amount.value(), 0.0);
    }

    #[test]
    fn test_amount_negative() {
        let result = Amount::new(-100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_amount_add() {
        let a1 = Amount::new(100.0).unwrap();
        let a2 = Amount::new(200.0).unwrap();
        let result = a1.add(&a2);
        assert_eq!(result.value(), 300.0);
    }

    #[test]
    fn test_amount_multiply() {
        let amount = Amount::new(100.0).unwrap();
        let result = amount.multiply(2.5);
        assert_eq!(result.value(), 250.0);
    }

    #[test]
    fn test_amount_divide() {
        let amount = Amount::new(100.0).unwrap();
        let result = amount.divide_by(4.0);
        assert_eq!(result.value(), 25.0);
    }
}

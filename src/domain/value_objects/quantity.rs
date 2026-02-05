use color_eyre::{Result, eyre::eyre};

/// 数量
#[derive(Debug, Clone, Copy)]
pub struct Quantity(f64);

impl Quantity {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(eyre!("数量が負の値です: {}", value));
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
    fn test_quantity_valid() {
        let qty = Quantity::new(100.0).unwrap();
        assert_eq!(qty.value(), 100.0);
    }

    #[test]
    fn test_quantity_zero() {
        let qty = Quantity::new(0.0).unwrap();
        assert_eq!(qty.value(), 0.0);
    }

    #[test]
    fn test_quantity_negative() {
        let result = Quantity::new(-10.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantity_decimal() {
        let qty = Quantity::new(123.456).unwrap();
        assert_eq!(qty.value(), 123.456);
    }
}

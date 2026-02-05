use color_eyre::Result;

/// 在庫残高（負の値も許容）
#[derive(Debug, Clone, Copy)]
pub struct InventoryBalance(f64);

impl InventoryBalance {
    pub fn new(value: f64) -> Result<Self> {
        // 在庫残高は負の値も許容（マイナス在庫）
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
    fn test_inventory_balance_positive() {
        let balance = InventoryBalance::new(100.0).unwrap();
        assert_eq!(balance.value(), 100.0);
    }

    #[test]
    fn test_inventory_balance_zero() {
        let balance = InventoryBalance::new(0.0).unwrap();
        assert_eq!(balance.value(), 0.0);
    }

    #[test]
    fn test_inventory_balance_negative() {
        let balance = InventoryBalance::new(-50.0).unwrap();
        assert_eq!(balance.value(), -50.0);
    }
}

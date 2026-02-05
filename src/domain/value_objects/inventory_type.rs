/// 在庫区分
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryType {
    Production,
    Purchase,
    Sales,
}

impl InventoryType {
    pub fn as_str(&self) -> &str {
        match self {
            InventoryType::Production => "生産",
            InventoryType::Purchase => "仕入",
            InventoryType::Sales => "売上",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_type_production() {
        let inv_type = InventoryType::Production;
        assert_eq!(inv_type.as_str(), "生産");
    }

    #[test]
    fn test_inventory_type_purchase() {
        let inv_type = InventoryType::Purchase;
        assert_eq!(inv_type.as_str(), "仕入");
    }

    #[test]
    fn test_inventory_type_sales() {
        let inv_type = InventoryType::Sales;
        assert_eq!(inv_type.as_str(), "売上");
    }

    #[test]
    fn test_inventory_type_equality() {
        let inv1 = InventoryType::Production;
        let inv2 = InventoryType::Production;
        assert_eq!(inv1, inv2);
    }
}

use crate::domain::value_objects::*;

/// 入出庫トランザクション
#[derive(Debug, Clone)]
pub struct InventoryTransaction {
    pub date: TransactionDate,
    pub inventory_type: InventoryType,
    pub product_code: ProductCode,
    pub product_name: String,
    pub quantity: Quantity,
}

impl InventoryTransaction {
    pub fn new(
        date: TransactionDate,
        inventory_type: InventoryType,
        product_code: ProductCode,
        product_name: String,
        quantity: Quantity,
    ) -> Self {
        Self {
            date,
            inventory_type,
            product_code,
            product_name,
            quantity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_transaction_production() {
        let date = TransactionDate::new("2024-01-15".to_string()).unwrap();
        let product_code = ProductCode::new("P001".to_string()).unwrap();
        let quantity = Quantity::new(100.0).unwrap();

        let transaction = InventoryTransaction::new(
            date,
            InventoryType::Production,
            product_code.clone(),
            "製品A".to_string(),
            quantity,
        );

        assert_eq!(transaction.date.value(), "2024-01-15");
        assert_eq!(transaction.inventory_type, InventoryType::Production);
        assert_eq!(transaction.product_code.value(), "P001");
        assert_eq!(transaction.product_name, "製品A");
        assert_eq!(transaction.quantity.value(), 100.0);
    }

    #[test]
    fn test_inventory_transaction_purchase() {
        let date = TransactionDate::new("2024-02-20".to_string()).unwrap();
        let product_code = ProductCode::new("M001".to_string()).unwrap();
        let quantity = Quantity::new(50.0).unwrap();

        let transaction = InventoryTransaction::new(
            date,
            InventoryType::Purchase,
            product_code,
            "材料B".to_string(),
            quantity,
        );

        assert_eq!(transaction.inventory_type, InventoryType::Purchase);
    }

    #[test]
    fn test_inventory_transaction_sales() {
        let date = TransactionDate::new("2024-03-10".to_string()).unwrap();
        let product_code = ProductCode::new("P002".to_string()).unwrap();
        let quantity = Quantity::new(30.0).unwrap();

        let transaction = InventoryTransaction::new(
            date,
            InventoryType::Sales,
            product_code,
            "製品C".to_string(),
            quantity,
        );

        assert_eq!(transaction.inventory_type, InventoryType::Sales);
    }
}

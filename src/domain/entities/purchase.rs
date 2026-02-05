use crate::domain::value_objects::*;

/// 仕入エンティティ
#[derive(Debug, Clone)]
pub struct Purchase {
    pub product_name: String,
    pub unit_price: Amount,
    pub quantity: Quantity,
    pub freight_code: FreightCode,
}

impl Purchase {
    pub fn new(
        product_name: String,
        unit_price: Amount,
        quantity: Quantity,
        freight_code: FreightCode,
    ) -> Self {
        Self {
            product_name,
            unit_price,
            quantity,
            freight_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_with_code() {
        let unit_price = Amount::new(100.0).unwrap();
        let quantity = Quantity::new(50.0).unwrap();
        let freight_code = FreightCode::new("T0001".to_string()).unwrap();

        let purchase = Purchase::new("原材料A".to_string(), unit_price, quantity, freight_code);

        assert_eq!(purchase.product_name, "原材料A");
        assert_eq!(purchase.unit_price.value(), 100.0);
        assert_eq!(purchase.quantity.value(), 50.0);
        assert!(purchase.freight_code.is_code());
    }

    #[test]
    fn test_purchase_with_direct_price() {
        let unit_price = Amount::new(100.0).unwrap();
        let quantity = Quantity::new(50.0).unwrap();
        let freight_code = FreightCode::new("150.5".to_string()).unwrap();

        let purchase = Purchase::new("原材料B".to_string(), unit_price, quantity, freight_code);

        assert_eq!(purchase.product_name, "原材料B");
        assert!(purchase.freight_code.is_direct_price());
        assert_eq!(purchase.freight_code.as_direct_price(), Some(150.5));
    }
}

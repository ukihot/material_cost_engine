use crate::domain::value_objects::*;

/// 生産エンティティ
#[derive(Debug, Clone)]
pub struct Production {
    pub product_code: ProductCode,
    pub quantity: Quantity,
    pub yield_rate: YieldRate,
    pub coagulant_cost: Amount,
    pub clay_treatment_cost: Amount,
}

impl Production {
    pub fn new(
        product_code: ProductCode,
        quantity: Quantity,
        yield_rate: YieldRate,
        coagulant_cost: Amount,
        clay_treatment_cost: Amount,
    ) -> Self {
        Self {
            product_code,
            quantity,
            yield_rate,
            coagulant_cost,
            clay_treatment_cost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_creation() {
        let product_code = ProductCode::new("P001".to_string()).unwrap();
        let quantity = Quantity::new(1000.0).unwrap();
        let yield_rate = YieldRate::new(0.95).unwrap();
        let coagulant_cost = Amount::new(100.0).unwrap();
        let clay_treatment_cost = Amount::new(50.0).unwrap();

        let production = Production::new(
            product_code.clone(),
            quantity,
            yield_rate,
            coagulant_cost,
            clay_treatment_cost,
        );

        assert_eq!(production.product_code.value(), "P001");
        assert_eq!(production.quantity.value(), 1000.0);
        assert_eq!(production.yield_rate.value(), 0.95);
        assert_eq!(production.coagulant_cost.value(), 100.0);
        assert_eq!(production.clay_treatment_cost.value(), 50.0);
    }
}

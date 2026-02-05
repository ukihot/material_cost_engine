use super::value_objects::*;

/// 生産エンティティ
#[derive(Debug, Clone)]
pub struct Production {
    pub product_code: ProductCode,
    pub production_number: String,
    pub quantity: Quantity,
    pub yield_rate: YieldRate,
    pub coagulant_cost: Amount,
    pub clay_treatment_cost: Amount,
}

impl Production {
    pub fn new(
        product_code: ProductCode,
        production_number: String,
        quantity: Quantity,
        yield_rate: YieldRate,
        coagulant_cost: Amount,
        clay_treatment_cost: Amount,
    ) -> Self {
        Self {
            product_code,
            production_number,
            quantity,
            yield_rate,
            coagulant_cost,
            clay_treatment_cost,
        }
    }
}

/// 配合マスタエンティティ
#[derive(Debug, Clone)]
pub struct FormulaEntry {
    pub product_code: ProductCode,
    pub material_code: ProductCode,
    pub consumption_ratio: ConsumptionRatio,
}

impl FormulaEntry {
    pub fn new(
        product_code: ProductCode,
        material_code: ProductCode,
        consumption_ratio: ConsumptionRatio,
    ) -> Self {
        Self {
            product_code,
            material_code,
            consumption_ratio,
        }
    }
}

/// 仕入エンティティ
#[derive(Debug, Clone)]
pub struct Purchase {
    pub product_code: ProductCode,
    pub product_name: String,
    pub unit_price: Amount,
}

impl Purchase {
    pub fn new(product_code: ProductCode, product_name: String, unit_price: Amount) -> Self {
        Self {
            product_code,
            product_name,
            unit_price,
        }
    }
}

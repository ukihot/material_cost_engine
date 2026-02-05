use crate::domain::value_objects::*;

/// 配合マスタエンティティ
#[derive(Debug, Clone)]
pub struct FormulaEntry {
    pub material_code: ProductCode,
    pub consumption_ratio: ConsumptionRatio,
}

impl FormulaEntry {
    pub fn new(material_code: ProductCode, consumption_ratio: ConsumptionRatio) -> Self {
        Self {
            material_code,
            consumption_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_entry_creation() {
        let material_code = ProductCode::new("M001".to_string()).unwrap();
        let consumption_ratio = ConsumptionRatio::new(0.5).unwrap();

        let entry = FormulaEntry::new(material_code.clone(), consumption_ratio);

        assert_eq!(entry.material_code.value(), "M001");
        assert_eq!(entry.consumption_ratio.value(), 0.5);
    }
}

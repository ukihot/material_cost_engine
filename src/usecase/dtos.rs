/// 材料消費結果DTO
#[derive(Debug, Clone)]
pub struct MaterialConsumptionDto {
    pub material_code: String,
    pub material_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total_cost: f64,
    pub freight_cost: f64,
    pub purchase_quantity: f64,
    pub freight_code_str: String,
    pub freight_kg_price: f64,
}

/// 材料費計算結果DTO
#[derive(Debug, Clone)]
pub struct MaterialCostResultDto {
    pub row_number: usize,
    pub raw_material_cost: f64,
    pub yield_cost: f64,
    pub coagulant_cost: f64,
    pub clay_treatment_cost: f64,
    pub freight_cost: f64,
    pub total_material_cost: f64,
}

/// 入出庫履歴レコードDTO
#[derive(Debug, Clone)]
pub struct InventoryHistoryRecordDto {
    pub date: String,
    pub inventory_type: String,
    pub product_code: String,
    pub product_name: String,
    pub base_quantity: f64,
    pub change_quantity: f64,
    pub balance: f64,
}

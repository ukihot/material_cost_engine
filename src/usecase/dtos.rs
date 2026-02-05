use crate::domain::sheet_schema::ProductionSheetSchema;

/// 生産データDTO
#[derive(Debug, Clone)]
pub struct ProductionDto {
    pub row_number: usize,
    pub product_code: String,
    pub production_number: String,
    pub quantity: f64,
    pub yield_rate: f64,
    pub coagulant_cost: f64,
    pub clay_treatment_cost: f64,
}

/// 材料消費結果DTO
#[derive(Debug, Clone)]
pub struct MaterialConsumptionDto {
    pub material_code: String,
    pub material_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total_cost: f64,
}

/// 材料費計算結果DTO
#[derive(Debug, Clone)]
pub struct MaterialCostResultDto {
    pub row_number: usize,
    pub product_code: String,
    pub material_consumptions: Vec<MaterialConsumptionDto>,
    pub raw_material_cost: f64,
    pub unit_cost: f64,
    pub yield_cost: f64,
    pub coagulant_cost: f64,
    pub clay_treatment_cost: f64,
    pub total_material_cost: f64,
}

/// プレゼンター初期化用DTO
#[derive(Debug, Clone)]
pub struct PresenterConfigDto {
    pub input_file_path: String,
    pub output_file_path: String,
    pub production_sheet_schema: ProductionSheetSchema,
}

use super::dtos::*;
use color_eyre::Result;

/// インプットポート（ユースケースの抽象）
pub trait CalculateMaterialCostInputPort {
    fn execute(&mut self, productions: Vec<ProductionDto>) -> Result<()>;
}

/// アウトプットポート（プレゼンターの抽象）
pub trait CalculateMaterialCostOutputPort {
    fn initialize(&mut self, config: PresenterConfigDto) -> Result<()>;
    fn present_no_data(&mut self);
    fn present_validation_error(&mut self, row_number: usize, message: &str);
    fn present_calculation_start(&mut self, total_rows: usize);
    fn present_processing_row(&mut self, row_number: usize, product_code: &str);
    fn present_material_consumptions(&mut self, consumptions: &[MaterialConsumptionDto]);
    fn present_calculation_result(&mut self, result: &MaterialCostResultDto);
    fn present_completion(&mut self);
    fn present_error(&mut self, message: &str);
    fn finalize(&mut self) -> Result<()>;
}

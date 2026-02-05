use super::dtos::*;
use color_eyre::Result;

/// インプットポート（ユースケースの抽象）
pub trait CalculateMaterialCostInputPort {
    fn execute(&mut self) -> Result<()>;
}

/// アウトプットポート（プレゼンターの抽象）
pub trait CalculateMaterialCostOutputPort {
    fn present_no_data(&mut self);
    fn present_calculation_start(&mut self, total_rows: usize);
    fn present_processing_row(&mut self, row_number: usize, product_code: &str);
    fn present_material_consumptions(&mut self, consumptions: &[MaterialConsumptionDto]);
    fn present_calculation_result(&mut self, result: &MaterialCostResultDto);
    fn present_completion(&mut self);
    fn present_error(&mut self, message: &str);
}

/// 入出庫履歴作成インプットポート
pub trait CreateInventoryHistoryInputPort {
    fn execute(&mut self) -> Result<()>;
}

/// 入出庫履歴作成アウトプットポート
pub trait CreateInventoryHistoryOutputPort {
    fn present_history_start(&mut self);
    fn present_history_record(&mut self, record: &InventoryHistoryRecordDto);
    fn present_history_completion(&mut self, total_records: usize);
    fn present_history_error(&mut self, message: &str);
    fn finalize(&mut self) -> Result<()>;
}

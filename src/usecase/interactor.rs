use super::dtos::*;
use super::ports::*;
use crate::domain::repositories::*;
use crate::domain::services::*;
use color_eyre::Result;

/// 材料費計算インタラクタ
pub struct CalculateMaterialCostInteractor<'a, F, P, FR, R, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    FR: FreightMasterRepository,
    R: ProductionRepository,
    O: CalculateMaterialCostOutputPort,
{
    formula_repo: &'a F,
    purchase_repo: &'a P,
    freight_repo: &'a FR,
    production_repo: &'a R,
    output_port: &'a mut O,
}

impl<'a, F, P, FR, R, O> CalculateMaterialCostInteractor<'a, F, P, FR, R, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    FR: FreightMasterRepository,
    R: ProductionRepository,
    O: CalculateMaterialCostOutputPort,
{
    pub fn new(
        formula_repo: &'a F,
        purchase_repo: &'a P,
        freight_repo: &'a FR,
        production_repo: &'a R,
        output_port: &'a mut O,
    ) -> Self {
        Self {
            formula_repo,
            purchase_repo,
            freight_repo,
            production_repo,
            output_port,
        }
    }
}

impl<'a, F, P, FR, R, O> CalculateMaterialCostInputPort
    for CalculateMaterialCostInteractor<'a, F, P, FR, R, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    FR: FreightMasterRepository,
    R: ProductionRepository,
    O: CalculateMaterialCostOutputPort,
{
    fn execute(&mut self) -> Result<()> {
        // リポジトリから生産データを取得
        let productions = match self.production_repo.find_all() {
            Ok(p) => p,
            Err(e) => {
                self.output_port.present_error(&format!("{:?}", e));
                return Err(e);
            }
        };

        // データがない場合
        if productions.is_empty() {
            self.output_port.present_no_data();
            return Ok(());
        }

        self.output_port
            .present_calculation_start(productions.len());

        for (idx, production) in productions.iter().enumerate() {
            self.output_port.present_processing_row(
                idx + 2, // ヘッダー行を考慮して+2
                production.product_code.value(),
            );

            // 材料消費を計算
            let result = match MaterialCostCalculationService::calculate_material_consumption(
                production,
                self.formula_repo,
                self.purchase_repo,
                self.freight_repo,
            ) {
                Ok(r) => r,
                Err(e) => {
                    self.output_port.present_error(&format!("{:?}", e));
                    return Err(e);
                }
            };

            // DTOに変換
            let consumption_dtos: Vec<MaterialConsumptionDto> = result
                .consumptions
                .iter()
                .map(|c| MaterialConsumptionDto {
                    material_code: c.material_code.value().to_string(),
                    material_name: c.material_name.clone(),
                    quantity: c.quantity.value(),
                    unit_price: c.unit_price.value(),
                    total_cost: c.total_cost.value(),
                    freight_cost: c.freight_cost.value(),
                    purchase_quantity: c.purchase_quantity.value(),
                    freight_code_str: c.freight_code_str.clone(),
                    freight_kg_price: c.freight_kg_price,
                })
                .collect();

            self.output_port
                .present_material_consumptions(&consumption_dtos);

            // 各種金額を計算
            let raw_material_cost =
                MaterialCostCalculationService::calculate_raw_material_cost(&result.consumptions);

            // 消費砂量の合計を計算（kg）
            let total_consumption_kg: f64 =
                result.consumptions.iter().map(|c| c.quantity.value()).sum();

            let unit_cost = MaterialCostCalculationService::calculate_unit_cost(
                &raw_material_cost,
                total_consumption_kg,
            );
            let yield_cost = MaterialCostCalculationService::calculate_yield_cost(
                &raw_material_cost,
                &production.yield_rate,
            );
            let total_material_cost = MaterialCostCalculationService::calculate_total_material_cost(
                &yield_cost,
                &production.coagulant_cost,
                &production.clay_treatment_cost,
                &result.total_freight_cost,
            );

            // 結果をDTOに変換
            let result_dto = MaterialCostResultDto {
                row_number: idx + 2, // ヘッダー行を考慮して+2
                raw_material_cost: raw_material_cost.value(),
                unit_cost: unit_cost.value(),
                yield_cost: yield_cost.value(),
                coagulant_cost: production.coagulant_cost.value(),
                clay_treatment_cost: production.clay_treatment_cost.value(),
                freight_cost: result.total_freight_cost.value(),
                total_material_cost: total_material_cost.value(),
            };

            self.output_port.present_calculation_result(&result_dto);
        }

        self.output_port.present_completion();
        Ok(())
    }
}

/// 入出庫履歴作成インタラクタ
pub struct CreateInventoryHistoryInteractor<'a, R, O>
where
    R: InventoryTransactionRepository,
    O: CreateInventoryHistoryOutputPort,
{
    transaction_repo: &'a R,
    output_port: &'a mut O,
}

impl<'a, R, O> CreateInventoryHistoryInteractor<'a, R, O>
where
    R: InventoryTransactionRepository,
    O: CreateInventoryHistoryOutputPort,
{
    pub fn new(transaction_repo: &'a R, output_port: &'a mut O) -> Self {
        Self {
            transaction_repo,
            output_port,
        }
    }
}

impl<'a, R, O> CreateInventoryHistoryInputPort for CreateInventoryHistoryInteractor<'a, R, O>
where
    R: InventoryTransactionRepository,
    O: CreateInventoryHistoryOutputPort,
{
    fn execute(&mut self) -> Result<()> {
        self.output_port.present_history_start();

        // 全トランザクションを取得
        let transactions = match self.transaction_repo.find_all_transactions() {
            Ok(t) => t,
            Err(e) => {
                self.output_port.present_history_error(&format!("{:?}", e));
                return Err(e);
            }
        };

        // 入出庫履歴を作成
        let records = match InventoryHistoryService::create_history(transactions) {
            Ok(r) => r,
            Err(e) => {
                self.output_port.present_history_error(&format!("{:?}", e));
                return Err(e);
            }
        };

        // 各レコードを出力
        for record in &records {
            let dto = InventoryHistoryRecordDto {
                date: record.date.value().to_string(),
                inventory_type: record.inventory_type.as_str().to_string(),
                product_code: record.product_code.value().to_string(),
                product_name: record.product_name.clone(),
                base_quantity: record.base_quantity.value(),
                change_quantity: record.change_quantity.value(),
                balance: record.balance.value(),
            };
            self.output_port.present_history_record(&dto);
        }

        self.output_port.present_history_completion(records.len());

        // インタラクタがアウトプットポートを終了
        self.output_port.finalize()?;

        Ok(())
    }
}

use super::dtos::*;
use super::ports::*;
use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::domain::services::*;
use crate::domain::value_objects::*;
use color_eyre::Result;

/// 材料費計算インタラクタ
pub struct CalculateMaterialCostInteractor<'a, F, P, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    O: CalculateMaterialCostOutputPort,
{
    formula_repo: &'a F,
    purchase_repo: &'a P,
    output_port: &'a mut O,
}

impl<'a, F, P, O> CalculateMaterialCostInteractor<'a, F, P, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    O: CalculateMaterialCostOutputPort,
{
    pub fn new(formula_repo: &'a F, purchase_repo: &'a P, output_port: &'a mut O) -> Self {
        Self {
            formula_repo,
            purchase_repo,
            output_port,
        }
    }

    fn validate_production_dto(&self, dto: &ProductionDto) -> Result<Production> {
        Ok(Production::new(
            ProductCode::new(dto.product_code.clone())?,
            dto.production_number.clone(),
            Quantity::new(dto.quantity)?,
            YieldRate::new(dto.yield_rate)?,
            Amount::new(dto.coagulant_cost)?,
            Amount::new(dto.clay_treatment_cost)?,
        ))
    }
}

impl<'a, F, P, O> CalculateMaterialCostInputPort for CalculateMaterialCostInteractor<'a, F, P, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    O: CalculateMaterialCostOutputPort,
{
    fn execute(&mut self, productions: Vec<ProductionDto>) -> Result<()> {
        // データがない場合
        if productions.is_empty() {
            self.output_port.present_no_data();
            return Ok(());
        }

        self.output_port
            .present_calculation_start(productions.len());

        for production_dto in productions {
            self.output_port
                .present_processing_row(production_dto.row_number, &production_dto.product_code);

            // バリデーション
            let production = self.validate_production_dto(&production_dto)?;

            // 材料消費を計算
            let consumptions = MaterialCostCalculationService::calculate_material_consumption(
                &production,
                self.formula_repo,
                self.purchase_repo,
            )?;

            // DTOに変換
            let consumption_dtos: Vec<MaterialConsumptionDto> = consumptions
                .iter()
                .map(|c| MaterialConsumptionDto {
                    material_code: c.material_code.value().to_string(),
                    material_name: c.material_name.clone(),
                    quantity: c.quantity.value(),
                    unit_price: c.unit_price.value(),
                    total_cost: c.total_cost.value(),
                })
                .collect();

            self.output_port
                .present_material_consumptions(&consumption_dtos);

            // 各種金額を計算
            let raw_material_cost =
                MaterialCostCalculationService::calculate_raw_material_cost(&consumptions);
            let unit_cost = MaterialCostCalculationService::calculate_unit_cost(&raw_material_cost);
            let yield_cost = MaterialCostCalculationService::calculate_yield_cost(
                &raw_material_cost,
                &production.yield_rate,
            );
            let total_material_cost = MaterialCostCalculationService::calculate_total_material_cost(
                &yield_cost,
                &production.coagulant_cost,
                &production.clay_treatment_cost,
            );

            // 結果をDTOに変換
            let result = MaterialCostResultDto {
                row_number: production_dto.row_number,
                product_code: production.product_code.value().to_string(),
                material_consumptions: consumption_dtos,
                raw_material_cost: raw_material_cost.value(),
                unit_cost: unit_cost.value(),
                yield_cost: yield_cost.value(),
                coagulant_cost: production.coagulant_cost.value(),
                clay_treatment_cost: production.clay_treatment_cost.value(),
                total_material_cost: total_material_cost.value(),
            };

            self.output_port.present_calculation_result(&result);
        }

        self.output_port.present_completion();
        Ok(())
    }
}

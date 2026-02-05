use super::entities::*;
use super::repositories::*;
use super::value_objects::*;
use color_eyre::Result;

/// 材料消費計算結果
#[derive(Debug, Clone)]
pub struct MaterialConsumption {
    pub material_code: ProductCode,
    pub material_name: String,
    pub quantity: Quantity,
    pub unit_price: Amount,
    pub total_cost: Amount,
}

/// 材料費計算ドメインサービス
pub struct MaterialCostCalculationService;

impl MaterialCostCalculationService {
    /// 材料消費を計算
    pub fn calculate_material_consumption<F, P>(
        production: &Production,
        formula_repo: &F,
        purchase_repo: &P,
    ) -> Result<Vec<MaterialConsumption>>
    where
        F: FormulaRepository,
        P: PurchaseRepository,
    {
        // 配合マスタから材料を取得
        let formulas = formula_repo.find_by_product_code(&production.product_code)?;

        let mut consumptions = Vec::new();

        for formula in formulas {
            // 消費数量を計算
            let consumption_qty =
                Quantity::new(production.quantity.value() * formula.consumption_ratio.value())?;

            // 仕入データから単価を取得
            let purchase = purchase_repo.find_latest_price(&formula.material_code)?;

            // 材料費を計算
            let total_cost = purchase.unit_price.multiply(consumption_qty.value());

            consumptions.push(MaterialConsumption {
                material_code: formula.material_code.clone(),
                material_name: purchase.product_name.clone(),
                quantity: consumption_qty,
                unit_price: purchase.unit_price,
                total_cost,
            });
        }

        Ok(consumptions)
    }

    /// 原砂金額を計算
    pub fn calculate_raw_material_cost(consumptions: &[MaterialConsumption]) -> Amount {
        consumptions
            .iter()
            .fold(Amount::zero(), |acc, c| acc.add(&c.total_cost))
    }

    /// 原単位を計算（円/t）
    pub fn calculate_unit_cost(raw_material_cost: &Amount) -> Amount {
        raw_material_cost.divide_by(1000.0)
    }

    /// 原砂歩留金額を計算
    pub fn calculate_yield_cost(raw_material_cost: &Amount, yield_rate: &YieldRate) -> Amount {
        raw_material_cost.multiply(yield_rate.value())
    }

    /// 材料費合計を計算
    pub fn calculate_total_material_cost(
        yield_cost: &Amount,
        coagulant_cost: &Amount,
        clay_treatment_cost: &Amount,
    ) -> Amount {
        yield_cost.add(coagulant_cost).add(clay_treatment_cost)
    }
}

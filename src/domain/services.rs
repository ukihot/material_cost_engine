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
    /// 運賃を計算
    pub fn calculate_freight_cost<F>(
        freight_code: &FreightCode,
        quantity: &Quantity,
        freight_repo: &F,
    ) -> Result<Amount>
    where
        F: FreightMasterRepository,
    {
        match freight_code {
            FreightCode::DirectPrice(price) => {
                // 直接指定されたKg単価
                Amount::new(price * quantity.value())
            }
            FreightCode::Code(code) => {
                // 運賃マスタから取得
                let freight_master = freight_repo.find_by_code(code)?;
                Amount::new(freight_master.kg_unit_price.value() * quantity.value())
            }
        }
    }

    /// 材料消費を計算
    pub fn calculate_material_consumption<F, P, FR>(
        production: &Production,
        formula_repo: &F,
        purchase_repo: &P,
        freight_repo: &FR,
    ) -> Result<Vec<MaterialConsumption>>
    where
        F: FormulaRepository,
        P: PurchaseRepository,
        FR: FreightMasterRepository,
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

            // 運賃を計算
            let freight_cost = Self::calculate_freight_cost(
                &purchase.freight_code,
                &purchase.quantity,
                freight_repo,
            )?;

            // 材料費を計算（単価 + 運賃/数量）
            let freight_per_unit = if purchase.quantity.value() > 0.0 {
                freight_cost.divide_by(purchase.quantity.value())
            } else {
                Amount::zero()
            };
            let unit_price_with_freight = purchase.unit_price.add(&freight_per_unit);
            let total_cost = unit_price_with_freight.multiply(consumption_qty.value());

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

    /// 材料費合計を計算（運賃を含む）
    pub fn calculate_total_material_cost(
        yield_cost: &Amount,
        coagulant_cost: &Amount,
        clay_treatment_cost: &Amount,
        freight_cost: &Amount,
    ) -> Amount {
        yield_cost
            .add(coagulant_cost)
            .add(clay_treatment_cost)
            .add(freight_cost)
    }
}

/// 入出庫履歴レコード
#[derive(Debug, Clone)]
pub struct InventoryHistoryRecord {
    pub date: TransactionDate,
    pub inventory_type: InventoryType,
    pub product_code: ProductCode,
    pub product_name: String,
    pub base_quantity: InventoryBalance,
    pub change_quantity: Quantity,
    pub balance: InventoryBalance,
}

/// 入出庫履歴計算ドメインサービス
pub struct InventoryHistoryService;

impl InventoryHistoryService {
    /// トランザクションから入出庫履歴を作成
    pub fn create_history(
        transactions: Vec<InventoryTransaction>,
    ) -> Result<Vec<InventoryHistoryRecord>> {
        use std::collections::HashMap;

        // 日付と商品コードでソート
        let mut sorted_transactions = transactions;
        sorted_transactions.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.product_code.value().cmp(b.product_code.value()))
        });

        // 商品ごとの残高を管理
        let mut balances: HashMap<String, f64> = HashMap::new();
        let mut records = Vec::new();

        for transaction in sorted_transactions {
            let product_code_str = transaction.product_code.value().to_string();
            let current_balance = *balances.get(&product_code_str).unwrap_or(&0.0);

            // 増減数量を計算（生産・仕入は加算、売上は減算）
            let change = match transaction.inventory_type {
                InventoryType::Production | InventoryType::Purchase => transaction.quantity.value(),
                InventoryType::Sales => -transaction.quantity.value(),
            };

            let new_balance = current_balance + change;
            balances.insert(product_code_str, new_balance);

            records.push(InventoryHistoryRecord {
                date: transaction.date,
                inventory_type: transaction.inventory_type,
                product_code: transaction.product_code,
                product_name: transaction.product_name,
                base_quantity: InventoryBalance::new(current_balance)?,
                change_quantity: Quantity::new(change.abs())?,
                balance: InventoryBalance::new(new_balance)?,
            });
        }

        Ok(records)
    }
}

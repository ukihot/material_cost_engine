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
    pub freight_cost: Amount,        // 実質運賃（按分後）
    pub purchase_quantity: Quantity, // 仕入数量
    pub freight_code_str: String,    // 運賃コード（ロギング用）
    pub freight_kg_price: f64,       // 運賃Kg単価（ロギング用）
}

/// 材料費計算結果
#[derive(Debug, Clone)]
pub struct MaterialCostResult {
    pub consumptions: Vec<MaterialConsumption>,
    pub total_freight_cost: Amount, // 全材料の運賃合計
}

/// 材料費計算ドメインサービス
pub struct MaterialCostCalculationService;

impl MaterialCostCalculationService {
    /// 材料消費を計算
    pub fn calculate_material_consumption<F, P, FR>(
        production: &Production,
        formula_repo: &F,
        purchase_repo: &P,
        freight_repo: &FR,
    ) -> Result<MaterialCostResult>
    where
        F: FormulaRepository,
        P: PurchaseRepository,
        FR: FreightMasterRepository,
    {
        // 配合マスタから材料を取得
        let formulas = formula_repo.find_by_product_code(&production.product_code)?;

        let mut consumptions = Vec::new();
        let mut total_freight = Amount::zero();

        for formula in formulas {
            // 消費数量を計算
            let consumption_qty =
                Quantity::new(production.quantity.value() * formula.consumption_ratio.value())?;

            // 仕入データから単価を取得
            let purchase = purchase_repo.find_latest_price(&formula.material_code)?;

            // 運賃Kg単価を取得
            let freight_kg_price = match &purchase.freight_code {
                crate::domain::value_objects::FreightCode::DirectPrice(price) => *price,
                crate::domain::value_objects::FreightCode::Code(code) => {
                    let freight_master = freight_repo.find_by_code(code)?;
                    freight_master.kg_unit_price.value()
                }
            };

            // 運賃コードを文字列化（ロギング用）
            let freight_code_str = match &purchase.freight_code {
                crate::domain::value_objects::FreightCode::DirectPrice(price) => {
                    format!("{:.2}", price)
                }
                crate::domain::value_objects::FreightCode::Code(code) => code.clone(),
            };

            // 実質運賃（按分後） = 運賃Kg単価 × 消費数量
            let material_freight = Amount::new(freight_kg_price * consumption_qty.value())?;
            total_freight = total_freight.add(&material_freight);

            // 材料費を計算（単価のみ、運賃は別途）
            let total_cost = purchase.unit_price.multiply(consumption_qty.value());

            consumptions.push(MaterialConsumption {
                material_code: formula.material_code.clone(),
                material_name: purchase.product_name.clone(),
                quantity: consumption_qty,
                unit_price: purchase.unit_price,
                total_cost,
                freight_cost: material_freight,
                purchase_quantity: purchase.quantity,
                freight_code_str,
                freight_kg_price,
            });
        }

        Ok(MaterialCostResult {
            consumptions,
            total_freight_cost: total_freight,
        })
    }

    /// 原砂金額を計算
    pub fn calculate_raw_material_cost(consumptions: &[MaterialConsumption]) -> Amount {
        consumptions
            .iter()
            .fold(Amount::zero(), |acc, c| acc.add(&c.total_cost))
    }

    /// 原単位を計算（円/t）
    /// 原単位 = (原砂金額 ÷ 消費砂量) × 1000
    pub fn calculate_unit_cost(raw_material_cost: &Amount, total_consumption_kg: f64) -> Amount {
        if total_consumption_kg == 0.0 {
            return Amount::zero();
        }
        // 円/kg の原単位を求める
        let unit_cost_per_kg = raw_material_cost.value() / total_consumption_kg;
        // 円/t に変換（1t = 1000kg）
        Amount::new(unit_cost_per_kg * 1000.0).unwrap_or_else(|_| Amount::zero())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // モックリポジトリ
    struct MockFormulaRepository {
        formulas: HashMap<String, Vec<FormulaEntry>>,
    }

    impl FormulaRepository for MockFormulaRepository {
        fn find_by_product_code(&self, product_code: &ProductCode) -> Result<Vec<FormulaEntry>> {
            self.formulas
                .get(product_code.value())
                .cloned()
                .ok_or_else(|| color_eyre::eyre::eyre!("配合マスタが見つかりません"))
        }
    }

    struct MockPurchaseRepository {
        purchases: HashMap<String, Purchase>,
    }

    impl PurchaseRepository for MockPurchaseRepository {
        fn find_latest_price(&self, product_code: &ProductCode) -> Result<Purchase> {
            self.purchases
                .get(product_code.value())
                .cloned()
                .ok_or_else(|| color_eyre::eyre::eyre!("仕入データが見つかりません"))
        }
    }

    struct MockFreightMasterRepository {
        freight_masters: HashMap<String, FreightMaster>,
    }

    impl FreightMasterRepository for MockFreightMasterRepository {
        fn find_by_code(&self, code: &str) -> Result<FreightMaster> {
            self.freight_masters
                .get(code)
                .cloned()
                .ok_or_else(|| color_eyre::eyre::eyre!("運賃マスタが見つかりません"))
        }
    }

    #[test]
    fn test_freight_calculation_with_direct_price() {
        // 運賃が直接数値で指定された場合のテスト
        // 運賃Kg単価: 10.0円/kg
        // 仕入数量: 100kg
        // 消費数量: 30kg
        // 期待される実質運賃: 10.0 × 30 = 300円

        let production = Production::new(
            ProductCode::new("P001".to_string()).unwrap(),
            Quantity::new(1000.0).unwrap(),
            YieldRate::new(0.95).unwrap(),
            Amount::new(100.0).unwrap(),
            Amount::new(50.0).unwrap(),
        );

        let mut formulas = HashMap::new();
        formulas.insert(
            "P001".to_string(),
            vec![FormulaEntry::new(
                ProductCode::new("M001".to_string()).unwrap(),
                ConsumptionRatio::new(0.03).unwrap(), // 3% = 30kg
            )],
        );

        let mut purchases = HashMap::new();
        purchases.insert(
            "M001".to_string(),
            Purchase::new(
                "材料A".to_string(),
                Amount::new(50.0).unwrap(),
                Quantity::new(100.0).unwrap(),
                FreightCode::DirectPrice(10.0), // 直接指定: 10円/kg
            ),
        );

        let formula_repo = MockFormulaRepository { formulas };
        let purchase_repo = MockPurchaseRepository { purchases };
        let freight_repo = MockFreightMasterRepository {
            freight_masters: HashMap::new(),
        };

        let result = MaterialCostCalculationService::calculate_material_consumption(
            &production,
            &formula_repo,
            &purchase_repo,
            &freight_repo,
        )
        .unwrap();

        assert_eq!(result.consumptions.len(), 1);
        let consumption = &result.consumptions[0];

        // 消費数量の確認
        assert_eq!(consumption.quantity.value(), 30.0);

        // 運賃Kg単価の確認
        assert_eq!(consumption.freight_kg_price, 10.0);

        // 実質運賃の確認: 10.0 × 30 = 300
        assert_eq!(consumption.freight_cost.value(), 300.0);

        // 合計運賃の確認
        assert_eq!(result.total_freight_cost.value(), 300.0);
    }

    #[test]
    fn test_freight_calculation_with_master_code() {
        // 運賃マスタから取得する場合のテスト
        // 運賃コード: T01
        // 運賃Kg単価（マスタ）: 15.0円/kg
        // 仕入数量: 200kg
        // 消費数量: 50kg
        // 期待される実質運賃: 15.0 × 50 = 750円

        let production = Production::new(
            ProductCode::new("P002".to_string()).unwrap(),
            Quantity::new(500.0).unwrap(),
            YieldRate::new(0.90).unwrap(),
            Amount::new(200.0).unwrap(),
            Amount::new(100.0).unwrap(),
        );

        let mut formulas = HashMap::new();
        formulas.insert(
            "P002".to_string(),
            vec![FormulaEntry::new(
                ProductCode::new("M002".to_string()).unwrap(),
                ConsumptionRatio::new(0.1).unwrap(), // 10% = 50kg
            )],
        );

        let mut purchases = HashMap::new();
        purchases.insert(
            "M002".to_string(),
            Purchase::new(
                "材料B".to_string(),
                Amount::new(80.0).unwrap(),
                Quantity::new(200.0).unwrap(),
                FreightCode::Code("T01".to_string()),
            ),
        );

        let mut freight_masters = HashMap::new();
        freight_masters.insert(
            "T01".to_string(),
            FreightMaster::new(
                "T01".to_string(),
                PatternName::new("パターンA".to_string()).unwrap(),
                Amount::new(15.0).unwrap(), // 15円/kg
                TransactionDate::new("2026-01-01".to_string()).unwrap(),
                None,
            )
            .unwrap(),
        );

        let formula_repo = MockFormulaRepository { formulas };
        let purchase_repo = MockPurchaseRepository { purchases };
        let freight_repo = MockFreightMasterRepository { freight_masters };

        let result = MaterialCostCalculationService::calculate_material_consumption(
            &production,
            &formula_repo,
            &purchase_repo,
            &freight_repo,
        )
        .unwrap();

        assert_eq!(result.consumptions.len(), 1);
        let consumption = &result.consumptions[0];

        // 消費数量の確認
        assert_eq!(consumption.quantity.value(), 50.0);

        // 運賃Kg単価の確認
        assert_eq!(consumption.freight_kg_price, 15.0);

        // 実質運賃の確認: 15.0 × 50 = 750
        assert_eq!(consumption.freight_cost.value(), 750.0);

        // 合計運賃の確認
        assert_eq!(result.total_freight_cost.value(), 750.0);
    }

    #[test]
    fn test_freight_calculation_with_multiple_materials() {
        // 複数材料の運賃計算テスト
        // 材料1: 直接指定 20円/kg, 消費数量 40kg → 実質運賃 800円
        // 材料2: マスタT02 25円/kg, 消費数量 60kg → 実質運賃 1500円
        // 合計運賃: 2300円

        let production = Production::new(
            ProductCode::new("P003".to_string()).unwrap(),
            Quantity::new(1000.0).unwrap(),
            YieldRate::new(0.92).unwrap(),
            Amount::new(150.0).unwrap(),
            Amount::new(75.0).unwrap(),
        );

        let mut formulas = HashMap::new();
        formulas.insert(
            "P003".to_string(),
            vec![
                FormulaEntry::new(
                    ProductCode::new("M003".to_string()).unwrap(),
                    ConsumptionRatio::new(0.04).unwrap(), // 4% = 40kg
                ),
                FormulaEntry::new(
                    ProductCode::new("M004".to_string()).unwrap(),
                    ConsumptionRatio::new(0.06).unwrap(), // 6% = 60kg
                ),
            ],
        );

        let mut purchases = HashMap::new();
        purchases.insert(
            "M003".to_string(),
            Purchase::new(
                "材料C".to_string(),
                Amount::new(60.0).unwrap(),
                Quantity::new(150.0).unwrap(),
                FreightCode::DirectPrice(20.0), // 20円/kg
            ),
        );
        purchases.insert(
            "M004".to_string(),
            Purchase::new(
                "材料D".to_string(),
                Amount::new(70.0).unwrap(),
                Quantity::new(180.0).unwrap(),
                FreightCode::Code("T02".to_string()),
            ),
        );

        let mut freight_masters = HashMap::new();
        freight_masters.insert(
            "T02".to_string(),
            FreightMaster::new(
                "T02".to_string(),
                PatternName::new("パターンB".to_string()).unwrap(),
                Amount::new(25.0).unwrap(), // 25円/kg
                TransactionDate::new("2026-01-01".to_string()).unwrap(),
                None,
            )
            .unwrap(),
        );

        let formula_repo = MockFormulaRepository { formulas };
        let purchase_repo = MockPurchaseRepository { purchases };
        let freight_repo = MockFreightMasterRepository { freight_masters };

        let result = MaterialCostCalculationService::calculate_material_consumption(
            &production,
            &formula_repo,
            &purchase_repo,
            &freight_repo,
        )
        .unwrap();

        assert_eq!(result.consumptions.len(), 2);

        // 材料1の確認
        let consumption1 = &result.consumptions[0];
        assert_eq!(consumption1.material_code.value(), "M003");
        assert_eq!(consumption1.quantity.value(), 40.0);
        assert_eq!(consumption1.freight_kg_price, 20.0);
        assert_eq!(consumption1.freight_cost.value(), 800.0); // 20 × 40

        // 材料2の確認
        let consumption2 = &result.consumptions[1];
        assert_eq!(consumption2.material_code.value(), "M004");
        assert_eq!(consumption2.quantity.value(), 60.0);
        assert_eq!(consumption2.freight_kg_price, 25.0);
        assert_eq!(consumption2.freight_cost.value(), 1500.0); // 25 × 60

        // 合計運賃の確認
        assert_eq!(result.total_freight_cost.value(), 2300.0); // 800 + 1500
    }

    #[test]
    fn test_freight_calculation_with_zero_consumption() {
        // 消費数量が0の場合のテスト（エッジケース）
        // 運賃Kg単価: 10円/kg
        // 消費数量: 0kg
        // 期待される実質運賃: 0円

        let production = Production::new(
            ProductCode::new("P004".to_string()).unwrap(),
            Quantity::new(0.0).unwrap(), // 生産数量0
            YieldRate::new(0.95).unwrap(),
            Amount::new(0.0).unwrap(),
            Amount::new(0.0).unwrap(),
        );

        let mut formulas = HashMap::new();
        formulas.insert(
            "P004".to_string(),
            vec![FormulaEntry::new(
                ProductCode::new("M005".to_string()).unwrap(),
                ConsumptionRatio::new(0.05).unwrap(),
            )],
        );

        let mut purchases = HashMap::new();
        purchases.insert(
            "M005".to_string(),
            Purchase::new(
                "材料E".to_string(),
                Amount::new(100.0).unwrap(),
                Quantity::new(100.0).unwrap(),
                FreightCode::DirectPrice(10.0),
            ),
        );

        let formula_repo = MockFormulaRepository { formulas };
        let purchase_repo = MockPurchaseRepository { purchases };
        let freight_repo = MockFreightMasterRepository {
            freight_masters: HashMap::new(),
        };

        let result = MaterialCostCalculationService::calculate_material_consumption(
            &production,
            &formula_repo,
            &purchase_repo,
            &freight_repo,
        )
        .unwrap();

        assert_eq!(result.consumptions.len(), 1);
        let consumption = &result.consumptions[0];

        // 消費数量が0
        assert_eq!(consumption.quantity.value(), 0.0);

        // 実質運賃も0
        assert_eq!(consumption.freight_cost.value(), 0.0);

        // 合計運賃も0
        assert_eq!(result.total_freight_cost.value(), 0.0);
    }

    #[test]
    fn test_freight_calculation_formula() {
        // 運賃計算式の検証テスト
        // 実質運賃 = 運賃Kg単価 × 消費数量
        //
        // ケース: 運賃Kg単価 12.5円/kg, 消費数量 37.5kg
        // 期待される実質運賃: 12.5 × 37.5 = 468.75円

        let production = Production::new(
            ProductCode::new("P005".to_string()).unwrap(),
            Quantity::new(1250.0).unwrap(),
            YieldRate::new(0.88).unwrap(),
            Amount::new(120.0).unwrap(),
            Amount::new(60.0).unwrap(),
        );

        let mut formulas = HashMap::new();
        formulas.insert(
            "P005".to_string(),
            vec![FormulaEntry::new(
                ProductCode::new("M006".to_string()).unwrap(),
                ConsumptionRatio::new(0.03).unwrap(), // 3% = 37.5kg
            )],
        );

        let mut purchases = HashMap::new();
        purchases.insert(
            "M006".to_string(),
            Purchase::new(
                "材料F".to_string(),
                Amount::new(90.0).unwrap(),
                Quantity::new(250.0).unwrap(),
                FreightCode::DirectPrice(12.5), // 12.5円/kg
            ),
        );

        let formula_repo = MockFormulaRepository { formulas };
        let purchase_repo = MockPurchaseRepository { purchases };
        let freight_repo = MockFreightMasterRepository {
            freight_masters: HashMap::new(),
        };

        let result = MaterialCostCalculationService::calculate_material_consumption(
            &production,
            &formula_repo,
            &purchase_repo,
            &freight_repo,
        )
        .unwrap();

        assert_eq!(result.consumptions.len(), 1);
        let consumption = &result.consumptions[0];

        // 消費数量: 1250 × 0.03 = 37.5
        assert_eq!(consumption.quantity.value(), 37.5);

        // 運賃Kg単価
        assert_eq!(consumption.freight_kg_price, 12.5);

        // 実質運賃: 12.5 × 37.5 = 468.75
        assert_eq!(consumption.freight_cost.value(), 468.75);

        // 合計運賃
        assert_eq!(result.total_freight_cost.value(), 468.75);
    }

    #[test]
    fn test_unit_cost_calculation() {
        // 原単位（円/t）の計算テスト
        // 原単位 = (原砂金額 ÷ 消費砂量) × 1000
        //
        // ケース1: 原砂金額 5000円, 消費砂量 100kg
        // 期待される原単位: (5000 ÷ 100) × 1000 = 50,000円/t
        let raw_material_cost = Amount::new(5000.0).unwrap();
        let total_consumption_kg = 100.0;
        let unit_cost = MaterialCostCalculationService::calculate_unit_cost(
            &raw_material_cost,
            total_consumption_kg,
        );
        assert_eq!(unit_cost.value(), 50000.0);

        // ケース2: 原砂金額 12000円, 消費砂量 80kg
        // 期待される原単位: (12000 ÷ 80) × 1000 = 150,000円/t
        let raw_material_cost = Amount::new(12000.0).unwrap();
        let total_consumption_kg = 80.0;
        let unit_cost = MaterialCostCalculationService::calculate_unit_cost(
            &raw_material_cost,
            total_consumption_kg,
        );
        assert_eq!(unit_cost.value(), 150000.0);

        // ケース3: 原砂金額 7500円, 消費砂量 50kg
        // 期待される原単位: (7500 ÷ 50) × 1000 = 150,000円/t
        let raw_material_cost = Amount::new(7500.0).unwrap();
        let total_consumption_kg = 50.0;
        let unit_cost = MaterialCostCalculationService::calculate_unit_cost(
            &raw_material_cost,
            total_consumption_kg,
        );
        assert_eq!(unit_cost.value(), 150000.0);

        // ケース4: 消費砂量が0の場合（ゼロ除算対策）
        // 期待される原単位: 0円/t
        let raw_material_cost = Amount::new(5000.0).unwrap();
        let total_consumption_kg = 0.0;
        let unit_cost = MaterialCostCalculationService::calculate_unit_cost(
            &raw_material_cost,
            total_consumption_kg,
        );
        assert_eq!(unit_cost.value(), 0.0);
    }
}

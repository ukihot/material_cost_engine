use crate::domain::repositories::*;
use crate::domain::sheet_schema::ProductionSheetSchema;
use crate::usecase::dtos::*;
use crate::usecase::interactor::CalculateMaterialCostInteractor;
use crate::usecase::ports::*;
use calamine::{Data, Reader, Xlsx};
use color_eyre::{Result, eyre::eyre};

// 共通ヘルパー関数
fn get_cell_string(row: &[Data], index: usize) -> String {
    row.get(index)
        .map(|c| c.to_string().trim().to_string())
        .unwrap_or_default()
}

/// Excelコントローラ
pub struct ExcelController<'a, F, P, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    O: CalculateMaterialCostOutputPort,
{
    formula_repo: &'a F,
    purchase_repo: &'a P,
    output_port: O,
    input_file_path: String,
    output_file_path: String,
}

impl<'a, F, P, O> ExcelController<'a, F, P, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    O: CalculateMaterialCostOutputPort,
{
    pub fn new(
        formula_repo: &'a F,
        purchase_repo: &'a P,
        output_port: O,
        input_file_path: String,
        output_file_path: String,
    ) -> Self {
        Self {
            formula_repo,
            purchase_repo,
            output_port,
            input_file_path,
            output_file_path,
        }
    }

    /// 材料費計算を実行
    pub fn execute(
        &mut self,
        workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    ) -> Result<()> {
        // シートスキーマをチェック
        let schema = Self::validate_production_sheet_schema(workbook)?;

        // プレゼンターを初期化
        let config = PresenterConfigDto {
            input_file_path: self.input_file_path.clone(),
            output_file_path: self.output_file_path.clone(),
            production_sheet_schema: schema.clone(),
        };
        self.output_port.initialize(config)?;

        // 生産データを読み込む
        let productions = Self::load_production_data(workbook, &schema)?;

        // インタラクタを作成してユースケースを実行
        let mut interactor = CalculateMaterialCostInteractor::new(
            self.formula_repo,
            self.purchase_repo,
            &mut self.output_port,
        );
        interactor.execute(productions)?;

        // プレゼンターを終了
        self.output_port.finalize()?;

        Ok(())
    }

    /// 【入庫】生産シートのスキーマをバリデーション
    fn validate_production_sheet_schema(
        workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    ) -> Result<ProductionSheetSchema> {
        let sheet_name = "【入庫】生産";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        if rows.is_empty() {
            return Err(eyre!("【入庫】生産シートが空です"));
        }

        let header_row = rows[0];
        let headers: Vec<String> = header_row
            .iter()
            .map(|cell| cell.to_string().trim().to_string())
            .collect();

        println!("\nシートスキーマをチェック中...");
        let schema = ProductionSheetSchema::from_headers(&headers)?;
        println!("  ✓ 【入庫】生産シートのスキーマチェック完了");

        Ok(schema)
    }

    /// 【入庫】生産シートからデータを読み込む
    fn load_production_data(
        workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
        schema: &ProductionSheetSchema,
    ) -> Result<Vec<ProductionDto>> {
        let sheet_name = "【入庫】生産";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        let mut productions = Vec::new();

        for (row_idx, row) in rows.iter().enumerate().skip(1) {
            let product_code = get_cell_string(row, schema.product_code().value());
            let production_number = get_cell_string(row, schema.production_number().value());
            let quantity_str = get_cell_string(row, schema.quantity().value());
            let yield_rate_str = get_cell_string(row, schema.yield_rate().value());
            let coagulant_str = get_cell_string(row, schema.coagulant().value());
            let clay_treatment_str = get_cell_string(row, schema.clay_treatment().value());

            // 必須項目チェック
            if product_code.is_empty() || quantity_str.is_empty() || yield_rate_str.is_empty() {
                return Err(eyre!(
                    "【入庫】生産シートの{}行目に必須データが欠けています\n  商品コード: {}\n  生産数量: {}\n  歩留率: {}",
                    row_idx + 1,
                    if product_code.is_empty() {
                        "空白"
                    } else {
                        &product_code
                    },
                    if quantity_str.is_empty() {
                        "空白"
                    } else {
                        &quantity_str
                    },
                    if yield_rate_str.is_empty() {
                        "空白"
                    } else {
                        &yield_rate_str
                    }
                ));
            }

            let quantity: f64 = quantity_str.parse().map_err(|_| {
                eyre!(
                    "{}行目: 生産数量が数値ではありません: {}",
                    row_idx + 1,
                    quantity_str
                )
            })?;

            let yield_rate: f64 = yield_rate_str.parse().map_err(|_| {
                eyre!(
                    "{}行目: 歩留率が数値ではありません: {}",
                    row_idx + 1,
                    yield_rate_str
                )
            })?;

            let coagulant_cost: f64 = if coagulant_str.is_empty() {
                0.0
            } else {
                coagulant_str.parse().map_err(|_| {
                    eyre!(
                        "{}行目: 凝集剤が数値ではありません: {}",
                        row_idx + 1,
                        coagulant_str
                    )
                })?
            };

            let clay_treatment_cost: f64 = if clay_treatment_str.is_empty() {
                0.0
            } else {
                clay_treatment_str.parse().map_err(|_| {
                    eyre!(
                        "{}行目: 粘土処理が数値ではありません: {}",
                        row_idx + 1,
                        clay_treatment_str
                    )
                })?
            };

            productions.push(ProductionDto {
                row_number: row_idx + 1,
                product_code,
                production_number,
                quantity,
                yield_rate,
                coagulant_cost,
                clay_treatment_cost,
            });
        }

        Ok(productions)
    }
}

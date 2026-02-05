use crate::domain::sheet_schema::ProductionSheetSchema;
use crate::usecase::dtos::*;
use crate::usecase::ports::*;
use calamine::{Reader, Xlsx, open_workbook};
use color_eyre::Result;
use rust_xlsxwriter::Workbook;

/// Excelプレゼンター
pub struct ExcelPresenter {
    config: Option<PresenterConfigDto>,
    workbook: Option<Workbook>,
    results: Vec<MaterialCostResultDto>,
}

impl ExcelPresenter {
    pub fn new() -> Self {
        Self {
            config: None,
            workbook: None,
            results: Vec::new(),
        }
    }

    fn schema(&self) -> &ProductionSheetSchema {
        &self.config.as_ref().unwrap().production_sheet_schema
    }
}

impl CalculateMaterialCostOutputPort for ExcelPresenter {
    fn initialize(&mut self, config: PresenterConfigDto) -> Result<()> {
        println!("Excelファイルを準備中...");

        // 既存のワークブックを読み込んでコピー
        let mut source_workbook: Xlsx<_> = open_workbook(&config.input_file_path)?;
        let mut new_workbook = Workbook::new();

        // すべてのシートをコピー
        let sheet_names = source_workbook.sheet_names().to_owned();
        for sheet_name in &sheet_names {
            if let Ok(range) = source_workbook.worksheet_range(sheet_name) {
                let worksheet = new_workbook.add_worksheet();
                worksheet.set_name(sheet_name)?;

                // データをコピー
                for (row_idx, row) in range.rows().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        let value = format!("{}", cell);
                        if !value.is_empty() {
                            worksheet.write_string(row_idx as u32, col_idx as u16, &value)?;
                        }
                    }
                }
            }
        }

        self.workbook = Some(new_workbook);
        self.config = Some(config);

        println!("  ✓ Excelファイルの準備完了");
        Ok(())
    }

    fn present_no_data(&mut self) {
        println!("  ℹ️  【入庫】生産シートにデータがありません（ヘッダーのみ）");
    }

    fn present_validation_error(&mut self, row_number: usize, message: &str) {
        eprintln!("\n❌ バリデーションエラー（{}行目）:", row_number);
        eprintln!("{}", message);
    }

    fn present_calculation_start(&mut self, total_rows: usize) {
        println!("\n🔧 【入庫】生産シートの処理を開始...");
        println!("  ✓ データ行数: {} 行", total_rows);
    }

    fn present_processing_row(&mut self, row_number: usize, product_code: &str) {
        println!(
            "\n  処理中: 行{} - 商品コード: {}",
            row_number, product_code
        );
    }

    fn present_material_consumptions(&mut self, consumptions: &[MaterialConsumptionDto]) {
        println!("    配合マスタ: {} 種類の材料", consumptions.len());
        for consumption in consumptions {
            println!(
                "      {} ({}): {:.2} kg",
                consumption.material_name, consumption.material_code, consumption.quantity
            );
            println!(
                "        単価: {:.2} 円 → 金額: {:.2} 円",
                consumption.unit_price, consumption.total_cost
            );
        }
    }

    fn present_calculation_result(&mut self, result: &MaterialCostResultDto) {
        println!("    原砂金額合計: {:.2} 円", result.raw_material_cost);
        println!("    原単位（円/t）: {:.2}", result.unit_cost);
        println!("    原砂歩留金額: {:.2} 円", result.yield_cost);
        println!("    凝集剤: {:.2} 円", result.coagulant_cost);
        println!("    粘土処理: {:.2} 円", result.clay_treatment_cost);
        println!("    材料費合計: {:.2} 円", result.total_material_cost);

        // 結果を保存（後でまとめて書き込む）
        self.results.push(result.clone());
    }

    fn present_completion(&mut self) {
        println!("\n✅ 【入庫】生産シートの処理が完了しました");
    }

    fn present_error(&mut self, message: &str) {
        eprintln!("\n❌ エラー: {}", message);
    }

    fn finalize(&mut self) -> Result<()> {
        if let Some(config) = &self.config
            && let Some(workbook) = &mut self.workbook
        {
            println!("\nExcelファイルに結果を書き込み中...");

            // 【入庫】生産シートを取得
            let sheet = workbook.worksheet_from_name("【入庫】生産")?;
            let schema = &config.production_sheet_schema;

            // 結果を書き込み
            for result in &self.results {
                let row = result.row_number as u32;
                sheet.write_number(
                    row,
                    schema.raw_material_cost().as_u16(),
                    result.raw_material_cost,
                )?;
                sheet.write_number(row, schema.unit_cost().as_u16(), result.unit_cost)?;
                sheet.write_number(row, schema.yield_cost().as_u16(), result.yield_cost)?;
                sheet.write_number(
                    row,
                    schema.material_cost().as_u16(),
                    result.total_material_cost,
                )?;
            }

            println!("  ✓ 結果の書き込み完了");

            // ファイルを保存
            println!("\nExcelファイルを保存中...");
            workbook.save(&config.output_file_path)?;
            println!("  ✓ 保存完了: {}", config.output_file_path);
        }
        Ok(())
    }
}

use crate::usecase::dtos::*;
use crate::usecase::ports::*;
use calamine::{Reader, Xlsx, open_workbook};
use color_eyre::Result;
use rust_xlsxwriter::Workbook;

/// Excelプレゼンター
pub struct ExcelPresenter {
    input_file_path: String,
    output_file_path: String,
    workbook: Option<Workbook>,
    results: Vec<MaterialCostResultDto>,
    history_records: Vec<InventoryHistoryRecordDto>,
    logs: Vec<String>,
    // 【入庫】生産シートの列インデックス
    production_col_raw_material_cost: Option<usize>,
    production_col_yield_cost: Option<usize>,
    production_col_coagulant_cost: Option<usize>,
    production_col_clay_treatment_cost: Option<usize>,
    production_col_freight_cost: Option<usize>,
    production_col_total_material_cost: Option<usize>,
}

impl ExcelPresenter {
    pub fn new(input_file_path: String, output_file_path: String) -> Result<Self> {
        let mut presenter = Self {
            input_file_path: input_file_path.clone(),
            output_file_path,
            workbook: None,
            results: Vec::new(),
            history_records: Vec::new(),
            logs: Vec::new(),
            production_col_raw_material_cost: None,
            production_col_yield_cost: None,
            production_col_coagulant_cost: None,
            production_col_clay_treatment_cost: None,
            production_col_freight_cost: None,
            production_col_total_material_cost: None,
        };

        // Excelファイルを準備
        presenter.initialize_workbook()?;
        // 列インデックスを取得
        presenter.load_column_indices()?;
        Ok(presenter)
    }

    fn initialize_workbook(&mut self) -> Result<()> {
        self.log("Excelファイルを準備中...".to_string());

        // 既存のワークブックを読み込んでコピー
        let mut source_workbook: Xlsx<_> = open_workbook(&self.input_file_path)?;
        let mut new_workbook = Workbook::new();

        // 日付フォーマットを作成
        let date_format = rust_xlsxwriter::Format::new().set_num_format("yyyy-mm-dd");

        // すべてのシートをコピー
        let sheet_names = source_workbook.sheet_names().to_owned();
        for sheet_name in &sheet_names {
            if let Ok(range) = source_workbook.worksheet_range(sheet_name) {
                let worksheet = new_workbook.add_worksheet();
                worksheet.set_name(sheet_name)?;

                // データをコピー
                for (row_idx, row) in range.rows().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        match cell {
                            calamine::Data::Int(i) => {
                                worksheet.write_number(
                                    row_idx as u32,
                                    col_idx as u16,
                                    *i as f64,
                                )?;
                            }
                            calamine::Data::Float(f) => {
                                worksheet.write_number(row_idx as u32, col_idx as u16, *f)?;
                            }
                            calamine::Data::DateTime(dt) => {
                                // DateTimeの場合、シリアル値として書き込み、日付フォーマットを適用
                                let dt_str = dt.to_string();
                                if let Ok(serial) = dt_str.parse::<f64>() {
                                    worksheet.write_number_with_format(
                                        row_idx as u32,
                                        col_idx as u16,
                                        serial,
                                        &date_format,
                                    )?;
                                } else {
                                    worksheet.write_string(
                                        row_idx as u32,
                                        col_idx as u16,
                                        &dt_str,
                                    )?;
                                }
                            }
                            calamine::Data::DateTimeIso(dt_str) => {
                                worksheet.write_string(row_idx as u32, col_idx as u16, dt_str)?;
                            }
                            _ => {
                                let value = format!("{}", cell);
                                if !value.is_empty() {
                                    worksheet.write_string(
                                        row_idx as u32,
                                        col_idx as u16,
                                        &value,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.workbook = Some(new_workbook);
        self.log("  ✓ Excelファイルの準備完了".to_string());
        Ok(())
    }

    fn load_column_indices(&mut self) -> Result<()> {
        // 元のExcelファイルから列インデックスを取得
        let mut source_workbook: Xlsx<_> = open_workbook(&self.input_file_path)?;

        // 【入庫】生産シートのヘッダー行を読み込む
        let sheet_name = "【入庫】生産";
        if let Ok(range) = source_workbook.worksheet_range(sheet_name)
            && let Some(header_row) = range.rows().next() {
                // 各列のインデックスを取得
                self.production_col_raw_material_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "原砂金額");

                self.production_col_yield_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "原砂歩留金額");

                self.production_col_coagulant_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "凝集剤");

                self.production_col_clay_treatment_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "粘土処理");

                self.production_col_freight_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "材料運賃");

                self.production_col_total_material_cost = header_row
                    .iter()
                    .position(|cell| cell.to_string().trim() == "材料費");

                self.log(format!(
                    "  ✓ 列インデックス取得: 原砂金額={:?}, 原砂歩留金額={:?}, 凝集剤={:?}, 粘土処理={:?}, 材料運賃={:?}, 材料費={:?}",
                    self.production_col_raw_material_cost,
                    self.production_col_yield_cost,
                    self.production_col_coagulant_cost,
                    self.production_col_clay_treatment_cost,
                    self.production_col_freight_cost,
                    self.production_col_total_material_cost
                ));
            }

        Ok(())
    }

    fn log(&mut self, message: String) {
        println!("{}", message);
        self.logs.push(message);
    }

    fn log_error(&mut self, message: String) {
        eprintln!("{}", message);
        self.logs.push(message);
    }

    /// Excelファイルに結果を書き込んで保存
    pub fn finalize(&mut self) -> Result<()> {
        let Some(mut workbook) = self.workbook.take() else {
            return Ok(());
        };

        self.log("\nExcelファイルに結果を書き込み中...".to_string());

        // 【入庫】生産シートに結果を書き込み
        if !self.results.is_empty() {
            let sheet = workbook.worksheet_from_name("【入庫】生産")?;

            for result in &self.results {
                let row = (result.row_number - 1) as u32;
                // 四捨五入して整数に変換
                if let Some(col) = self.production_col_raw_material_cost {
                    sheet.write_number(row, col as u16, result.raw_material_cost.round())?;
                }
                if let Some(col) = self.production_col_yield_cost {
                    sheet.write_number(row, col as u16, result.yield_cost.round())?;
                }
                if let Some(col) = self.production_col_coagulant_cost {
                    sheet.write_number(row, col as u16, result.coagulant_cost.round())?;
                }
                if let Some(col) = self.production_col_clay_treatment_cost {
                    sheet.write_number(row, col as u16, result.clay_treatment_cost.round())?;
                }
                if let Some(col) = self.production_col_freight_cost {
                    sheet.write_number(row, col as u16, result.freight_cost.round())?;
                }
                if let Some(col) = self.production_col_total_material_cost {
                    sheet.write_number(row, col as u16, result.total_material_cost.round())?;
                }
            }

            self.log("  ✓ 材料費計算結果の書き込み完了".to_string());
        }

        // 入出庫履歴シートに書き込み
        if !self.history_records.is_empty() {
            self.log("\n入出庫履歴シートに書き込み中...".to_string());
            let history_sheet = workbook.worksheet_from_name("【集計】入出庫履歴")?;

            // 日付フォーマットを作成
            let date_format = rust_xlsxwriter::Format::new().set_num_format("yyyy-mm-dd");

            for (idx, record) in self.history_records.iter().enumerate() {
                let row = (idx + 1) as u32;
                // 日付を文字列として書き込み、フォーマットを適用
                history_sheet.write_string_with_format(row, 0, &record.date, &date_format)?;
                history_sheet.write_string(row, 1, &record.inventory_type)?;
                history_sheet.write_string(row, 2, &record.product_code)?;
                history_sheet.write_string(row, 3, &record.product_name)?;
                history_sheet.write_number(row, 4, record.base_quantity)?;
                history_sheet.write_number(row, 5, record.change_quantity)?;
                history_sheet.write_number(row, 6, record.balance)?;
            }

            self.log("  ✓ 入出庫履歴の書き込み完了".to_string());
        }

        // syslogシートを作成してログを書き込み
        let syslog_sheet = workbook.add_worksheet();
        syslog_sheet.set_name("syslog")?;

        for (idx, log_message) in self.logs.iter().enumerate() {
            syslog_sheet.write_string(idx as u32, 0, log_message)?;
        }

        // ファイルを保存
        self.log("\nExcelファイルを保存中...".to_string());
        workbook.save(&self.output_file_path)?;
        self.log(format!("  ✓ 保存完了: {}", self.output_file_path));

        Ok(())
    }
}

impl CalculateMaterialCostOutputPort for ExcelPresenter {
    fn present_no_data(&mut self) {
        self.log("  ℹ️  【入庫】生産シートにデータがありません（ヘッダーのみ）".to_string());
    }

    fn present_calculation_start(&mut self, total_rows: usize) {
        self.log("\n🔧 【入庫】生産シートの処理を開始...".to_string());
        self.log(format!("  ✓ データ行数: {} 行", total_rows));
    }

    fn present_processing_row(&mut self, row_number: usize, product_code: &str) {
        self.log(format!(
            "\n  処理中: 行{} - 商品コード: {}",
            row_number, product_code
        ));
    }

    fn present_material_consumptions(&mut self, consumptions: &[MaterialConsumptionDto]) {
        self.log(format!("    配合マスタ: {} 種類の材料", consumptions.len()));
        for consumption in consumptions {
            self.log(format!(
                "      {} ({}): 消費数量 {:.2} kg",
                consumption.material_name, consumption.material_code, consumption.quantity
            ));
            self.log(format!(
                "        単価: {:.2} 円 → 金額: {:.2} 円",
                consumption.unit_price, consumption.total_cost
            ));
            self.log(format!(
                "        仕入数量: {:.2} kg, 運賃コード: {}, 運賃Kg単価: {:.2} 円/kg",
                consumption.purchase_quantity,
                consumption.freight_code_str,
                consumption.freight_kg_price
            ));
            self.log(format!(
                "        実質運賃（按分後）: {:.2} 円 (= {:.2} × {:.2})",
                consumption.freight_cost, consumption.freight_kg_price, consumption.quantity
            ));
        }
    }

    fn present_calculation_result(&mut self, result: &MaterialCostResultDto) {
        self.log(format!(
            "    原砂金額合計: {:.2} 円",
            result.raw_material_cost
        ));
        self.log(format!("    原砂歩留金額: {:.2} 円", result.yield_cost));
        self.log(format!("    凝集剤: {:.2} 円", result.coagulant_cost));
        self.log(format!(
            "    粘土処理: {:.2} 円",
            result.clay_treatment_cost
        ));
        self.log(format!("    運賃: {:.2} 円", result.freight_cost));
        self.log(format!(
            "    材料費合計: {:.2} 円",
            result.total_material_cost
        ));

        // 結果を保存（後でまとめて書き込む）
        self.results.push(result.clone());
    }

    fn present_completion(&mut self) {
        self.log("\n✅ 【入庫】生産シートの処理が完了しました".to_string());
    }

    fn present_error(&mut self, message: &str) {
        self.log_error(format!("\n❌ エラー: {}", message));
    }
}

impl CreateInventoryHistoryOutputPort for ExcelPresenter {
    fn present_history_start(&mut self) {
        self.log("\n🔧 入出庫履歴の作成を開始...".to_string());
    }

    fn present_history_record(&mut self, record: &InventoryHistoryRecordDto) {
        self.history_records.push(record.clone());
    }

    fn present_history_completion(&mut self, total_records: usize) {
        self.log(format!("  ✓ 入出庫履歴レコード数: {} 件", total_records));
        self.log("✅ 入出庫履歴の作成が完了しました".to_string());
    }

    fn present_history_error(&mut self, message: &str) {
        self.log_error(format!("\n❌ 入出庫履歴エラー: {}", message));
    }

    fn finalize(&mut self) -> Result<()> {
        self.finalize()
    }
}

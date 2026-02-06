use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::domain::sheet_schema::*;
use crate::domain::value_objects::*;
use calamine::{Data, Reader, Xlsx};
use chrono::Datelike;
use color_eyre::{Result, eyre::eyre};
use std::collections::HashMap;

// 共通ヘルパー関数
fn find_column_index(header_row: &[Data], column_name: &str, sheet_name: &str) -> Result<usize> {
    header_row
        .iter()
        .position(|cell| cell.to_string().trim() == column_name)
        .ok_or_else(|| {
            eyre!(
                "{}シート: 列 '{}' が見つかりません",
                sheet_name,
                column_name
            )
        })
}

fn get_cell_string(row: &[Data], index: usize) -> String {
    row.get(index)
        .map(|c| c.to_string().trim().to_string())
        .unwrap_or_default()
}

/// Excelの日付セルを文字列に変換
fn get_cell_date_string(row: &[Data], index: usize) -> String {
    match row.get(index) {
        Some(Data::DateTime(dt)) => {
            // ExcelDateTimeのシリアル値を取得して日付に変換
            // ExcelDateTimeは内部的にシリアル値を持っている
            // as_f64()メソッドがあればそれを使う、なければto_string()してパース
            let dt_str = dt.to_string();
            if let Ok(serial) = dt_str.parse::<f64>() {
                excel_serial_to_date(serial)
            } else {
                // パースできない場合は日付文字列として扱う
                dt_str
                    .split_whitespace()
                    .next()
                    .unwrap_or(&dt_str)
                    .to_string()
            }
        }
        Some(Data::DateTimeIso(dt_str)) => dt_str.split('T').next().unwrap_or(dt_str).to_string(),
        Some(Data::Float(f)) => excel_serial_to_date(*f),
        Some(Data::Int(i)) => excel_serial_to_date(*i as f64),
        Some(Data::String(s)) => {
            if let Ok(serial) = s.parse::<f64>() {
                excel_serial_to_date(serial)
            } else {
                s.trim().to_string()
            }
        }
        Some(other) => {
            let s = other.to_string().trim().to_string();
            if let Ok(serial) = s.parse::<f64>() {
                excel_serial_to_date(serial)
            } else {
                s
            }
        }
        None => String::new(),
    }
}

/// Excelシリアル値を日付文字列に変換
fn excel_serial_to_date(serial: f64) -> String {
    // Excelの日付シリアル値は1900年1月1日を1とする
    // ただし、Excelには1900年がうるう年というバグがあるため調整が必要
    let days = if serial > 59.0 {
        serial - 2.0 // 1900年のうるう年バグを補正
    } else {
        serial - 1.0
    };

    // 1899年12月30日を基準日とする
    let base_date = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    let target_date = base_date + chrono::Duration::days(days as i64);

    format!(
        "{:04}-{:02}-{:02}",
        target_date.year(),
        target_date.month(),
        target_date.day()
    )
}

/// Excelベースの配合マスタリポジトリ
pub struct ExcelFormulaRepository {
    data: HashMap<String, Vec<FormulaEntry>>,
}

impl ExcelFormulaRepository {
    pub fn new(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Self> {
        let sheet_name = "配合マスタ";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        if rows.is_empty() {
            return Err(eyre!("配合マスタシートが空です"));
        }

        let header_row = rows[0];
        let col_product_code = find_column_index(header_row, "製造商品コード", sheet_name)?;
        let col_material_code = find_column_index(header_row, "材料商品コード", sheet_name)?;
        let col_consumption_ratio = find_column_index(header_row, "消費比率", sheet_name)?;

        let mut data: HashMap<String, Vec<FormulaEntry>> = HashMap::new();

        for row in rows.iter().skip(1) {
            let product_code_str = get_cell_string(row, col_product_code);
            let material_code_str = get_cell_string(row, col_material_code);
            let consumption_ratio_str = get_cell_string(row, col_consumption_ratio);

            if product_code_str.is_empty()
                || material_code_str.is_empty()
                || consumption_ratio_str.is_empty()
            {
                continue;
            }

            let material_code = ProductCode::new(material_code_str)?;
            let consumption_ratio = ConsumptionRatio::new(consumption_ratio_str.parse()?)?;

            let entry = FormulaEntry::new(material_code, consumption_ratio);

            data.entry(product_code_str).or_default().push(entry);
        }

        Ok(Self { data })
    }
}

impl FormulaRepository for ExcelFormulaRepository {
    fn find_by_product_code(&self, product_code: &ProductCode) -> Result<Vec<FormulaEntry>> {
        self.data.get(product_code.value()).cloned().ok_or_else(|| {
            eyre!(
                "配合マスタに商品コード '{}' が見つかりません",
                product_code.value()
            )
        })
    }
}

/// Excelベースの運賃マスタリポジトリ
pub struct ExcelFreightMasterRepository {
    data: HashMap<String, FreightMaster>,
}

impl ExcelFreightMasterRepository {
    pub fn new(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Self> {
        let sheet_name = "運賃マスタ";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        if rows.is_empty() {
            return Err(eyre!("運賃マスタシートが空です"));
        }

        let header_row = rows[0];
        let col_freight_code = find_column_index(header_row, "運賃コード", sheet_name)?;
        let col_pattern_name = find_column_index(header_row, "パターン名", sheet_name)?;
        let col_kg_unit_price = find_column_index(header_row, "Kg単価", sheet_name)?;
        let col_valid_from = find_column_index(header_row, "有効開始日", sheet_name)?;
        let col_valid_to = find_column_index(header_row, "有効終了日", sheet_name)?;

        let mut data: HashMap<String, FreightMaster> = HashMap::new();

        for (row_idx, row) in rows.iter().enumerate().skip(1) {
            let freight_code_str = get_cell_string(row, col_freight_code);
            let pattern_name_str = get_cell_string(row, col_pattern_name);
            let kg_unit_price_str = get_cell_string(row, col_kg_unit_price);
            let valid_from_str = get_cell_date_string(row, col_valid_from);
            let valid_to_str = get_cell_date_string(row, col_valid_to);

            if freight_code_str.is_empty()
                || pattern_name_str.is_empty()
                || kg_unit_price_str.is_empty()
                || valid_from_str.is_empty()
            {
                continue;
            }

            // パース処理のみ（バリデーションはドメイン層で実施）
            let pattern_name = PatternName::new(pattern_name_str.clone())
                .map_err(|e| eyre!("運賃マスタ {}行目: {}", row_idx + 1, e))?;

            let kg_unit_price: f64 = kg_unit_price_str.parse().map_err(|_| {
                eyre!(
                    "運賃マスタ {}行目: Kg単価が数値ではありません: '{}'",
                    row_idx + 1,
                    kg_unit_price_str
                )
            })?;

            let valid_from = TransactionDate::new(valid_from_str)
                .map_err(|e| eyre!("運賃マスタ {}行目: {}", row_idx + 1, e))?;

            let valid_to = if valid_to_str.is_empty() {
                None
            } else {
                Some(
                    TransactionDate::new(valid_to_str)
                        .map_err(|e| eyre!("運賃マスタ {}行目: {}", row_idx + 1, e))?,
                )
            };

            let freight_master = FreightMaster::new(
                freight_code_str.clone(),
                pattern_name,
                Amount::new(kg_unit_price)?,
                valid_from,
                valid_to,
            )
            .map_err(|e| eyre!("運賃マスタ {}行目: {}", row_idx + 1, e))?;

            data.insert(freight_code_str, freight_master);
        }

        Ok(Self { data })
    }
}

impl FreightMasterRepository for ExcelFreightMasterRepository {
    fn find_by_code(&self, freight_code: &str) -> Result<FreightMaster> {
        self.data
            .get(freight_code)
            .cloned()
            .ok_or_else(|| eyre!("運賃マスタに運賃コード '{}' が見つかりません", freight_code))
    }
}

/// Excelベースの仕入リポジトリ
pub struct ExcelPurchaseRepository {
    data: HashMap<String, Purchase>,
}

impl ExcelPurchaseRepository {
    pub fn new(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Self> {
        let sheet_name = "【入庫】仕入";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        if rows.is_empty() {
            return Err(eyre!("【入庫】仕入シートが空です"));
        }

        let headers: Vec<String> = rows[0]
            .iter()
            .map(|cell| cell.to_string().trim().to_string())
            .collect();

        let schema = PurchaseSheetSchema::from_headers(&headers)?;

        let mut data: HashMap<String, Purchase> = HashMap::new();

        for (row_idx, row) in rows.iter().enumerate().skip(1) {
            let product_code_str = get_cell_string(row, schema.product_code().value());
            let product_name = get_cell_string(row, schema.product_name().value());
            let unit_price_str = get_cell_string(row, schema.unit_price().value());
            let quantity_str = get_cell_string(row, schema.quantity().value());
            let freight_str = get_cell_string(row, schema.freight().value());

            if product_code_str.is_empty() || unit_price_str.is_empty() {
                continue;
            }

            let unit_price = Amount::new(unit_price_str.parse().map_err(|_| {
                eyre!(
                    "【入庫】仕入シート {}行目: 仕入単価が数値ではありません: '{}'",
                    row_idx + 1,
                    unit_price_str
                )
            })?)?;

            let quantity: f64 = if quantity_str.is_empty() {
                0.0
            } else {
                quantity_str.parse().map_err(|_| {
                    eyre!(
                        "【入庫】仕入シート {}行目: 数量が数値ではありません: '{}'",
                        row_idx + 1,
                        quantity_str
                    )
                })?
            };

            let freight_code = if freight_str.is_empty() {
                FreightCode::DirectPrice(0.0)
            } else {
                FreightCode::new(freight_str.clone())
                    .map_err(|e| eyre!("【入庫】仕入シート {}行目: {}", row_idx + 1, e))?
            };

            let purchase = Purchase::new(
                product_name,
                unit_price,
                Quantity::new(quantity)?,
                freight_code,
            );

            data.insert(product_code_str, purchase);
        }

        Ok(Self { data })
    }
}

impl PurchaseRepository for ExcelPurchaseRepository {
    fn find_latest_price(&self, product_code: &ProductCode) -> Result<Purchase> {
        self.data.get(product_code.value()).cloned().ok_or_else(|| {
            eyre!(
                "仕入データに商品コード '{}' が見つかりません",
                product_code.value()
            )
        })
    }
}

/// Excel入出庫トランザクションリポジトリ
pub struct ExcelInventoryTransactionRepository {
    transactions: Vec<InventoryTransaction>,
}

impl ExcelInventoryTransactionRepository {
    pub fn new(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Self> {
        let mut transactions = Vec::new();

        // 【入庫】生産シートから読み込み
        if let Ok(range) = workbook.worksheet_range("【入庫】生産") {
            let rows: Vec<_> = range.rows().collect();
            if !rows.is_empty() {
                let headers: Vec<String> = rows[0]
                    .iter()
                    .map(|cell| cell.to_string().trim().to_string())
                    .collect();

                let schema = ProductionSheetSchema::from_headers(&headers)?;

                for (row_idx, row) in rows.iter().enumerate().skip(1) {
                    let date_str = get_cell_date_string(row, schema.production_date().value());
                    let product_code_str = get_cell_string(row, schema.product_code().value());
                    let quantity_str = get_cell_string(row, schema.quantity().value());

                    if !date_str.is_empty()
                        && !product_code_str.is_empty()
                        && !quantity_str.is_empty()
                    {
                        let quantity = quantity_str.parse::<f64>().map_err(|_| {
                            eyre!(
                                "【入庫】生産シート {}行目: 生産数量が数値ではありません: '{}'",
                                row_idx + 1,
                                quantity_str
                            )
                        })?;

                        let transaction_date = TransactionDate::new(date_str.clone())
                            .map_err(|e| eyre!("【入庫】生産シート {}行目: {}", row_idx + 1, e))?;

                        transactions.push(InventoryTransaction::new(
                            transaction_date,
                            InventoryType::Production,
                            ProductCode::new(product_code_str.clone())?,
                            product_code_str,
                            Quantity::new(quantity)?,
                        ));
                    }
                }
            }
        }

        // 【入庫】仕入シートから読み込み
        if let Ok(range) = workbook.worksheet_range("【入庫】仕入") {
            let rows: Vec<_> = range.rows().collect();
            if !rows.is_empty() {
                let headers: Vec<String> = rows[0]
                    .iter()
                    .map(|cell| cell.to_string().trim().to_string())
                    .collect();

                let schema = PurchaseSheetSchema::from_headers(&headers)?;

                for (row_idx, row) in rows.iter().enumerate().skip(1) {
                    let date_str = get_cell_date_string(row, schema.purchase_date().value());
                    let product_code_str = get_cell_string(row, schema.product_code().value());
                    let product_name = get_cell_string(row, schema.product_name().value());
                    let quantity_str = get_cell_string(row, schema.quantity().value());

                    if !date_str.is_empty()
                        && !product_code_str.is_empty()
                        && !quantity_str.is_empty()
                    {
                        let quantity = quantity_str.parse::<f64>().map_err(|_| {
                            eyre!(
                                "【入庫】仕入シート {}行目: 数量が数値ではありません: '{}'",
                                row_idx + 1,
                                quantity_str
                            )
                        })?;

                        let transaction_date = TransactionDate::new(date_str.clone())
                            .map_err(|e| eyre!("【入庫】仕入シート {}行目: {}", row_idx + 1, e))?;

                        transactions.push(InventoryTransaction::new(
                            transaction_date,
                            InventoryType::Purchase,
                            ProductCode::new(product_code_str)?,
                            product_name,
                            Quantity::new(quantity)?,
                        ));
                    }
                }
            }
        }

        // 【出庫】売上シートから読み込み
        if let Ok(range) = workbook.worksheet_range("【出庫】売上") {
            let rows: Vec<_> = range.rows().collect();
            if !rows.is_empty() {
                let headers: Vec<String> = rows[0]
                    .iter()
                    .map(|cell| cell.to_string().trim().to_string())
                    .collect();

                let schema = SalesSheetSchema::from_headers(&headers)?;

                for (row_idx, row) in rows.iter().enumerate().skip(1) {
                    let date_str = get_cell_date_string(row, schema.sales_date().value());
                    let product_code_str = get_cell_string(row, schema.product_code().value());
                    let product_name = get_cell_string(row, schema.product_name().value());
                    let quantity_str = get_cell_string(row, schema.quantity().value());

                    if !date_str.is_empty()
                        && !product_code_str.is_empty()
                        && !quantity_str.is_empty()
                    {
                        let quantity = quantity_str.parse::<f64>().map_err(|_| {
                            eyre!(
                                "【出庫】売上シート {}行目: 数量が数値ではありません: '{}'",
                                row_idx + 1,
                                quantity_str
                            )
                        })?;

                        let transaction_date = TransactionDate::new(date_str.clone())
                            .map_err(|e| eyre!("【出庫】売上シート {}行目: {}", row_idx + 1, e))?;

                        transactions.push(InventoryTransaction::new(
                            transaction_date,
                            InventoryType::Sales,
                            ProductCode::new(product_code_str)?,
                            product_name,
                            Quantity::new(quantity)?,
                        ));
                    }
                }
            }
        }

        Ok(Self { transactions })
    }
}

impl InventoryTransactionRepository for ExcelInventoryTransactionRepository {
    fn find_all_transactions(&self) -> Result<Vec<InventoryTransaction>> {
        Ok(self.transactions.clone())
    }
}

/// Excel生産データリポジトリ
pub struct ExcelProductionRepository {
    productions: Vec<Production>,
}

impl ExcelProductionRepository {
    pub fn new(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Self> {
        let sheet_name = "【入庫】生産";
        let range = workbook.worksheet_range(sheet_name)?;
        let rows: Vec<_> = range.rows().collect();

        if rows.is_empty() {
            return Err(eyre!("【入庫】生産シートが空です"));
        }

        let headers: Vec<String> = rows[0]
            .iter()
            .map(|cell| cell.to_string().trim().to_string())
            .collect();

        let schema = ProductionSheetSchema::from_headers(&headers)?;

        let mut productions = Vec::new();

        for (row_idx, row) in rows.iter().enumerate().skip(1) {
            let production_date = get_cell_date_string(row, schema.production_date().value());
            let product_code_str = get_cell_string(row, schema.product_code().value());
            let quantity_str = get_cell_string(row, schema.quantity().value());
            let yield_rate_str = get_cell_string(row, schema.yield_rate().value());
            let coagulant_str = get_cell_string(row, schema.coagulant().value());
            let clay_treatment_str = get_cell_string(row, schema.clay_treatment().value());

            // 必須項目チェック
            if production_date.is_empty()
                || product_code_str.is_empty()
                || quantity_str.is_empty()
                || yield_rate_str.is_empty()
            {
                return Err(eyre!(
                    "【入庫】生産シートの{}行目に必須データが欠けています\n  生産日: {}\n  商品コード: {}\n  生産数量: {}\n  歩留率: {}",
                    row_idx + 1,
                    if production_date.is_empty() {
                        "空白"
                    } else {
                        &production_date
                    },
                    if product_code_str.is_empty() {
                        "空白"
                    } else {
                        &product_code_str
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
                    "【入庫】生産シート {}行目: 生産数量が数値ではありません: '{}'",
                    row_idx + 1,
                    quantity_str
                )
            })?;

            let yield_rate: f64 = yield_rate_str.parse().map_err(|_| {
                eyre!(
                    "【入庫】生産シート {}行目: 歩留率が数値ではありません: '{}'",
                    row_idx + 1,
                    yield_rate_str
                )
            })?;

            let coagulant_cost: f64 = if coagulant_str.is_empty() {
                0.0
            } else {
                coagulant_str.parse().map_err(|_| {
                    eyre!(
                        "【入庫】生産シート {}行目: 凝集剤が数値ではありません: '{}'",
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
                        "【入庫】生産シート {}行目: 粘土処理が数値ではありません: '{}'",
                        row_idx + 1,
                        clay_treatment_str
                    )
                })?
            };

            productions.push(Production::new(
                ProductCode::new(product_code_str)?,
                Quantity::new(quantity)?,
                YieldRate::new(yield_rate)?,
                Amount::new(coagulant_cost)?,
                Amount::new(clay_treatment_cost)?,
            ));
        }

        Ok(Self { productions })
    }
}

impl ProductionRepository for ExcelProductionRepository {
    fn find_all(&self) -> Result<Vec<Production>> {
        Ok(self.productions.clone())
    }
}

/// Excelリポジトリファクトリ
pub struct ExcelRepositoryFactory {
    pub formula_repo: ExcelFormulaRepository,
    pub freight_repo: ExcelFreightMasterRepository,
    pub purchase_repo: ExcelPurchaseRepository,
    pub production_repo: ExcelProductionRepository,
    pub transaction_repo: ExcelInventoryTransactionRepository,
}

impl ExcelRepositoryFactory {
    /// Excelファイルからすべてのリポジトリを初期化
    pub fn from_file(file_path: &str) -> Result<Self> {
        use calamine::{Reader, Xlsx, open_workbook};

        println!("Excelファイルを読み取り中: {}", file_path);
        let mut workbook = open_workbook::<Xlsx<_>, _>(file_path).map_err(|e| {
            eyre!(
                "入力ファイルを開けませんでした\n\
                ファイル: {}\n\
                原因: {}\n\n\
                対処方法:\n\
                  - ファイルがExcelなどで開かれている場合は閉じてください\n\
                  - ファイルパスが正しいか確認してください",
                file_path,
                e
            )
        })?;

        // シート名を表示
        let sheet_names = workbook.sheet_names().to_owned();
        println!("\n既存のシート構成:");
        for (i, name) in sheet_names.iter().enumerate() {
            println!("  {}. {}", i + 1, name);
        }

        // リポジトリを初期化
        println!("\nリポジトリを初期化中...");
        let formula_repo = ExcelFormulaRepository::new(&mut workbook)?;
        let freight_repo = ExcelFreightMasterRepository::new(&mut workbook)?;
        let purchase_repo = ExcelPurchaseRepository::new(&mut workbook)?;
        let production_repo = ExcelProductionRepository::new(&mut workbook)?;
        let transaction_repo = ExcelInventoryTransactionRepository::new(&mut workbook)?;
        println!("  ✓ リポジトリの初期化完了");

        Ok(Self {
            formula_repo,
            freight_repo,
            purchase_repo,
            production_repo,
            transaction_repo,
        })
    }
}

use crate::domain::entities::*;
use crate::domain::repositories::*;
use crate::domain::value_objects::*;
use calamine::{Data, Reader, Xlsx};
use color_eyre::{Result, eyre::eyre};
use std::collections::HashMap;

// 共通ヘルパー関数
fn find_column_index(header_row: &[Data], column_name: &str) -> Result<usize> {
    header_row
        .iter()
        .position(|cell| cell.to_string().trim() == column_name)
        .ok_or_else(|| eyre!("列 '{}' が見つかりません", column_name))
}

fn get_cell_string(row: &[Data], index: usize) -> String {
    row.get(index)
        .map(|c| c.to_string().trim().to_string())
        .unwrap_or_default()
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
        let col_product_code = find_column_index(header_row, "製造商品コード")?;
        let col_material_code = find_column_index(header_row, "材料商品コード")?;
        let col_consumption_ratio = find_column_index(header_row, "消費比率")?;

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

            let product_code = ProductCode::new(product_code_str.clone())?;
            let material_code = ProductCode::new(material_code_str)?;
            let consumption_ratio = ConsumptionRatio::new(consumption_ratio_str.parse()?)?;

            let entry = FormulaEntry::new(product_code.clone(), material_code, consumption_ratio);

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

        let header_row = rows[0];
        let col_product_code = find_column_index(header_row, "商品コード")?;
        let col_product_name = find_column_index(header_row, "商品")?;
        let col_unit_price = find_column_index(header_row, "仕入単価")?;

        let mut data: HashMap<String, Purchase> = HashMap::new();

        for row in rows.iter().skip(1) {
            let product_code_str = get_cell_string(row, col_product_code);
            let product_name = get_cell_string(row, col_product_name);
            let unit_price_str = get_cell_string(row, col_unit_price);

            if product_code_str.is_empty() || unit_price_str.is_empty() {
                continue;
            }

            let product_code = ProductCode::new(product_code_str.clone())?;
            let unit_price = Amount::new(unit_price_str.parse()?)?;

            let purchase = Purchase::new(product_code, product_name, unit_price);

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

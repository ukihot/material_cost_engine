use color_eyre::{Result, eyre::eyre};
use std::collections::HashMap;

/// 列位置を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnIndex(usize);

impl ColumnIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

/// 【入庫】生産シートのスキーマ
#[derive(Debug, Clone)]
pub struct ProductionSheetSchema {
    col_production_date: ColumnIndex,
    col_product_code: ColumnIndex,
    col_quantity: ColumnIndex,
    col_yield_rate: ColumnIndex,
    col_coagulant: ColumnIndex,
    col_clay_treatment: ColumnIndex,
    col_freight: ColumnIndex,
}

impl ProductionSheetSchema {
    pub fn from_headers(headers: &[String]) -> Result<Self> {
        let mut header_map: HashMap<&str, usize> = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            let trimmed = header.trim();
            if !trimmed.is_empty() {
                header_map.insert(trimmed, idx);
            }
        }

        let required_headers = [
            "生産日",
            "商品コード",
            "生産品番",
            "生産数量",
            "歩留率",
            "凝集剤",
            "粘土処理",
            "材料運賃",
        ];

        let mut missing = Vec::new();
        for &header in &required_headers {
            if !header_map.contains_key(header) {
                missing.push(header);
            }
        }

        if !missing.is_empty() {
            return Err(eyre!(
                "【入庫】生産シートに必須カラムが見つかりません: {:?}",
                missing
            ));
        }

        Ok(Self {
            col_production_date: ColumnIndex::new(*header_map.get("生産日").unwrap()),
            col_product_code: ColumnIndex::new(*header_map.get("商品コード").unwrap()),
            col_quantity: ColumnIndex::new(*header_map.get("生産数量").unwrap()),
            col_yield_rate: ColumnIndex::new(*header_map.get("歩留率").unwrap()),
            col_coagulant: ColumnIndex::new(*header_map.get("凝集剤").unwrap()),
            col_clay_treatment: ColumnIndex::new(*header_map.get("粘土処理").unwrap()),
            col_freight: ColumnIndex::new(*header_map.get("材料運賃").unwrap()),
        })
    }

    pub fn production_date(&self) -> ColumnIndex {
        self.col_production_date
    }

    pub fn product_code(&self) -> ColumnIndex {
        self.col_product_code
    }

    pub fn quantity(&self) -> ColumnIndex {
        self.col_quantity
    }

    pub fn yield_rate(&self) -> ColumnIndex {
        self.col_yield_rate
    }

    pub fn coagulant(&self) -> ColumnIndex {
        self.col_coagulant
    }

    pub fn clay_treatment(&self) -> ColumnIndex {
        self.col_clay_treatment
    }

    pub fn freight(&self) -> ColumnIndex {
        self.col_freight
    }
}

/// 【入庫】仕入シートのスキーマ
#[derive(Debug, Clone)]
pub struct PurchaseSheetSchema {
    col_purchase_date: ColumnIndex,
    col_product_code: ColumnIndex,
    col_product_name: ColumnIndex,
    col_unit_price: ColumnIndex,
    col_quantity: ColumnIndex,
    col_freight: ColumnIndex,
}

impl PurchaseSheetSchema {
    pub fn from_headers(headers: &[String]) -> Result<Self> {
        let mut header_map: HashMap<&str, usize> = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            let trimmed = header.trim();
            if !trimmed.is_empty() {
                header_map.insert(trimmed, idx);
            }
        }

        let required_headers = ["仕入日", "商品コード", "商品", "仕入単価", "数量", "運賃"];

        let mut missing = Vec::new();
        for &header in &required_headers {
            if !header_map.contains_key(header) {
                missing.push(header);
            }
        }

        if !missing.is_empty() {
            return Err(eyre!(
                "【入庫】仕入シートに必須カラムが見つかりません: {:?}",
                missing
            ));
        }

        Ok(Self {
            col_purchase_date: ColumnIndex::new(*header_map.get("仕入日").unwrap()),
            col_product_code: ColumnIndex::new(*header_map.get("商品コード").unwrap()),
            col_product_name: ColumnIndex::new(*header_map.get("商品").unwrap()),
            col_unit_price: ColumnIndex::new(*header_map.get("仕入単価").unwrap()),
            col_quantity: ColumnIndex::new(*header_map.get("数量").unwrap()),
            col_freight: ColumnIndex::new(*header_map.get("運賃").unwrap()),
        })
    }

    pub fn purchase_date(&self) -> ColumnIndex {
        self.col_purchase_date
    }

    pub fn product_code(&self) -> ColumnIndex {
        self.col_product_code
    }

    pub fn product_name(&self) -> ColumnIndex {
        self.col_product_name
    }

    pub fn unit_price(&self) -> ColumnIndex {
        self.col_unit_price
    }

    pub fn quantity(&self) -> ColumnIndex {
        self.col_quantity
    }

    pub fn freight(&self) -> ColumnIndex {
        self.col_freight
    }
}

/// 【出庫】売上シートのスキーマ
#[derive(Debug, Clone)]
pub struct SalesSheetSchema {
    col_sales_date: ColumnIndex,
    col_product_code: ColumnIndex,
    col_product_name: ColumnIndex,
    col_quantity: ColumnIndex,
}

impl SalesSheetSchema {
    pub fn from_headers(headers: &[String]) -> Result<Self> {
        let mut header_map: HashMap<&str, usize> = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            let trimmed = header.trim();
            if !trimmed.is_empty() {
                header_map.insert(trimmed, idx);
            }
        }

        let required_headers = ["売上日", "商品コード", "商品名", "数量"];

        let mut missing = Vec::new();
        for &header in &required_headers {
            if !header_map.contains_key(header) {
                missing.push(header);
            }
        }

        if !missing.is_empty() {
            return Err(eyre!(
                "【出庫】売上シートに必須カラムが見つかりません: {:?}",
                missing
            ));
        }

        Ok(Self {
            col_sales_date: ColumnIndex::new(*header_map.get("売上日").unwrap()),
            col_product_code: ColumnIndex::new(*header_map.get("商品コード").unwrap()),
            col_product_name: ColumnIndex::new(*header_map.get("商品名").unwrap()),
            col_quantity: ColumnIndex::new(*header_map.get("数量").unwrap()),
        })
    }

    pub fn sales_date(&self) -> ColumnIndex {
        self.col_sales_date
    }

    pub fn product_code(&self) -> ColumnIndex {
        self.col_product_code
    }

    pub fn product_name(&self) -> ColumnIndex {
        self.col_product_name
    }

    pub fn quantity(&self) -> ColumnIndex {
        self.col_quantity
    }
}

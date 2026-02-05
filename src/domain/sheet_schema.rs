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

    pub fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

/// 【入庫】生産シートのスキーマ
#[derive(Debug, Clone)]
pub struct ProductionSheetSchema {
    col_item: ColumnIndex,
    col_product_code: ColumnIndex,
    col_production_number: ColumnIndex,
    col_quantity: ColumnIndex,
    col_raw_material_cost: ColumnIndex,
    col_unit_cost: ColumnIndex,
    col_yield_rate: ColumnIndex,
    col_yield_cost: ColumnIndex,
    col_coagulant: ColumnIndex,
    col_clay_treatment: ColumnIndex,
    col_material_cost: ColumnIndex,
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
            "項",
            "商品コード",
            "生産品番",
            "生産数量",
            "原砂金額",
            "原単位（円/ｔ）",
            "歩留率",
            "原砂歩留金額",
            "凝集剤",
            "粘土処理",
            "材料費",
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
            col_item: ColumnIndex::new(*header_map.get("項").unwrap()),
            col_product_code: ColumnIndex::new(*header_map.get("商品コード").unwrap()),
            col_production_number: ColumnIndex::new(*header_map.get("生産品番").unwrap()),
            col_quantity: ColumnIndex::new(*header_map.get("生産数量").unwrap()),
            col_raw_material_cost: ColumnIndex::new(*header_map.get("原砂金額").unwrap()),
            col_unit_cost: ColumnIndex::new(*header_map.get("原単位（円/ｔ）").unwrap()),
            col_yield_rate: ColumnIndex::new(*header_map.get("歩留率").unwrap()),
            col_yield_cost: ColumnIndex::new(*header_map.get("原砂歩留金額").unwrap()),
            col_coagulant: ColumnIndex::new(*header_map.get("凝集剤").unwrap()),
            col_clay_treatment: ColumnIndex::new(*header_map.get("粘土処理").unwrap()),
            col_material_cost: ColumnIndex::new(*header_map.get("材料費").unwrap()),
        })
    }

    pub fn item(&self) -> ColumnIndex {
        self.col_item
    }

    pub fn product_code(&self) -> ColumnIndex {
        self.col_product_code
    }

    pub fn production_number(&self) -> ColumnIndex {
        self.col_production_number
    }

    pub fn quantity(&self) -> ColumnIndex {
        self.col_quantity
    }

    pub fn raw_material_cost(&self) -> ColumnIndex {
        self.col_raw_material_cost
    }

    pub fn unit_cost(&self) -> ColumnIndex {
        self.col_unit_cost
    }

    pub fn yield_rate(&self) -> ColumnIndex {
        self.col_yield_rate
    }

    pub fn yield_cost(&self) -> ColumnIndex {
        self.col_yield_cost
    }

    pub fn coagulant(&self) -> ColumnIndex {
        self.col_coagulant
    }

    pub fn clay_treatment(&self) -> ColumnIndex {
        self.col_clay_treatment
    }

    pub fn material_cost(&self) -> ColumnIndex {
        self.col_material_cost
    }
}

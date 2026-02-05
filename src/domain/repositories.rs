use super::entities::*;
use super::value_objects::*;
use color_eyre::Result;

/// 配合マスタリポジトリ
pub trait FormulaRepository {
    fn find_by_product_code(&self, product_code: &ProductCode) -> Result<Vec<FormulaEntry>>;
}

/// 仕入リポジトリ
pub trait PurchaseRepository {
    fn find_latest_price(&self, product_code: &ProductCode) -> Result<Purchase>;
}

/// 生産リポジトリ
pub trait ProductionRepository {
    fn find_all(&self) -> Result<Vec<Production>>;
}

/// 入出庫トランザクションリポジトリ
pub trait InventoryTransactionRepository {
    fn find_all_transactions(&self) -> Result<Vec<InventoryTransaction>>;
}

/// 運賃マスタリポジトリ
pub trait FreightMasterRepository {
    fn find_by_code(&self, freight_code: &str) -> Result<FreightMaster>;
}

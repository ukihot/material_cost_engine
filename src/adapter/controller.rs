use crate::domain::repositories::*;
use crate::usecase::interactor::{
    CalculateMaterialCostInteractor, CreateInventoryHistoryInteractor,
};
use crate::usecase::ports::*;
use color_eyre::Result;

/// Excelコントローラ
pub struct ExcelController<'a, F, P, FR, R, T, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    FR: FreightMasterRepository,
    R: ProductionRepository,
    T: InventoryTransactionRepository,
    O: CalculateMaterialCostOutputPort + CreateInventoryHistoryOutputPort,
{
    formula_repo: &'a F,
    purchase_repo: &'a P,
    freight_repo: &'a FR,
    production_repo: &'a R,
    transaction_repo: &'a T,
    output_port: &'a mut O,
}

impl<'a, F, P, FR, R, T, O> ExcelController<'a, F, P, FR, R, T, O>
where
    F: FormulaRepository,
    P: PurchaseRepository,
    FR: FreightMasterRepository,
    R: ProductionRepository,
    T: InventoryTransactionRepository,
    O: CalculateMaterialCostOutputPort + CreateInventoryHistoryOutputPort,
{
    pub fn new(
        formula_repo: &'a F,
        purchase_repo: &'a P,
        freight_repo: &'a FR,
        production_repo: &'a R,
        transaction_repo: &'a T,
        output_port: &'a mut O,
    ) -> Self {
        Self {
            formula_repo,
            purchase_repo,
            freight_repo,
            production_repo,
            transaction_repo,
            output_port,
        }
    }

    /// 材料費計算を実行
    pub fn execute_material_cost_calculation(&mut self) -> Result<()> {
        let mut interactor = CalculateMaterialCostInteractor::new(
            self.formula_repo,
            self.purchase_repo,
            self.freight_repo,
            self.production_repo,
            self.output_port,
        );
        interactor.execute()
    }

    /// 入出庫履歴作成を実行
    pub fn execute_inventory_history_creation(&mut self) -> Result<()> {
        let mut interactor =
            CreateInventoryHistoryInteractor::new(self.transaction_repo, self.output_port);
        interactor.execute()
    }
}

mod adapter;
mod config;
mod domain;
mod infrastructure;
mod usecase;

use adapter::controller::ExcelController;
use adapter::presenter::ExcelPresenter;
use color_eyre::Result;
use config::Config;
use infrastructure::excel_repositories::ExcelRepositoryFactory;
use std::io::{self, Write};

fn main() {
    // エラーハンドリングを初期化
    if let Err(e) = color_eyre::install() {
        eprintln!("エラーハンドリングの初期化に失敗: {}", e);
    }

    // エラーが発生しても必ず入力待ちをする
    if let Err(e) = run() {
        eprintln!("\n❌ エラーが発生しました:");
        eprintln!("{:?}", e);
    }

    // 結果に関わらず入力待ち
    let _ = wait_for_enter();
}

fn run() -> Result<()> {
    // 設定を読み込む
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n❌ 設定ファイルの読み込みエラー");
            eprintln!("{}", e);
            eprintln!("\n対処方法:");
            eprintln!("  1. 実行ファイル(.exe)と同じフォルダに config.toml を配置してください");
            eprintln!("  2. config.toml の内容例:");
            eprintln!("     [paths]");
            eprintln!("     input_file = \"tests/直接材料費原価計算表.xlsx\"");
            eprintln!("     output_file = \"tests/直接材料費原価計算表_結果.xlsx\"");
            return Err(e);
        }
    };

    let input_path = &config.paths.input_file;
    let output_path = &config.paths.output_file;

    // Excelファイルを読み取り、リポジトリを初期化
    let factory = ExcelRepositoryFactory::from_file(input_path)?;

    // プレゼンターを初期化
    let mut presenter = ExcelPresenter::new(input_path.clone(), output_path.clone())?;

    // コントローラを組み立てる
    let mut controller = ExcelController::new(
        &factory.formula_repo,
        &factory.purchase_repo,
        &factory.freight_repo,
        &factory.production_repo,
        &factory.transaction_repo,
        &mut presenter,
    );

    // ユースケース1: 材料費計算
    controller.execute_material_cost_calculation()?;

    // ユースケース2: 入出庫履歴作成
    controller.execute_inventory_history_creation()?;

    // 結果を保存
    presenter.finalize()?;

    Ok(())
}

fn wait_for_enter() -> Result<()> {
    println!("\nEnterキーを押して終了...");
    if let Err(e) = io::stdout().flush() {
        eprintln!("出力のフラッシュに失敗: {}", e);
    }
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("入力の読み取りに失敗: {}", e);
    }
    Ok(())
}

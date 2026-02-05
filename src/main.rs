mod adapter;
mod domain;
mod usecase;

use adapter::controller::ExcelController;
use adapter::presenter::ExcelPresenter;
use adapter::repositories::{ExcelFormulaRepository, ExcelPurchaseRepository};
use calamine::{Reader, Xlsx, open_workbook};
use color_eyre::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    color_eyre::install()?;

    let input_path = "tests/直接材料費原価計算表.xlsx";
    let output_path = "tests/直接材料費原価計算表_結果.xlsx";

    // Excelファイルを読み取る
    println!("Excelファイルを読み取り中: {}", input_path);
    let mut workbook = match open_workbook::<Xlsx<_>, _>(input_path) {
        Ok(wb) => wb,
        Err(e) => {
            eprintln!("\nエラー: 入力ファイルを開けませんでした");
            eprintln!("ファイル: {}", input_path);
            eprintln!("原因: {}", e);
            eprintln!("\n対処方法:");
            eprintln!("  - ファイルがExcelなどで開かれている場合は閉じてください");
            eprintln!("  - ファイルパスが正しいか確認してください");
            wait_for_enter()?;
            return Ok(());
        }
    };

    // シート名を表示
    let sheet_names = workbook.sheet_names().to_owned();
    println!("\n既存のシート構成:");
    for (i, name) in sheet_names.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    // リポジトリを初期化
    println!("\nリポジトリを初期化中...");
    let formula_repo = match ExcelFormulaRepository::new(&mut workbook) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("\n❌ 配合マスタの読み込みエラー:");
            eprintln!("{:?}", e);
            wait_for_enter()?;
            return Ok(());
        }
    };

    let purchase_repo = match ExcelPurchaseRepository::new(&mut workbook) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("\n❌ 仕入データの読み込みエラー:");
            eprintln!("{:?}", e);
            wait_for_enter()?;
            return Ok(());
        }
    };

    println!("  ✓ リポジトリの初期化完了");

    // プレゼンター、コントローラを組み立てる
    let presenter = ExcelPresenter::new();
    let mut controller = ExcelController::new(
        &formula_repo,
        &purchase_repo,
        presenter,
        input_path.to_string(),
        output_path.to_string(),
    );

    // コントローラを実行
    if let Err(e) = controller.execute(&mut workbook) {
        eprintln!("\n❌ 材料費計算エラー:");
        eprintln!("{:?}", e);

        wait_for_enter()?;
        return Ok(());
    }

    // 終了前に入力待ち
    wait_for_enter()?;

    Ok(())
}

fn wait_for_enter() -> Result<()> {
    println!("\nEnterキーを押して終了...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

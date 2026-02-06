fn main() {
    // Windowsプラットフォームの場合のみアイコンを設定
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // アイコンファイルを設定（存在する場合）
        if std::path::Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
        }

        // アプリケーション情報を設定
        res.set("ProductName", "Material Cost Engine");
        res.set("FileDescription", "材料費計算エンジン");
        res.set("CompanyName", "株式会社ツチヨシ産業");
        res.set(
            "LegalCopyright",
            "Copyright (c) 2026 Yu Tokunaga. All rights reserved.",
        );

        // リソースをコンパイル
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}

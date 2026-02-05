use color_eyre::{Result, eyre};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Paths,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub input_file: String,
    pub output_file: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config.toml";
        let config_str = fs::read_to_string(config_path).map_err(|e| {
            eyre::eyre!(
                "設定ファイル '{}' が見つかりません。\n\
                実行ファイルと同じディレクトリに config.toml を配置してください。\n\
                元のエラー: {}",
                config_path,
                e
            )
        })?;

        let config: Config = toml::from_str(&config_str).map_err(|e| {
            eyre::eyre!(
                "設定ファイル '{}' の解析に失敗しました。\n\
                フォーマットが正しいか確認してください。\n\
                元のエラー: {}",
                config_path,
                e
            )
        })?;

        Ok(config)
    }
}

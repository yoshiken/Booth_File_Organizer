use ::regex::Regex;
use anyhow::{anyhow, Result};
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
// Log imports will be added as needed in individual files

mod api_types;
pub mod booth_client;
mod booth_commands;
mod config;
mod database;
pub mod errors;
mod file_commands;
mod process_commands;
mod sync_commands;
mod system_commands;
mod tag_commands;
mod tag_validator;

use crate::config::{app, files, regex};
use booth_client::BoothClient;
use database::Database;
pub use errors::{AppError, AppResult};

// アプリケーション状態管理
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub booth_client: Arc<BoothClient>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let app_data_dir = dirs::data_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app::DATA_DIR_NAME);

        // ディレクトリが存在しない場合は作成
        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir)?;
        }

        let db_path = app_data_dir.join(app::DATABASE_FILENAME);
        let db = Database::new(&db_path.to_string_lossy())?;
        let booth_client = BoothClient::new();

        Ok(AppState {
            db: Arc::new(Mutex::new(db)),
            booth_client: Arc::new(booth_client),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub message: String,
    pub shop_name: Option<String>,
    pub product_name: Option<String>,
    pub files_extracted: Vec<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSelectResult {
    pub success: bool,
    pub files: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub total_files: usize,
    pub missing_files: Vec<MissingFile>,
    pub orphaned_files: usize,
    pub updated_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissingFile {
    pub id: i64,
    pub file_name: String,
    pub file_path: String,
    pub booth_shop_name: Option<String>,    // author_name を使用
    pub booth_product_name: Option<String>, // product_name を使用
}

// System/utility commands are now in system_commands.rs module

// Process/ZIP commands are now in process_commands.rs module

// File operation commands are now in file_commands.rs module

pub async fn process_zip_internal(
    zip_path: String,
    booth_url: Option<String>,
    output_dir: Option<String>,
    booth_client: &BoothClient,
) -> Result<ProcessResult> {
    let zip_path = PathBuf::from(&zip_path);

    // zip ファイルの存在確認
    if !zip_path.exists() {
        let path_display = zip_path.display();
        return Err(anyhow!(
            "ZIPファイルが見つかりません: {path_display}"
        ));
    }

    // BOOTH URLからショップ名と商品名を抽出（改善版）
    let (shop_name, product_name) = if let Some(url) = &booth_url {
        match extract_booth_info_with_api(url, booth_client).await {
            Ok((shop, product)) => (
                Some(sanitize_folder_name(&shop)),
                Some(sanitize_folder_name(&product)),
            ),
            Err(_) => {
                // フォールバック
                let file_stem = zip_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                (
                    Some("Unknown_Shop".to_string()),
                    Some(sanitize_folder_name(file_stem)),
                )
            }
        }
    } else {
        // URLが無い場合はファイル名から推測
        let file_stem = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        (
            Some("Unknown_Shop".to_string()),
            Some(sanitize_folder_name(file_stem)),
        )
    };

    // 出力ディレクトリの決定
    let output_base = if let Some(dir) = output_dir {
        PathBuf::from(dir)
    } else {
        // デフォルトでデスクトップのBOOTH_Organizedフォルダに出力
        dirs::desktop_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("BOOTH_Organized")
    };

    // ショップ名/商品名のフォルダ構造作成
    let final_output_dir = match (&shop_name, &product_name) {
        (Some(shop), Some(product)) => output_base.join(shop).join(product),
        (Some(shop), None) => output_base.join(shop),
        (None, Some(product)) => output_base.join("Unknown_Shop").join(product),
        (None, None) => output_base.join("Unknown"),
    };

    // ディレクトリ作成
    fs::create_dir_all(&final_output_dir)
        .map_err(|e| anyhow!("出力ディレクトリの作成に失敗: {}", e))?;

    // ZIP展開
    let extracted_files = extract_zip_with_encoding(&zip_path, &final_output_dir)?;

    Ok(ProcessResult {
        success: true,
        message: {
            let count = extracted_files.len();
            format!("{count}個のファイルを展開しました")
        },
        shop_name,
        product_name,
        files_extracted: extracted_files,
        output_path: Some(final_output_dir.to_string_lossy().to_string()),
    })
}

// 改善されたBOOTH情報取得（実際のAPIを使用）
pub async fn extract_booth_info_with_api(
    url: &str,
    booth_client: &BoothClient,
) -> Result<(String, String)> {
    // BOOTH APIを使って実際の商品情報を取得
    match booth_client.get_product_info(url).await {
        Ok(info) => Ok((info.shop_name, info.product_name)),
        Err(_) => {
            // フォールバック: URLから推測
            extract_booth_info_fallback(url)
        }
    }
}

pub fn extract_booth_info_fallback(url: &str) -> Result<(String, String)> {
    // BOOTH URLからショップ名を抽出（フォールバック）
    let regex = Regex::new(r"https://([^.]+)\.booth\.pm/items/(\d+)")
        .map_err(|e| anyhow!("正規表現の作成に失敗しました: {}", e))?;

    if let Some(captures) = regex.captures(url) {
        let shop_name = captures
            .get(regex::GROUP_1)
            .map(|m| m.as_str().to_string())
            .unwrap_or("unknown_shop".to_string());
        let product_id = captures
            .get(regex::GROUP_2)
            .ok_or_else(|| anyhow!("商品IDの抽出に失敗しました"))?
            .as_str();
        let product_name = format!("product_{product_id}");

        Ok((shop_name, product_name))
    } else {
        Err(anyhow!("無効なBOOTH URL: {}", url))
    }
}

// 安全なフォルダ名生成
pub fn sanitize_folder_name(name: &str) -> String {
    // Windows/Linux/macOSで使えない文字を置換
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = name.to_string();

    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }

    // 末尾のドットやスペースを削除
    sanitized = sanitized.trim_end_matches('.').trim().to_string();

    // 長すぎる名前は切り詰める（255文字制限）
    if sanitized.len() > files::MAX_FILENAME_LENGTH {
        sanitized.truncate(files::MAX_FILENAME_LENGTH);
    }

    // 空文字列の場合はデフォルト名
    if sanitized.is_empty() {
        sanitized = "unnamed".to_string();
    }

    sanitized
}

fn extract_zip_with_encoding(zip_path: &Path, output_dir: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;
    let mut extracted_files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // ファイル名のエンコーディング検出と変換
        let file_name = detect_and_convert_filename(file.name_raw())?;

        let output_path = output_dir.join(&file_name);

        if file.is_dir() {
            // ディレクトリの場合
            fs::create_dir_all(&output_path)?;
        } else {
            // ファイルの場合
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;

            extracted_files.push(file_name);
        }
    }

    Ok(extracted_files)
}

fn detect_and_convert_filename(raw_bytes: &[u8]) -> Result<String> {
    // まずUTF-8として試行
    if let Ok(utf8_str) = std::str::from_utf8(raw_bytes) {
        return Ok(utf8_str.to_string());
    }

    // Shift-JISとして解析を試行
    let (decoded, _encoding, had_errors) = SHIFT_JIS.decode(raw_bytes);
    if !had_errors {
        return Ok(decoded.to_string());
    }

    // CP932として試行（Windows日本語環境）
    if let Some(cp932) = encoding_rs::Encoding::for_label(b"cp932") {
        let (decoded_cp932, _encoding, had_errors) = cp932.decode(raw_bytes);
        if !had_errors {
            return Ok(decoded_cp932.to_string());
        }
    }

    // 最後の手段として、無効文字を置換してUTF-8に変換
    Ok(String::from_utf8_lossy(raw_bytes).to_string())
}

// BOOTH commands are now in booth_commands.rs module

// Sync/statistics commands are now in sync_commands.rs module

// Batch tag operations are now in tag_commands.rs module

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();

    // アプリケーション状態を初期化
    let app_state = AppState::new().expect("Failed to initialize application state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // 既存のコマンド（変更なし）
            system_commands::greet,
            process_commands::select_output_folder,
            process_commands::select_zip_files,
            process_commands::process_zip_file,
            file_commands::save_file_to_db,
            file_commands::get_all_files_from_db,
            file_commands::get_files_with_tags_from_db,
            tag_commands::add_tag_to_file_db,
            tag_commands::remove_tag_from_file_db,
            tag_commands::get_all_tags_from_db,
            tag_commands::get_tags_for_file_db,
            system_commands::save_output_folder,
            system_commands::load_output_folder,
            booth_commands::validate_booth_url,
            booth_commands::fetch_booth_product_info,
            system_commands::open_folder,
            booth_commands::download_booth_thumbnail,
            sync_commands::search_files_db,
            sync_commands::search_files_by_tags_db,
            sync_commands::find_duplicate_files_db,
            tag_commands::batch_add_tag_to_files_db,
            tag_commands::batch_remove_tag_from_files_db,
            file_commands::delete_file_db,
            file_commands::delete_file_and_folder,
            file_commands::batch_delete_files_db,
            file_commands::batch_update_files_db,
            sync_commands::get_batch_statistics_db,
            file_commands::get_files_with_tags_by_ids_db,
            booth_commands::update_file_booth_url_db,
            sync_commands::sync_file_system_db,
            sync_commands::remove_missing_files_db,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use anyhow::{anyhow, Result};
use encoding_rs::SHIFT_JIS;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use log::error;

use crate::booth_client::BoothClient;
use crate::database::DatabaseRefactored;
use crate::api_types::FileRecord;
use crate::commands::file_commands::ProcessResult;

/// Process a ZIP file with BOOTH information
pub async fn process_file(
    zip_path: &str,
    booth_url: Option<String>,
    output_dir: Option<String>,
    tags: Option<Vec<String>>,
    booth_client: Arc<BoothClient>,
    db: Arc<Mutex<DatabaseRefactored>>,
) -> Result<ProcessResult, String> {
    let result = process_zip_internal(
        zip_path.to_string(),
        booth_url.clone(),
        output_dir,
        &booth_client,
    )
    .await;

    match result {
        Ok(mut res) => {
            // Save to database if processing succeeded
            if res.success {
                let file_name = Path::new(&zip_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let file_size = std::fs::metadata(&zip_path)
                    .map(|meta| meta.len() as i64)
                    .ok();

                let db = db
                    .lock()
                    .map_err(|e| format!("Database lock error: {}", e))?;
                    
                let file_record = FileRecord {
                    id: None,
                    file_name,
                    file_path: zip_path.to_string(),
                    file_size,
                    booth_url: booth_url.clone(),
                    shop_name: res.shop_name.clone(),
                    product_name: res.product_name.clone(),
                    created_at: None,
                    processed: true,
                };

                match db.save_file(&file_record, tags) {
                    Ok(file_id) => {
                        res.message = format!(
                            "{} (ID: {}でデータベースに保存されました)",
                            res.message, file_id
                        );
                    }
                    Err(e) => {
                        error!("Failed to save to database: {}", e);
                        // Database save failure is not treated as error (file processing succeeded)
                    }
                }
            }
            Ok(res)
        }
        Err(e) => Ok(ProcessResult {
            success: false,
            message: format!("処理エラー: {}", e),
            shop_name: None,
            product_name: None,
        }),
    }
}

async fn process_zip_internal(
    zip_path: String,
    booth_url: Option<String>,
    output_dir: Option<String>,
    booth_client: &BoothClient,
) -> Result<ProcessResult> {
    let zip_path = PathBuf::from(&zip_path);

    // Check if zip file exists
    if !zip_path.exists() {
        return Err(anyhow!(
            "ZIPファイルが見つかりません: {}",
            zip_path.display()
        ));
    }

    // Extract shop name and product name from BOOTH URL
    let (shop_name, product_name) = if let Some(url) = &booth_url {
        match extract_booth_info_with_api(url, booth_client).await {
            Ok((shop, product)) => (
                Some(sanitize_folder_name(&shop)),
                Some(sanitize_folder_name(&product)),
            ),
            Err(_) => {
                // Fallback
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
        // If no URL, guess from filename
        let file_stem = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        (
            Some("Unknown_Shop".to_string()),
            Some(sanitize_folder_name(file_stem)),
        )
    };

    // Determine output directory
    let output_base = if let Some(dir) = output_dir {
        PathBuf::from(dir)
    } else {
        // Default to Desktop/BOOTH_Organized folder
        dirs::desktop_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("BOOTH_Organized")
    };

    // Create shop/product folder structure
    let final_output_dir = match (&shop_name, &product_name) {
        (Some(shop), Some(product)) => output_base.join(shop).join(product),
        (Some(shop), None) => output_base.join(shop),
        (None, Some(product)) => output_base.join("Unknown_Shop").join(product),
        (None, None) => output_base.join("Unknown"),
    };

    // Create directories
    fs::create_dir_all(&final_output_dir)
        .map_err(|e| anyhow!("出力ディレクトリの作成に失敗: {}", e))?;

    // Extract ZIP
    let extracted_files = extract_zip_with_encoding(&zip_path, &final_output_dir)?;

    Ok(ProcessResult {
        success: true,
        message: format!("{}個のファイルを展開しました", extracted_files.len()),
        shop_name,
        product_name,
    })
}

// Get BOOTH info using actual API
async fn extract_booth_info_with_api(
    url: &str,
    booth_client: &BoothClient,
) -> Result<(String, String)> {
    // Get actual product info using BOOTH API
    match booth_client.get_product_info(url).await {
        Ok(info) => Ok((info.shop_name, info.product_name)),
        Err(_) => {
            // Fallback: guess from URL
            extract_booth_info_fallback(url)
        }
    }
}

fn extract_booth_info_fallback(url: &str) -> Result<(String, String)> {
    // Extract shop name from BOOTH URL (fallback)
    let regex = regex::Regex::new(r"https://([^.]+)\.booth\.pm/items/(\d+)")
        .map_err(|e| anyhow!("正規表現の作成に失敗しました: {}", e))?;
    
    if let Some(captures) = regex.captures(url) {
        let shop_name = captures
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or("unknown_shop".to_string());
        let product_id = captures
            .get(2)
            .ok_or_else(|| anyhow!("商品IDの抽出に失敗しました"))?
            .as_str();
        let product_name = format!("product_{}", product_id);

        Ok((shop_name, product_name))
    } else {
        Err(anyhow!("無効なBOOTH URL: {}", url))
    }
}

// Generate safe folder name
fn sanitize_folder_name(name: &str) -> String {
    // Replace characters invalid on Windows/Linux/macOS
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = name.to_string();

    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }

    // Remove trailing dots and spaces
    sanitized = sanitized.trim_end_matches('.').trim().to_string();

    // Truncate if too long (255 character limit)
    if sanitized.len() > 200 {
        sanitized.truncate(200);
    }

    // Default name if empty
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

        // Detect and convert filename encoding
        let file_name = detect_and_convert_filename(file.name_raw())?;

        let output_path = output_dir.join(&file_name);

        if file.is_dir() {
            // Directory
            fs::create_dir_all(&output_path)?;
        } else {
            // File
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
    // First try UTF-8
    if let Ok(utf8_str) = std::str::from_utf8(raw_bytes) {
        return Ok(utf8_str.to_string());
    }

    // Try Shift-JIS
    let (decoded, _encoding, had_errors) = SHIFT_JIS.decode(raw_bytes);
    if !had_errors {
        return Ok(decoded.to_string());
    }

    // Try CP932 (Windows Japanese)
    if let Some(cp932) = encoding_rs::Encoding::for_label(b"cp932") {
        let (decoded_cp932, _encoding, had_errors) = cp932.decode(raw_bytes);
        if !had_errors {
            return Ok(decoded_cp932.to_string());
        }
    }

    // Fallback to lossy UTF-8
    Ok(String::from_utf8_lossy(raw_bytes).to_string())
}
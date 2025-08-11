use crate::{AppState, booth_client::BoothProductInfo, sanitize_folder_name};
use crate::config::booth;
use std::path::PathBuf;

// BOOTH URL検証コマンド（商品ページ専用）
#[tauri::command]
pub async fn validate_booth_url(url: String) -> Result<bool, String> {
    match url::Url::parse(&url) {
        Ok(parsed_url) => {
            let host = parsed_url.host_str().unwrap_or("");
            let path = parsed_url.path();
            
            // BOOTHドメインをチェック
            let is_booth_domain = host == booth::MAIN_DOMAIN || host.ends_with(booth::SUBDOMAIN_SUFFIX);
            
            if !is_booth_domain {
                return Ok(false);
            }
            
            // 商品ページのパスパターンをチェック（言語プレフィックス対応）
            // パターン: /items/[id] または /[lang]/items/[id]
            let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            
            let items_index = path_segments.iter().position(|&segment| segment == "items");
            
            if let Some(items_idx) = items_index {
                // "items"の次にセグメントがあることを確認
                if items_idx + 1 < path_segments.len() {
                    let item_id = path_segments[items_idx + 1];
                    // 商品IDが数字かどうかをチェック
                    let is_numeric_id = item_id.chars().all(|c| c.is_ascii_digit()) && !item_id.is_empty();
                    Ok(is_numeric_id)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}

// BOOTH商品情報取得コマンド
#[tauri::command]
pub async fn fetch_booth_product_info(
    url: String,
    state: tauri::State<'_, AppState>,
) -> Result<BoothProductInfo, String> {
    let booth_client = state.booth_client.clone();

    match booth_client.get_product_info(&url).await {
        Ok(product_info) => {
            // 商品情報の必須フィールドをチェック
            if product_info.shop_name.trim().is_empty() || product_info.product_name.trim().is_empty() {
                return Err("商品情報が不完全です。正しい商品ページのURLを確認してください。".to_string());
            }
            
            // 商品名が「ページが見つかりません」等のエラーメッセージでないかチェック
            let product_name_lower = product_info.product_name.to_lowercase();
            if product_name_lower.contains("not found") || 
               product_name_lower.contains("404") || 
               product_name_lower.contains("見つかりません") ||
               product_name_lower.contains("エラー") {
                return Err("商品ページが見つかりません。URLを確認してください。".to_string());
            }
            
            Ok(product_info)
        }
        Err(e) => Err(format!("BOOTH商品情報の取得に失敗しました: {} (購入ページやカートページではなく、商品ページのURLを入力してください)", e)),
    }
}

// BOOTHサムネイルダウンロードコマンド
#[tauri::command]
pub async fn download_booth_thumbnail(
    thumbnail_url: String,
    shop_name: String,
    product_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let booth_client = state.booth_client.clone();

    // サムネイル保存ディレクトリ
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("BOOTH_Organizer")
        .join("thumbnails");

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("サムネイルディレクトリの作成に失敗: {}", e))?;
    }

    // ファイル名生成（安全な文字のみ）
    let safe_shop = sanitize_folder_name(&shop_name);
    let safe_product = sanitize_folder_name(&product_name);

    // 拡張子を取得（JPGをデフォルト）
    let extension = if thumbnail_url.contains(".png") {
        "png"
    } else if thumbnail_url.contains(".gif") {
        "gif"
    } else if thumbnail_url.contains(".webp") {
        "webp"
    } else {
        "jpg"
    };

    let filename = format!("{}_{}.{}", safe_shop, safe_product, extension);
    let file_path = app_data_dir.join(&filename);

    // 既に存在する場合はスキップ
    if file_path.exists() {
        return Ok(file_path.to_string_lossy().to_string());
    }

    // サムネイルダウンロード
    match booth_client.download_thumbnail(&thumbnail_url).await {
        Ok(image_data) => {
            std::fs::write(&file_path, image_data)
                .map_err(|e| format!("サムネイルの保存に失敗: {}", e))?;

            Ok(file_path.to_string_lossy().to_string())
        }
        Err(e) => Err(format!("サムネイルのダウンロードに失敗: {}", e)),
    }
}

#[tauri::command]
pub async fn update_file_booth_url_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
    booth_url: Option<String>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // URLをproduct_urlフィールドとして更新
    use crate::database::FileUpdateFields;
    
    let update_fields = FileUpdateFields {
        product_id: None,
        product_name: None,
        author_name: None,
        price: None,
        description: None,
        thumbnail_url: None,
        product_url: booth_url,
    };
    
    db.update_file(file_id, update_fields)
        .map_err(|e| format!("Failed to update file BOOTH URL: {}", e))
}
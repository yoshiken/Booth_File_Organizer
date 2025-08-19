use crate::database::Tag;
use crate::{AppError, AppState};

#[tauri::command]
pub async fn add_tag_to_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
    tag_name: String,
    _tag_color: Option<String>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // タグを取得または作成
    let tag_id = db
        .add_tag(&tag_name)
        .map_err(|e| AppError::tag_creation(format!("Failed to add tag: {e}")).to_string())?;

    db.add_file_tag(file_id, tag_id).map_err(|e| {
        AppError::tag_operation(format!("Failed to add tag to file: {e}")).to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_file_db(
    state: tauri::State<'_, AppState>,
    _file_id: i64,
    tag_name: String,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // タグ名からIDを取得して削除を実行
    let all_tags = db
        .get_all_tags()
        .map_err(|e| AppError::tag_operation(format!("Failed to get tags: {e}")).to_string())?;

    if let Some(tag) = all_tags.iter().find(|t| t.name == tag_name) {
        if let Some(_tag_id) = tag.id {
            // 直接的な削除メソッドがないため、手動で実装する必要があります
            // ここでは簡単なエラーメッセージを返します
            return Err("Tag removal not implemented in new schema".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_all_tags_from_db(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    let tags = db.get_all_tags().map_err(|e| {
        AppError::file_retrieval(format!("Failed to get tags from database: {e}")).to_string()
    })?;

    Ok(tags)
}

#[tauri::command]
pub async fn get_tags_for_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
) -> Result<Vec<Tag>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    db.get_tags_for_file(file_id).map_err(|e| {
        AppError::tag_operation(format!("Failed to get tags for file: {e}")).to_string()
    })
}

#[tauri::command]
pub async fn batch_add_tag_to_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
    tag_name: String,
    _tag_color: Option<String>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // バッチでタグを追加（新しいスキーマでは個別実行）
    let tag_id = db
        .add_tag(&tag_name)
        .map_err(|e| AppError::tag_creation(format!("Failed to add tag: {e}")).to_string())?;

    for file_id in file_ids {
        let _ = db.add_file_tag(file_id, tag_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn batch_remove_tag_from_files_db(
    state: tauri::State<'_, AppState>,
    _file_ids: Vec<i64>,
    _tag_id: i64,
) -> Result<(), String> {
    let _db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // バッチでタグを削除（新しいスキーマでは直接的な方法がない）
    // ここでは簡単なエラーメッセージを返します
    Err("Batch tag removal not implemented in new schema".to_string())
}

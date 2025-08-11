use crate::{AppState, Tag, AppError};

#[tauri::command]
pub async fn add_tag_to_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
    tag_name: String,
    tag_color: Option<String>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    // タグを取得または作成
    let tag = db
        .get_or_create_tag(&tag_name, tag_color.as_deref())
        .map_err(|e| AppError::tag_creation(format!("Failed to get or create tag: {}", e)).to_string())?;

    if let Some(tag_id) = tag.id {
        db.add_tag_to_file(file_id, tag_id)
            .map_err(|e| AppError::tag_operation(format!("Failed to add tag to file: {}", e)).to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
    tag_name: String,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.remove_tag_from_file_by_name(file_id, &tag_name)
        .map_err(|e| AppError::tag_operation(format!("Failed to remove tag from file: {}", e)).to_string())
}

#[tauri::command]
pub async fn get_all_tags_from_db(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    let tags = db.get_all_tags()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get tags from database: {}", e)).to_string())?;
    
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
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.get_tags_for_file(file_id)
        .map_err(|e| AppError::tag_operation(format!("Failed to get tags for file: {}", e)).to_string())
}

#[tauri::command]
pub async fn batch_add_tag_to_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
    tag_name: String,
    tag_color: Option<String>,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.batch_add_tag_to_files(&file_ids, &tag_name, tag_color.as_deref())
        .map_err(|e| AppError::tag_operation(format!("Failed to batch add tag: {}", e)).to_string())
}

#[tauri::command]
pub async fn batch_remove_tag_from_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
    tag_id: i64,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.batch_remove_tag_from_files(&file_ids, tag_id)
        .map_err(|e| AppError::tag_operation(format!("Failed to batch remove tag: {}", e)).to_string())
}
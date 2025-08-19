// API型定義 - TypeScript自動生成対応
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// =============================================================================
// Core Domain Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub id: Option<i64>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub booth_product_id: Option<i64>,
    pub booth_shop_name: Option<String>,
    pub booth_product_name: Option<String>,
    pub booth_url: Option<String>,
    pub booth_price: Option<i64>,
    pub booth_thumbnail_path: Option<String>,
    pub encoding_info: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub category: Option<String>,
    pub parent_tag_id: Option<i64>,
    pub usage_count: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FileWithTags {
    pub file: FileRecord,
    pub tags: Vec<Tag>,
}

// =============================================================================
// Pagination Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaginationRequest {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponse<T> {
    pub items: Vec<T>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next_page: bool,
    pub has_prev_page: bool,
}

// =============================================================================
// API Request/Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilesRequest {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilesPaginatedRequest {
    pub query: String,
    pub pagination: PaginationRequest,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SearchByTagsRequest {
    pub tag_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SearchByTagsPaginatedRequest {
    pub tag_names: Vec<String>,
    pub pagination: PaginationRequest,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GetFilesPaginatedRequest {
    pub pagination: PaginationRequest,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GetTagsPaginatedRequest {
    pub pagination: PaginationRequest,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AddTagRequest {
    pub file_id: i64,
    pub tag_name: String,
    pub tag_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BatchAddTagRequest {
    pub file_ids: Vec<i64>,
    pub tag_name: String,
    pub tag_color: String,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoothUrlRequest {
    pub file_id: i64,
    pub booth_url: String,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BoothProductInfo {
    pub shop_name: String,
    pub product_name: String,
    pub price: Option<i64>,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
    pub product_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingStatus {
    pub current_step: String,
    pub progress: f64,
    pub total_files: usize,
    pub processed_files: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

// =============================================================================
// API Command Definitions
// =============================================================================

/// APIコマンドのメタデータ
#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ApiParameter>,
    pub return_type: String,
    pub examples: Vec<ApiExample>,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiParameter {
    pub name: String,
    pub type_name: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiExample {
    pub title: String,
    #[ts(skip)]
    pub request: serde_json::Value,
    #[ts(skip)]
    pub response: serde_json::Value,
}

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[ts(skip)]
    pub details: Option<serde_json::Value>,
}

// =============================================================================
// TypeScript Generation Helper
// =============================================================================

/// TypeScript型定義を生成するヘルパー関数
#[allow(dead_code)] // TypeScriptバインディング生成用（将来使用予定）
pub fn generate_typescript_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // TypeScript型定義ファイルを生成
    FileRecord::export()?;
    Tag::export()?;
    FileWithTags::export()?;
    PaginationRequest::export()?;
    // Note: PaginationResponse<T> is generic, so we'll export specific instances
    SearchFilesRequest::export()?;
    SearchFilesPaginatedRequest::export()?;
    SearchByTagsRequest::export()?;
    SearchByTagsPaginatedRequest::export()?;
    GetFilesPaginatedRequest::export()?;
    GetTagsPaginatedRequest::export()?;
    AddTagRequest::export()?;
    BatchAddTagRequest::export()?;
    UpdateBoothUrlRequest::export()?;
    BoothProductInfo::export()?;
    ProcessingStatus::export()?;
    ApiCommand::export()?;
    ApiParameter::export()?;
    ApiExample::export()?;
    ApiError::export()?;

    // TypeScript bindings generated successfully
    Ok(())
}

/// OpenAPI仕様を生成するヘルパー関数
#[allow(dead_code)] // OpenAPI仕様生成用（将来使用予定）
pub fn generate_openapi_spec() -> Result<String, Box<dyn std::error::Error>> {
    use schemars::schema_for;

    let schema = schema_for!(ApiCommand);
    Ok(serde_json::to_string_pretty(&schema)?)
}

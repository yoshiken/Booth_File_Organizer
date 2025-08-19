use thiserror::Error;

/// アプリケーション全体で使用する統一エラー型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("BOOTH client error: {0}")]
    BoothClient(#[from] crate::booth_client::BoothClientError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("ZIP extraction error: {0}")]
    ZipExtraction(#[from] zip::result::ZipError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Encoding error: {message}")]
    Encoding { message: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    // Additional variants for common string error patterns
    // These preserve the exact original error message without adding prefixes
    #[error("{0}")]
    DatabaseLock(String),

    #[error("{0}")]
    FileSave(String),

    #[error("{0}")]
    FileRetrieval(String),

    #[error("{0}")]
    FileDeletion(String),

    #[error("{0}")]
    FileUpdate(String),

    #[error("{0}")]
    TagOperation(String),

    #[error("{0}")]
    TagCreation(String),

    #[error("{0}")]
    Custom(String),
}

impl AppError {
    /// バリデーションエラーを作成
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// エンコーディングエラーを作成
    pub fn encoding(message: impl Into<String>) -> Self {
        Self::Encoding {
            message: message.into(),
        }
    }

    /// 設定エラーを作成
    pub fn config(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// 不明なエラーを作成
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown(message.into())
    }

    /// データベースロックエラーを作成
    pub fn database_lock(message: impl Into<String>) -> Self {
        Self::DatabaseLock(message.into())
    }

    /// ファイル保存エラーを作成
    pub fn file_save(message: impl Into<String>) -> Self {
        Self::FileSave(message.into())
    }

    /// ファイル取得エラーを作成
    pub fn file_retrieval(message: impl Into<String>) -> Self {
        Self::FileRetrieval(message.into())
    }

    /// ファイル削除エラーを作成
    pub fn file_deletion(message: impl Into<String>) -> Self {
        Self::FileDeletion(message.into())
    }

    /// ファイル更新エラーを作成
    pub fn file_update(message: impl Into<String>) -> Self {
        Self::FileUpdate(message.into())
    }

    /// タグ操作エラーを作成
    pub fn tag_operation(message: impl Into<String>) -> Self {
        Self::TagOperation(message.into())
    }

    /// タグ作成エラーを作成
    pub fn tag_creation(message: impl Into<String>) -> Self {
        Self::TagCreation(message.into())
    }

    /// カスタムエラーを作成 (任意の文字列エラーメッセージ用)
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

/// アプリケーション全体で使用するResult型
pub type AppResult<T> = Result<T, AppError>;

/// エラーのカテゴリ分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// ユーザー操作エラー（修正可能）
    UserError,
    /// システムエラー（再試行可能）
    SystemError,
    /// 設定エラー（設定変更が必要）
    ConfigError,
    /// 重大なエラー（アプリケーション終了）
    FatalError,
}

impl AppError {
    /// エラーのカテゴリを取得
    pub fn category(&self) -> ErrorCategory {
        match self {
            AppError::Validation { .. } => ErrorCategory::UserError,
            AppError::UrlParse(_) => ErrorCategory::UserError,
            AppError::Configuration(_) => ErrorCategory::ConfigError,
            AppError::Database(_) => ErrorCategory::SystemError,
            AppError::Network(_) => ErrorCategory::SystemError,
            AppError::Io(_) => ErrorCategory::SystemError,
            AppError::BoothClient(_) => ErrorCategory::SystemError,
            AppError::ZipExtraction(_) => ErrorCategory::SystemError,
            AppError::Internal(_) => ErrorCategory::SystemError,
            AppError::Serialization(_) => ErrorCategory::FatalError,
            AppError::Encoding { .. } => ErrorCategory::SystemError,
            AppError::Unknown(_) => ErrorCategory::FatalError,
            AppError::DatabaseLock(_) => ErrorCategory::SystemError,
            AppError::FileSave(_) => ErrorCategory::SystemError,
            AppError::FileRetrieval(_) => ErrorCategory::SystemError,
            AppError::FileDeletion(_) => ErrorCategory::SystemError,
            AppError::FileUpdate(_) => ErrorCategory::SystemError,
            AppError::TagOperation(_) => ErrorCategory::SystemError,
            AppError::TagCreation(_) => ErrorCategory::SystemError,
            AppError::Custom(_) => ErrorCategory::SystemError,
        }
    }

    /// ユーザー向けメッセージを取得
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(_) => {
                "データベースエラーが発生しました。アプリケーションを再起動してください。"
                    .to_string()
            }
            AppError::BoothClient(_) => {
                "BOOTH商品情報の取得に失敗しました。URLを確認してください。".to_string()
            }
            AppError::Network(_) => {
                "ネットワークエラーが発生しました。インターネット接続を確認してください。"
                    .to_string()
            }
            AppError::Io(_) => {
                "ファイル操作中にエラーが発生しました。ファイルの権限を確認してください。"
                    .to_string()
            }
            AppError::UrlParse(_) => {
                "無効なURLです。正しいBOOTH URLを入力してください。".to_string()
            }
            AppError::ZipExtraction(_) => {
                "ZIPファイルの展開に失敗しました。ファイルが破損している可能性があります。"
                    .to_string()
            }
            AppError::Internal(err) => format!("内部エラーが発生しました: {err}"),
            AppError::Validation { field, message } => format!("{field}: {message}"),
            AppError::Configuration(msg) => format!("設定エラー: {msg}"),
            AppError::Encoding { message } => format!("文字エンコーディングエラー: {message}"),
            AppError::Serialization(_) => "データの変換中にエラーが発生しました。".to_string(),
            AppError::Unknown(msg) => format!("予期しないエラー: {msg}"),
            // The new error types will display their contained message directly
            AppError::DatabaseLock(msg) => msg.clone(),
            AppError::FileSave(msg) => msg.clone(),
            AppError::FileRetrieval(msg) => msg.clone(),
            AppError::FileDeletion(msg) => msg.clone(),
            AppError::FileUpdate(msg) => msg.clone(),
            AppError::TagOperation(msg) => msg.clone(),
            AppError::TagCreation(msg) => msg.clone(),
            AppError::Custom(msg) => msg.clone(),
        }
    }

    /// 再試行可能かどうか
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::SystemError)
    }
}

/// String から AppError への変換 (既存のエラーメッセージを保持)
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Custom(msg)
    }
}

/// &str から AppError への変換 (既存のエラーメッセージを保持)
impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Custom(msg.to_string())
    }
}

/// AppError を String に変換 (エラーメッセージの表示)
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// Helper functions to make conversion from existing patterns seamless
impl AppError {
    /// Create an error from a formatted string (replaces format!(...) patterns)
    pub fn from_format(msg: String) -> Self {
        AppError::Custom(msg)
    }

    /// Create database lock error from formatted string
    pub fn db_lock_from_format(msg: String) -> Self {
        AppError::DatabaseLock(msg)
    }

    /// Create file save error from formatted string
    pub fn file_save_from_format(msg: String) -> Self {
        AppError::FileSave(msg)
    }

    /// Create file retrieval error from formatted string
    pub fn file_retrieval_from_format(msg: String) -> Self {
        AppError::FileRetrieval(msg)
    }

    /// Create file deletion error from formatted string
    pub fn file_deletion_from_format(msg: String) -> Self {
        AppError::FileDeletion(msg)
    }

    /// Create file update error from formatted string
    pub fn file_update_from_format(msg: String) -> Self {
        AppError::FileUpdate(msg)
    }

    /// Create tag operation error from formatted string
    pub fn tag_operation_from_format(msg: String) -> Self {
        AppError::TagOperation(msg)
    }

    /// Create tag creation error from formatted string
    pub fn tag_creation_from_format(msg: String) -> Self {
        AppError::TagCreation(msg)
    }
}

/// Convert AppResult<T> to Result<T, String> for Tauri commands
/// This ensures all error messages are preserved exactly
pub fn to_tauri_result<T>(result: AppResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

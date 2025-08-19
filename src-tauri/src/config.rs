// Configuration constants for BOOTH File Organizer
// This module centralizes all magic numbers and hardcoded strings to improve maintainability

/// Application configuration constants
pub mod app {
    /// Name of the application data directory
    pub const DATA_DIR_NAME: &str = "BOOTH_Organizer";

    /// Database file name
    pub const DATABASE_FILENAME: &str = "booth_organizer.db";
}

/// Tag-related configuration constants
pub mod tags {
    /// Maximum allowed length for tag names
    pub const MAX_TAG_LENGTH: usize = 50;

    /// Default tag color (blue) - Reserved for future UI implementation
    #[allow(dead_code)]
    pub const DEFAULT_TAG_COLOR: &str = "#007ACC";

    /// Alternative default color (blue) - Reserved for future UI implementation
    #[allow(dead_code)]
    pub const ALTERNATIVE_DEFAULT_COLOR: &str = "#3498db";

    /// Initial usage count for new tags - Reserved for future feature
    #[allow(dead_code)]
    pub const INITIAL_USAGE_COUNT: i64 = 0;
}

/// File processing configuration constants
pub mod files {
    /// Maximum filename length before truncation
    pub const MAX_FILENAME_LENGTH: usize = 200;

    /// Default file size for unknown files - Reserved for future feature
    #[allow(dead_code)]
    pub const UNKNOWN_FILE_SIZE: i64 = 0;
}

/// BOOTH-related configuration constants
pub mod booth {
    /// Main BOOTH domain
    pub const MAIN_DOMAIN: &str = "booth.pm";

    /// BOOTH subdomain suffix pattern
    pub const SUBDOMAIN_SUFFIX: &str = ".booth.pm";

    /// Default shop name for unknown shops - Reserved for future feature
    #[allow(dead_code)]
    pub const DEFAULT_SHOP_NAME: &str = "Unknown_Shop";

    /// Supported language codes - Reserved for internationalization
    #[allow(dead_code)]
    pub const LANG_JAPANESE: &str = "ja";
    #[allow(dead_code)]
    pub const LANG_ENGLISH: &str = "en";
}

/// Database schema constants
pub mod database {
    /// Files table name - Reserved for database abstraction
    #[allow(dead_code)]
    pub const FILES_TABLE: &str = "files";

    /// Tags table name - Reserved for database abstraction
    #[allow(dead_code)]
    pub const TAGS_TABLE: &str = "tags";

    /// File-tags junction table name - Reserved for database abstraction
    #[allow(dead_code)]
    pub const FILE_TAGS_TABLE: &str = "file_tags";

    /// Default datetime function - Reserved for database abstraction
    #[allow(dead_code)]
    pub const CURRENT_TIMESTAMP: &str = "CURRENT_TIMESTAMP";

    /// Datetime now function - Reserved for database abstraction
    #[allow(dead_code)]
    pub const DATETIME_NOW: &str = "datetime('now')";
}


/// Regex group indices
pub mod regex {
    /// First capture group index
    pub const GROUP_1: usize = 1;

    /// Second capture group index
    pub const GROUP_2: usize = 2;
}

/// File extension filters
pub mod file_filters {
    /// Archive file extensions - Reserved for future feature
    #[allow(dead_code)]
    pub const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz", "bz2"];
}

/// UI text constants - Reserved for future UI implementation
#[allow(dead_code)]
pub mod ui_text {
    /// Folder selection dialog title (Japanese)
    pub const SELECT_OUTPUT_FOLDER_JP: &str = "出力先フォルダを選択";

    /// Archive file selection dialog title (Japanese)
    pub const SELECT_ARCHIVE_FILE_JP: &str = "BOOTHアーカイブファイルを選択";

    /// Archive filter name
    pub const ARCHIVE_FILTER_NAME: &str = "Archive files";

    /// File selection error prefix (Japanese)
    pub const FILE_SELECTION_ERROR_JP: &str = "ファイル選択エラー: {}";

    /// Folder selection error prefix (Japanese)
    pub const FOLDER_SELECTION_ERROR_JP: &str = "フォルダ選択エラー: {}";

    /// No files selected message (Japanese)
    pub const NO_FILES_SELECTED_JP: &str = "ファイルが選択されませんでした";

    /// Processing error prefix (Japanese)
    pub const PROCESSING_ERROR_JP: &str = "処理エラー: {}";

    /// Files selected message format (Japanese)
    pub const FILES_SELECTED_FORMAT_JP: &str = "{}個のファイルが選択されました";

    /// Save success message format (Japanese)
    pub const SAVE_SUCCESS_FORMAT_JP: &str = "{} (ID: {}でデータベースに保存されました)";

    /// ZIP not found error format (Japanese)
    pub const ZIP_NOT_FOUND_JP: &str = "ZIPファイルが見つかりません: {}";

    /// Unknown file extension
    pub const UNKNOWN_EXTENSION: &str = "unknown";
}

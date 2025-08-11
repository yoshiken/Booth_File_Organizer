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
    
    /// Default tag color (blue)
    pub const DEFAULT_TAG_COLOR: &str = "#007ACC";
    
    /// Alternative default color (blue)
    pub const ALTERNATIVE_DEFAULT_COLOR: &str = "#3498db";
    
    /// Initial usage count for new tags
    pub const INITIAL_USAGE_COUNT: i64 = 0;
}

/// File processing configuration constants
pub mod files {
    /// Maximum filename length before truncation
    pub const MAX_FILENAME_LENGTH: usize = 200;
    
    /// Default file size for unknown files
    pub const UNKNOWN_FILE_SIZE: i64 = 0;
}

/// BOOTH-related configuration constants
pub mod booth {
    /// Main BOOTH domain
    pub const MAIN_DOMAIN: &str = "booth.pm";
    
    /// BOOTH subdomain suffix pattern
    pub const SUBDOMAIN_SUFFIX: &str = ".booth.pm";
    
    /// Default shop name for unknown shops
    pub const DEFAULT_SHOP_NAME: &str = "Unknown_Shop";
    
    /// Supported language codes
    pub const LANG_JAPANESE: &str = "ja";
    pub const LANG_ENGLISH: &str = "en";
}

/// Database schema constants
pub mod database {
    /// Files table name
    pub const FILES_TABLE: &str = "files";
    
    /// Tags table name
    pub const TAGS_TABLE: &str = "tags";
    
    /// File-tags junction table name
    pub const FILE_TAGS_TABLE: &str = "file_tags";
    
    /// Default datetime function
    pub const CURRENT_TIMESTAMP: &str = "CURRENT_TIMESTAMP";
    
    /// Datetime now function
    pub const DATETIME_NOW: &str = "datetime('now')";
}

/// Common test values (for test files only)
#[cfg(test)]
pub mod test_values {
    /// Common test file sizes
    pub const TEST_SIZE_1KB: i64 = 1024;
    pub const TEST_SIZE_2KB: i64 = 2048;
    pub const TEST_SIZE_3KB: i64 = 3072;
    pub const TEST_SIZE_4KB: i64 = 4096;
    pub const TEST_SIZE_5KB: i64 = 5120;
    
    /// Test colors
    pub const TEST_COLOR_RED: &str = "#FF0000";
    pub const TEST_COLOR_GREEN: &str = "#00FF00";
    pub const TEST_COLOR_BLUE: &str = "#0000FF";
    pub const TEST_COLOR_ORANGE: &str = "#f39c12";
    pub const TEST_COLOR_PURPLE: &str = "#9b59b6";
    pub const TEST_COLOR_CYAN: &str = "#e74c3c";
    
    /// Common test URLs
    pub const TEST_BOOTH_URL_1: &str = "https://booth.pm/items/123456";
    pub const TEST_BOOTH_URL_2: &str = "https://example.booth.pm/items/789";
    pub const TEST_BOOTH_URL_3: &str = "https://coolshop.booth.pm/items/123456";
    
    /// Test product IDs
    pub const TEST_PRODUCT_ID_1: i64 = 123456;
    pub const TEST_PRODUCT_ID_2: i64 = 789;
    
    /// Test prices
    pub const TEST_PRICE_3000: i64 = 3000;
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
    /// Archive file extensions
    pub const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "rar", "7z", "tar", "gz", "bz2"];
}

/// UI text constants
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
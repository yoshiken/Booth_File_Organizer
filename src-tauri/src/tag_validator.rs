// タグバリデーション機能のモジュール
// 現在は未使用ですが、将来の拡張のため基本構造を保持

use crate::config::tags;

/// メインのバリデーション関数（将来の実装用にスタブとして保持）
#[allow(dead_code)]
pub fn is_valid_tag(tag_text: &str) -> bool {
    // 基本的な検証のみ実装（現在は未使用）
    !tag_text.trim().is_empty() && tag_text.trim().len() <= tags::MAX_TAG_LENGTH
}
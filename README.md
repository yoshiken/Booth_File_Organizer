# 📦 BOOTH File Organizer

[![Build Status](https://github.com/your-username/booth-organizer/workflows/CI/badge.svg)](https://github.com/your-username/booth-organizer/actions)
[![Release](https://img.shields.io/github/v/release/your-username/booth-organizer)](https://github.com/your-username/booth-organizer/releases)
[![License](https://img.shields.io/github/license/your-username/booth-organizer)](LICENSE)

VRChat用BOOTHアセットの自動整理ツール - Rust/Tauri製のクロスプラットフォーム対応デスクトップアプリケーション

## ✨ 機能

### 🎯 コア機能
- **自動ファイル整理**: ZIPファイルを`ショップ名/商品名`フォルダに自動展開
- **BOOTH連携**: 商品URLから自動でメタデータ取得
- **インテリジェント命名**: 日本語ファイル名の文字化け対応
- **タグ管理**: カスタムタグでアセット分類

### 🔍 検索・タグ管理
- **フルテキスト検索**: ファイル名、ショップ名、商品名から横断検索
- **タグベース検索**: 複数タグでのAND検索
- **タグ管理モーダル**: ファイルごとにタグの追加・削除
- **重複ファイル検出**: ハッシュベースで同一ファイルを特定
- **視覚的UI**: モダンな検索結果表示

### 🚀 パフォーマンス
- **非同期処理**: UIブロックなしの高速処理
- **レート制限**: BOOTH APIへの負荷軽減
- **SQLiteDB**: 高速なローカルデータベース
- **インデックス最適化**: 大量ファイルでも高速検索

## 🔧 インストール

### Windows（推奨：ポータブル版）
1. [Releases](https://github.com/your-username/booth-organizer/releases)から最新の`BOOTHFileOrganizer-Portable-v*.exe`をダウンロード
2. ダウンロードしたファイルを任意のフォルダに配置
3. 実行ファイルをダブルクリックで起動
4. **インストール不要** - すぐに使用開始可能

> **ポータブル版の利点**
> - インストール不要で即座に使用可能
> - Windowsセキュリティの誤検知を回避しやすい
> - 管理者権限不要
> - USBメモリなどに入れて持ち運び可能

### Windows（インストーラー版）
1. [Releases](https://github.com/your-username/booth-organizer/releases)から最新の`booth-organizer-setup.exe`をダウンロード
2. インストーラーを実行
3. アプリケーションを起動

### macOS
1. [Releases](https://github.com/your-username/booth-organizer/releases)から最新の`.dmg`ファイルをダウンロード
2. DMGをマウントしてアプリケーションフォルダーにドラッグ
3. アプリケーションを起動

### Linux
1. [Releases](https://github.com/your-username/booth-organizer/releases)から最新の`.AppImage`をダウンロード
2. 実行権限を付与: `chmod +x booth-organizer.AppImage`
3. アプリケーションを実行: `./booth-organizer.AppImage`

## 🚀 使い方

### 基本的な使用手順

1. **BOOTH URLを入力**
   ```
   https://shop.booth.pm/items/12345
   ```

2. **ZIPファイルを選択**
   - 「ファイルを選択」ボタンで複数ZIPを選択
   - またはドラッグ&ドロップで追加

3. **処理実行**
   - 「処理開始」ボタンで自動整理を開始
   - 進捗バーで処理状況を確認

4. **結果確認**
   - デスクトップの`BOOTH_Organized`フォルダに整理済みファイル
   - アプリ内でタグ付けと検索が可能

### 高度な機能

#### 🏷️ タグ管理
```
Avatar, VRChat, 衣装, アクセサリー
```
- カスタムカラーでタグを視覚的に分類
- ファイルに複数タグを関連付け
- タグクリックで瞬時にフィルタリング
- 「タグ」ボタンからモーダルでタグの追加・削除が可能

#### 🔍 検索機能
- **クエリ例**: `avatar 衣装` → アバター衣装ファイルを検索
- **タグ検索**: 複数タグでAND条件検索
- **重複検出**: 同一ファイルのコピーを特定

## 🛠️ 開発

### 必要環境
- **Rust** 1.70+
- **Node.js** 18+
- **npm** 9+

### セットアップ
```bash
# リポジトリクローン
git clone https://github.com/your-username/booth-organizer.git
cd booth-organizer

# 依存関係インストール
npm install

# 開発サーバー起動
npm run tauri dev
```

### ビルド
```bash
# Windowsポータブル版ビルド（推奨）
npm run tauri:build:windows:portable

# プロダクションビルド
npm run tauri build

# Windowsクロスビルド (Linux環境)
npm run tauri:build:windows
```

### テスト
```bash
# Rustテスト実行
cd src-tauri
cargo test

# フロントエンドテスト
npm test
```

## 🏗️ アーキテクチャ

### 技術スタック
```
Frontend: React 18 + TypeScript + Vite
Backend:  Rust + Tauri 2.0 + Repository Pattern
Database: SQLite + rusqlite
HTTP:     reqwest + scraper
Testing:  TDD (Test-Driven Development)
UI:       Component-Based Architecture + CSS Variables
```

### 🔥 **v1.0.0 (2025年8月) 大型アップデート完了**
**モジュラーアーキテクチャ + 品質向上で完全リニューアル！**

#### 🏗️ **アーキテクチャ刷新**
- ✅ **コマンド分離**: 31個の巨大コマンドハンドラーを6つの機能別モジュールに分離
- ✅ **Repository Pattern**: God Object解消、責務別Repository実装
- ✅ **エラーハンドリング統一**: 一元的なエラー処理システム構築
- ✅ **設定値中央管理**: マジックナンバー・文字列を定数化

#### ⚡ **機能強化**
- ✅ **BOOTH JSON API対応**: HTML解析から高速JSONAPIへ移行
- ✅ **ページネーション機能**: 大量データでも快適操作
- ✅ **データベース最適化**: 戦略的インデックスによる高速化
- ✅ **構造化ログシステム**: println!文を統一ログシステムに置換

#### 🛡️ **品質向上**
- ✅ **TDD徹底**: 各変更でテスト・ビルド実行
- ✅ **Clippy警告解決**: 30個→23個に警告削減
- ✅ **未使用コード整理**: デッドコードの適切な管理
- ✅ **Default trait実装**: Rustベストプラクティス準拠

### アーキテクチャ図

#### 🎨 Frontend Architecture (Component-Based)
```
App.tsx (461行) - 状態管理とコーディネーション
├── UrlInputSection (109行)      # URL入力・検証
├── ProcessingQueueSection (282行) # ファイル処理キュー
├── FileSearchSection (484行)     # 検索・フィルタリング
└── FileSyncSection (200行)       # 同期機能
```

#### 🚀 Backend Architecture (Modular Commands + Repository Pattern)
```
lib.rs - アプリケーション状態管理
├── file_commands.rs         # ファイルDB操作 (8コマンド)
├── tag_commands.rs          # タグ管理操作 (6コマンド)
├── booth_commands.rs        # BOOTH API統合 (4コマンド)
├── process_commands.rs      # ZIP/ファイル処理 (3コマンド)
├── sync_commands.rs         # 検索・同期・統計 (6コマンド)
├── system_commands.rs       # システムユーティリティ (4コマンド)
├── paginated_commands.rs    # ページネーション機能
├── errors.rs                # 統一エラーハンドリング
├── config.rs                # 設定値中央管理
└── repositories/           # Repository Pattern実装
    ├── FileRepository      # ファイル操作専用
    ├── TagRepository       # タグ操作専用
    └── FileTagRepository   # 関連操作専用
```

### ディレクトリ構造
```
booth-organizer/
├── src/                         # React フロントエンド
│   ├── App.tsx                 # メインコーディネーター (461行)
│   ├── App.css                 # スタイル定義
│   └── components/             # 分離されたコンポーネント
│       ├── UrlInputSection.tsx      # URL入力・検証
│       ├── ProcessingQueueSection.tsx # ファイル処理キュー
│       ├── FileSearchSection.tsx     # 検索・フィルタリング
│       └── FileSyncSection.tsx       # 同期機能
├── src-tauri/                  # Rust バックエンド
│   ├── src/
│   │   ├── lib.rs              # アプリケーション状態・ヘルパー関数
│   │   ├── file_commands.rs    # ファイルDB操作コマンド
│   │   ├── tag_commands.rs     # タグ管理コマンド
│   │   ├── booth_commands.rs   # BOOTH API統合コマンド
│   │   ├── process_commands.rs # ZIP/ファイル処理コマンド
│   │   ├── sync_commands.rs    # 検索・同期・統計コマンド
│   │   ├── system_commands.rs  # システムユーティリティコマンド
│   │   ├── paginated_commands.rs # ページネーション機能
│   │   ├── errors.rs           # 統一エラーハンドリング
│   │   ├── config.rs           # 設定値中央管理
│   │   ├── database_refactored.rs # Database ファサード
│   │   ├── repositories/       # Repository Pattern実装
│   │   │   ├── mod.rs
│   │   │   ├── file_repository.rs    # ファイル操作
│   │   │   ├── tag_repository.rs     # タグ操作
│   │   │   └── file_tag_repository.rs # 関連操作
│   │   ├── services/           # サービス層
│   │   │   ├── mod.rs
│   │   │   └── file_processor.rs     # ファイル処理サービス
│   │   ├── booth_client.rs     # BOOTH API クライアント
│   │   ├── api_types.rs        # API型定義・バインディング生成
│   │   └── tag_validator.rs    # タグ検証システム
│   ├── Cargo.toml              # Rust 依存関係
│   └── ...
├── Makefile                    # ビルド自動化
├── BUILD.md                    # ビルド詳細手順
└── .github/workflows/          # CI/CD 設定
```

### 🧪 テスト戦略
```
Repository Tests (TDD):
├── FileRepository      - 5 テスト (100% カバレッジ)
├── TagRepository       - 6 テスト (100% カバレッジ)
├── FileTagRepository   - 4 テスト (100% カバレッジ)
└── DatabaseRefactored  - 4 統合テスト

Frontend Tests (予定):
├── Component Unit Tests
├── Integration Tests
└── E2E Tests
```

## 🔒 セキュリティ

- **SQL Injection防止**: Prepared statements使用
- **Path Traversal対策**: ファイルパス検証
- **入力検証**: URL・ファイル名の厳密なバリデーション
- **サンドボックス**: Tauriセキュリティモデル準拠

## 🤝 コントリビューション

1. Forkして新しいブランチを作成
2. 機能追加・バグ修正を実装
3. テストを追加・実行
4. Pull Requestを作成

### 開発ガイドライン
- **Code Style**: `cargo fmt` + `eslint`
- **Testing**: TDD (Test-Driven Development)
- **Commits**: Conventional Commits形式
- **Documentation**: Rustdoc + JSDoc

## 📄 ライセンス

MIT License - 詳細は[LICENSE](LICENSE)を参照

## 🙏 謝辞

- [Tauri](https://tauri.app/) - クロスプラットフォームフレームワーク
- [BOOTH](https://booth.pm/) - VRアセットマーケットプレイス
- [VRChat](https://vrchat.com/) - メタバースプラットフォーム

---

**💡 ヒント**: 初回起動時はファイアウォールの許可が必要な場合があります
**🐛 バグ報告**: [Issues](https://github.com/your-username/booth-organizer/issues)でお知らせください
**💬 サポート**: [Discussions](https://github.com/your-username/booth-organizer/discussions)で質問・提案をどうぞ
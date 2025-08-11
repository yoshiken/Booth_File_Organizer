# BOOTH File Organizer - Build Automation
# =======================================

# Version and build info
VERSION = 1.0.0
BUILD_DATE = $(shell date +%Y-%m-%d)
REPO_ROOT = $(shell pwd)
TARGET_DIR = src-tauri/target/x86_64-pc-windows-gnu/release
PORTABLE_DIR = BOOTH_File_Organizer_Windows_Portable
PACKAGE_NAME = BOOTH_File_Organizer_v$(VERSION)_Windows_Portable

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

.PHONY: help build-frontend build-windows build-portable package clean test install-deps check-deps portable-only package-only lint format

# Default target
help:
	@echo "$(BLUE)BOOTH File Organizer Build System$(NC)"
	@echo "=================================="
	@echo ""
	@echo "$(GREEN)Available targets:$(NC)"
	@echo "  $(YELLOW)build-windows$(NC)    - Build Windows executable"
	@echo "  $(YELLOW)build-portable$(NC)   - Create Windows portable version"
	@echo "  $(YELLOW)package$(NC)          - Package portable version"
	@echo "  $(YELLOW)portable-only$(NC)    - Create portable from existing exe"
	@echo "  $(YELLOW)package-only$(NC)     - Package existing portable"
	@echo "  $(YELLOW)all$(NC)              - Build everything (frontend + windows + portable + package)"
	@echo "  $(YELLOW)clean$(NC)            - Clean build artifacts"
	@echo "  $(YELLOW)test$(NC)             - Run tests"
	@echo "  $(YELLOW)lint$(NC)             - Run code linting"
	@echo "  $(YELLOW)format$(NC)           - Format code"
	@echo "  $(YELLOW)install-deps$(NC)     - Install required dependencies"
	@echo "  $(YELLOW)check-deps$(NC)       - Check if dependencies are installed"
	@echo ""
	@echo "$(GREEN)Version:$(NC) $(VERSION)"
	@echo "$(GREEN)Build Date:$(NC) $(BUILD_DATE)"

# Check dependencies
check-deps:
	@echo "$(BLUE)Checking dependencies...$(NC)"
	@command -v npm >/dev/null 2>&1 || { echo "$(RED)npm is required but not installed$(NC)"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "$(RED)cargo is required but not installed$(NC)"; exit 1; }
	@command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 || { echo "$(RED)mingw-w64 is required but not installed$(NC)"; exit 1; }
	@rustup target list --installed | grep -q x86_64-pc-windows-gnu || { echo "$(RED)Rust Windows GNU target not installed$(NC)"; exit 1; }
	@echo "$(GREEN)All dependencies are installed$(NC)"

# Install required dependencies
install-deps:
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@echo "$(YELLOW)Installing Rust Windows GNU target...$(NC)"
	rustup target add x86_64-pc-windows-gnu
	@echo "$(YELLOW)Installing npm dependencies...$(NC)"
	npm install
	@echo "$(GREEN)Dependencies installed successfully$(NC)"

# Build frontend
build-frontend:
	@echo "$(BLUE)Building frontend...$(NC)"
	npm run build
	@echo "$(GREEN)Frontend build completed$(NC)"

# Run tests
test:
	@echo "$(BLUE)Running tests...$(NC)"
	cd src-tauri && cargo test
	@echo "$(GREEN)Tests completed$(NC)"

# Build Windows executable
build-windows: check-deps build-frontend
	@echo "$(BLUE)Building Windows executable...$(NC)"
	npx tauri build --target x86_64-pc-windows-gnu
	@echo "$(GREEN)Windows build completed$(NC)"
	@echo "$(YELLOW)Executable location:$(NC) $(TARGET_DIR)/booth-organizer.exe"
	@ls -lh $(TARGET_DIR)/booth-organizer.exe

# Create portable version
build-portable: build-windows
	@echo "$(BLUE)Creating portable version...$(NC)"
	@cd $(TARGET_DIR) && rm -rf $(PORTABLE_DIR)
	@cd $(TARGET_DIR) && mkdir -p $(PORTABLE_DIR)
	@cd $(TARGET_DIR) && cp booth-organizer.exe $(PORTABLE_DIR)/
	@cd $(TARGET_DIR) && cp *.dll $(PORTABLE_DIR)/
	@cd $(TARGET_DIR) && echo "BOOTH File Organizer - Windows Portable Edition" > $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "===============================================" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "バージョン: $(VERSION)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "ビルド日: $(BUILD_DATE)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【セットアップ】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "1. このフォルダ内のすべてのファイルを任意の場所にコピーしてください" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "2. booth-organizer.exe をダブルクリックして実行します" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【必要なファイル】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- booth-organizer.exe (メイン実行ファイル)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- booth_organizer_lib.dll (アプリケーションライブラリ)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- WebView2Loader.dll (Webビューコンポーネント)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【動作要件】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- Windows 10/11 x64" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- Microsoft Edge WebView2 Runtime (通常は既にインストール済み)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【主な機能】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- BOOTHアセットの自動ダウンロード・整理" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- タグ管理機能" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ファイル検索・絞り込み" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- usage_count=0のタグ自動削除機能" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【新機能】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- タグが使用されなくなった時の自動削除" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ファイル削除・タグ削除時の自動クリーンアップ" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- データベースの最適化" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- モダンなタグUI" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- 改良されたタグ管理モーダル" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【注意事項】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- 初回起動時にWindows Defenderの警告が表示される場合があります" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ポータブル版のため、設定は実行ファイルと同じフォルダに保存されます" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- アンインストールは、このフォルダを削除するだけです" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "お問い合わせ: GitHub Issues" >> $(PORTABLE_DIR)/README.txt
	@echo "$(GREEN)Portable version created$(NC)"
	@echo "$(YELLOW)Portable directory:$(NC) $(TARGET_DIR)/$(PORTABLE_DIR)"
	@cd $(TARGET_DIR) && ls -la $(PORTABLE_DIR)/

# Package portable version
package: build-portable
	@echo "$(BLUE)Packaging portable version...$(NC)"
	@cd $(TARGET_DIR) && rm -f $(PACKAGE_NAME).tar.gz
	@cd $(TARGET_DIR) && tar -czf $(PACKAGE_NAME).tar.gz $(PORTABLE_DIR)/
	@echo "$(GREEN)Package created successfully$(NC)"
	@echo "$(YELLOW)Package location:$(NC) $(TARGET_DIR)/$(PACKAGE_NAME).tar.gz"
	@cd $(TARGET_DIR) && ls -lh $(PACKAGE_NAME).tar.gz
	@echo ""
	@echo "$(GREEN)Build Summary:$(NC)"
	@echo "=============="
	@echo "$(YELLOW)Version:$(NC) $(VERSION)"
	@echo "$(YELLOW)Build Date:$(NC) $(BUILD_DATE)"
	@echo "$(YELLOW)Executable:$(NC) $(TARGET_DIR)/booth-organizer.exe"
	@echo "$(YELLOW)Package:$(NC) $(TARGET_DIR)/$(PACKAGE_NAME).tar.gz"
	@cd $(TARGET_DIR) && echo "$(YELLOW)Package Size:$(NC) $$(du -h $(PACKAGE_NAME).tar.gz | cut -f1)"

# Build everything
all: test package
	@echo "$(GREEN)All builds completed successfully!$(NC)"

# Clean build artifacts
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	npm run clean 2>/dev/null || true
	cd src-tauri && cargo clean
	rm -rf dist/
	rm -rf $(TARGET_DIR)/$(PORTABLE_DIR)
	rm -f $(TARGET_DIR)/$(PACKAGE_NAME).tar.gz
	@echo "$(GREEN)Clean completed$(NC)"

# Development helpers
dev-setup: install-deps
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@echo "$(GREEN)Development environment ready$(NC)"

# Quick build for development (debug mode)
dev-build:
	@echo "$(BLUE)Building development version...$(NC)"
	npm run build
	npx tauri build --debug
	@echo "$(GREEN)Development build completed$(NC)"

# Show build information
info:
	@echo "$(BLUE)Build Information$(NC)"
	@echo "================="
	@echo "$(YELLOW)Version:$(NC) $(VERSION)"
	@echo "$(YELLOW)Build Date:$(NC) $(BUILD_DATE)"
	@echo "$(YELLOW)Repository:$(NC) $(REPO_ROOT)"
	@echo "$(YELLOW)Target Directory:$(NC) $(TARGET_DIR)"
	@echo "$(YELLOW)Portable Directory:$(NC) $(PORTABLE_DIR)"
	@echo "$(YELLOW)Package Name:$(NC) $(PACKAGE_NAME)"
	@echo ""
	@echo "$(YELLOW)Node.js:$(NC) $$(node --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)npm:$(NC) $$(npm --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Rust:$(NC) $$(rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Cargo:$(NC) $$(cargo --version 2>/dev/null || echo 'Not installed')"

# Create portable version from existing executable
portable-only:
	@echo "$(BLUE)Creating portable version from existing executable...$(NC)"
	@if [ ! -f "$(TARGET_DIR)/booth-organizer.exe" ]; then \
		echo "$(RED)Error: booth-organizer.exe not found. Run 'make build-windows' first.$(NC)"; \
		exit 1; \
	fi
	@cd $(TARGET_DIR) && rm -rf $(PORTABLE_DIR)
	@cd $(TARGET_DIR) && mkdir -p $(PORTABLE_DIR)
	@cd $(TARGET_DIR) && cp booth-organizer.exe $(PORTABLE_DIR)/
	@cd $(TARGET_DIR) && cp *.dll $(PORTABLE_DIR)/
	@cd $(TARGET_DIR) && echo "BOOTH File Organizer - Windows Portable Edition" > $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "===============================================" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "バージョン: $(VERSION)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "ビルド日: $(BUILD_DATE)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【セットアップ】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "1. このフォルダ内のすべてのファイルを任意の場所にコピーしてください" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "2. booth-organizer.exe をダブルクリックして実行します" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【必要なファイル】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- booth-organizer.exe (メイン実行ファイル)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- booth_organizer_lib.dll (アプリケーションライブラリ)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- WebView2Loader.dll (Webビューコンポーネント)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【動作要件】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- Windows 10/11 x64" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- Microsoft Edge WebView2 Runtime (通常は既にインストール済み)" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【主な機能】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- BOOTHアセットの自動ダウンロード・整理" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- タグ管理機能" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ファイル検索・絞り込み" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- usage_count=0のタグ自動削除機能" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【新機能】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- タグが使用されなくなった時の自動削除" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ファイル削除・タグ削除時の自動クリーンアップ" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- データベースの最適化" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- モダンなタグUI" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- 改良されたタグ管理モーダル" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "【注意事項】" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- 初回起動時にWindows Defenderの警告が表示される場合があります" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- ポータブル版のため、設定は実行ファイルと同じフォルダに保存されます" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "- アンインストールは、このフォルダを削除するだけです" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "" >> $(PORTABLE_DIR)/README.txt
	@cd $(TARGET_DIR) && echo "お問い合わせ: GitHub Issues" >> $(PORTABLE_DIR)/README.txt
	@echo "$(GREEN)Portable version created$(NC)"
	@echo "$(YELLOW)Portable directory:$(NC) $(TARGET_DIR)/$(PORTABLE_DIR)"
	@cd $(TARGET_DIR) && ls -la $(PORTABLE_DIR)/

# Package existing portable version
package-only: portable-only
	@echo "$(BLUE)Packaging existing portable version...$(NC)"
	@cd $(TARGET_DIR) && rm -f $(PACKAGE_NAME).tar.gz
	@cd $(TARGET_DIR) && tar -czf $(PACKAGE_NAME).tar.gz $(PORTABLE_DIR)/
	@echo "$(GREEN)Package created successfully$(NC)"
	@echo "$(YELLOW)Package location:$(NC) $(TARGET_DIR)/$(PACKAGE_NAME).tar.gz"
	@cd $(TARGET_DIR) && ls -lh $(PACKAGE_NAME).tar.gz
	@echo ""
	@echo "$(GREEN)Build Summary:$(NC)"
	@echo "=============="
	@echo "$(YELLOW)Version:$(NC) $(VERSION)"
	@echo "$(YELLOW)Build Date:$(NC) $(BUILD_DATE)"
	@echo "$(YELLOW)Executable:$(NC) $(TARGET_DIR)/booth-organizer.exe"
	@echo "$(YELLOW)Package:$(NC) $(TARGET_DIR)/$(PACKAGE_NAME).tar.gz"
	@cd $(TARGET_DIR) && echo "$(YELLOW)Package Size:$(NC) $$(du -h $(PACKAGE_NAME).tar.gz | cut -f1)"

# Code quality targets
lint:
	@echo "$(BLUE)Running linter...$(NC)"
	cd src-tauri && cargo clippy -- -W clippy::all
	npm run lint 2>/dev/null || echo "Frontend linting not available"
	@echo "$(GREEN)Linting completed$(NC)"

format:
	@echo "$(BLUE)Formatting code...$(NC)"
	cd src-tauri && cargo fmt
	npm run format 2>/dev/null || echo "Frontend formatting not available"
	@echo "$(GREEN)Code formatted$(NC)"

# Install mingw-w64 (for Ubuntu/Debian)
install-mingw:
	@echo "$(BLUE)Installing mingw-w64...$(NC)"
	sudo apt update
	sudo apt install -y mingw-w64 gcc-mingw-w64-x86-64
	@echo "$(GREEN)mingw-w64 installed$(NC)"
// FileSearchSection - ファイル検索機能の責務を分離
// TDDでApp.tsxから抽出してリファクタリング

import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";


// 型定義
interface Tag {
  id?: number;
  name: string;
  color: string;
  category?: string;
  parent_tag_id?: number;
  usage_count: number;
  created_at?: string;
}

interface FileRecord {
  id?: number;
  file_path: string;
  file_name: string;
  file_size?: number;
  file_hash?: string;
  booth_product_id?: number;
  booth_shop_name?: string;
  booth_product_name?: string;
  booth_url?: string;
  booth_price?: number;
  booth_thumbnail_path?: string;
  encoding_info?: string;
  created_at?: string;
  updated_at?: string;
  metadata?: string;
}

interface FileWithTags {
  file: FileRecord;
  tags: Tag[];
}

interface GeneralModal {
  isOpen: boolean;
  type: 'url-edit' | 'tag-add' | 'tag-manage' | 'delete-confirm';
  title: string;
  fileId?: number;
  fileName?: string;
  currentValue?: string;
  placeholder?: string;
  onConfirm: (value: string) => void;
  onDelete?: (tagName: string) => void;
  existingTags?: Tag[];
}

export interface FileSearchSectionProps {
  availableTags: Tag[];
  onOpenFolder: (filePath: string) => void;
  onSetGeneralModal: (modal: GeneralModal) => void;
  onDeleteFile: (fileId: number) => void;
  onAddTagToSearchFile: (fileId: number, tagName: string) => void;
  onRemoveTagFromFile: (fileId: number, tagName: string) => void;
  onUpdateBoothUrl: (fileId: number, url: string) => void;
  onRefreshTags: () => void;
}

// カラーパレット定数
const COLOR_PALETTE = [
  "#007ACC", "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", 
  "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F", "#BB8FCE"
];

export function FileSearchSection({
  availableTags,
  onOpenFolder,
  onSetGeneralModal,
  onDeleteFile,
  onAddTagToSearchFile,
  onRemoveTagFromFile,
  onUpdateBoothUrl,
  onRefreshTags
}: FileSearchSectionProps) {
  // 検索関連の状態
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [searchResultsWithTags, setSearchResultsWithTags] = useState<FileWithTags[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  
  // バッチ操作関連の状態
  const [selectedSearchFileIds, setSelectedSearchFileIds] = useState<number[]>([]);
  const [isSearchBatchMode, setIsSearchBatchMode] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [newTagColor, setNewTagColor] = useState("#007ACC");

  // 検索実行
  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setSearchResultsWithTags([]);
      return;
    }

    setIsSearching(true);
    try {
      const results = await invoke<FileWithTags[]>('search_files_db', {
        query: searchQuery
      });
      console.log('Search results:', results);
      
      // 結果の検証とフィルタリング
      if (Array.isArray(results)) {
        const validResults = results.filter(item => 
          item && 
          item.file && 
          typeof item.file === 'object' &&
          item.file.file_name
        );
        setSearchResultsWithTags(validResults);
      } else {
        console.error('Invalid search results format:', results);
        setSearchResultsWithTags([]);
      }
    } catch (error) {
      console.error('検索エラー:', error);
      setSearchResultsWithTags([]);
    } finally {
      setIsSearching(false);
    }
  };


  // タグで検索を実行
  useEffect(() => {
    const searchByTags = async () => {
      if (selectedTags.length === 0 && !searchQuery.trim()) {
        setSearchResultsWithTags([]);
        return;
      }

      setIsSearching(true);
      try {
        let results: FileWithTags[] = [];
        
        // タグ検索とテキスト検索の組み合わせ対応
        if (selectedTags.length > 0) {
          console.log('=== Frontend: Calling search_files_by_tags_db ===');
          console.log('Selected tags:', selectedTags);
          console.log('Sending parameters:', { tagNames: selectedTags });
          
          const tagResults = await invoke<FileWithTags[]>('search_files_by_tags_db', {
            tagNames: selectedTags
          });
          
          console.log('Tag search results:', tagResults);
          
          // テキスト検索クエリがある場合は、タグ検索結果をさらにフィルタリング
          if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            results = tagResults.filter(item => 
              item && item.file &&
              (item.file.file_name?.toLowerCase().includes(query) ||
               item.file.booth_shop_name?.toLowerCase().includes(query) ||
               item.file.booth_product_name?.toLowerCase().includes(query))
            );
          } else {
            results = tagResults;
          }
        } else if (searchQuery.trim()) {
          // テキスト検索のみ
          await handleSearch();
          return; // handleSearchが結果をセットするので、ここで終了
        }
        
        // 結果が配列であることを確認してセット
        if (Array.isArray(results)) {
          const validResults = results.filter(item => 
            item && 
            item.file && 
            typeof item.file === 'object' &&
            item.file.file_name
          );
          setSearchResultsWithTags(validResults);
        } else {
          setSearchResultsWithTags([]);
        }
      } catch (error) {
        console.error('検索エラー:', error);
        setSearchResultsWithTags([]);
      } finally {
        setIsSearching(false);
      }
    };

    // 検索クエリまたは選択タグが変更されたときに実行
    const timeoutId = setTimeout(() => {
      searchByTags();
    }, 300); // デバウンス

    return () => clearTimeout(timeoutId);
  }, [selectedTags, searchQuery]);

  // バッチモードの切り替え
  const toggleSearchBatchMode = () => {
    setIsSearchBatchMode(!isSearchBatchMode);
    setSelectedSearchFileIds([]);
  };

  // ファイル選択の切り替え
  const toggleSearchFileSelection = (fileId: number) => {
    setSelectedSearchFileIds(prev => {
      if (prev.includes(fileId)) {
        return prev.filter(id => id !== fileId);
      } else {
        return [...prev, fileId];
      }
    });
  };

  // 全ファイル選択
  const selectAllSearchFiles = () => {
    const allFileIds = searchResultsWithTags
      .map(f => f.file.id)
      .filter((id): id is number => id !== undefined);
    setSelectedSearchFileIds(allFileIds);
  };

  // 選択解除
  const clearSearchSelection = () => {
    setSelectedSearchFileIds([]);
  };

  // バッチでタグを追加
  const searchBatchAddTag = async () => {
    if (!newTagName.trim() || selectedSearchFileIds.length === 0) {
      alert('⚠️ タグ名を入力し、対象ファイルを選択してください。');
      return;
    }

    const tagName = newTagName.trim();
    const fileCount = selectedSearchFileIds.length;
    
    // 処理開始のフィードバック
    alert(`🏷️ タグ追加処理開始\n\nタグ名: "${tagName}"\n対象ファイル: ${fileCount}件\n\n処理中...`);

    try {
      await invoke('batch_add_tag_to_files_db', {
        file_ids: selectedSearchFileIds,
        tag_name: tagName,
        tag_color: newTagColor
      });

      // 成功のフィードバック
      alert(`✅ タグ追加完了！\n\n追加したタグ: "${tagName}"\n対象ファイル: ${fileCount}件\n\n⚠️ 注意: 既にタグが付いているファイルは自動的にスキップされます。`);

      // 検索結果を更新
      if (searchQuery.trim() || selectedTags.length > 0) {
        if (selectedTags.length > 0) {
          const results = await invoke<FileWithTags[]>('search_files_by_tags_db', {
            tagNames: selectedTags
          });
          setSearchResultsWithTags(results);
        } else {
          await handleSearch();
        }
      }

      // タグリストを更新
      onRefreshTags();
      setNewTagName("");
      setSelectedSearchFileIds([]);
    } catch (error) {
      console.error('バッチタグ追加エラー:', error);
      alert(`❌ タグ追加に失敗しました\n\nエラー詳細: ${error}\n\nもう一度お試しください。`);
    }
  };

  // バッチでファイルを削除
  const searchBatchDeleteFiles = async () => {
    if (selectedSearchFileIds.length === 0) return;

    // モーダル削除確認を開く
    onSetGeneralModal({
      isOpen: true,
      type: 'delete-confirm',
      title: 'バッチ削除確認',
      fileName: `選択した${selectedSearchFileIds.length}個のファイル`,
      placeholder: '',
      onConfirm: async () => {
        try {
          await invoke('batch_delete_files_db', {
            fileIds: selectedSearchFileIds
          });

          // 検索結果から削除されたファイルを除外
          setSearchResultsWithTags(prev => 
            prev.filter(f => !selectedSearchFileIds.includes(f.file.id!))
          );
          setSelectedSearchFileIds([]);
          onRefreshTags();
          // モーダルを閉じる
          onSetGeneralModal({ isOpen: false, type: 'delete-confirm', title: '', onConfirm: () => {} });
          alert(`✅ ${selectedSearchFileIds.length}個のファイルを削除しました。`);
        } catch (error) {
          console.error('バッチ削除エラー:', error);
          // エラー時もモーダルを閉じる
          onSetGeneralModal({ isOpen: false, type: 'delete-confirm', title: '', onConfirm: () => {} });
          alert(`❌ ファイルの削除に失敗しました: ${error}`);
        }
      }
    });
  };

  // エラーをキャッチして画面全体が壊れるのを防ぐ
  try {
    return (
      <section className="search-section" style={{ position: 'relative', overflow: 'visible' }}>
        <h2>🔍 ファイル検索</h2>
      
      {/* 文字列検索 */}
      <div className="search-input-container">
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="ファイル名、ショップ名、商品名で検索..."
          className="search-input"
          onKeyPress={(e) => {
            if (e.key === 'Enter') {
              handleSearch();
            }
          }}
        />
        <button 
          onClick={handleSearch}
          disabled={isSearching || !searchQuery.trim()}
          className="search-btn"
        >
          {isSearching ? '検索中...' : '検索'}
        </button>
      </div>

      {/* タグ検索 */}
      <div className="tag-search-section">
        <h3>🏷️ タグで絞り込み (利用可能タグ: {availableTags.filter(tag => tag.usage_count > 0).length}件)</h3>
        
        
        <div className="tags-filter">
          {availableTags.filter(tag => tag.usage_count > 0).map((tag) => (
            <button
              key={tag.id}
              onClick={() => {
                setSelectedTags(prev => {
                  if (prev.includes(tag.name)) {
                    return prev.filter(t => t !== tag.name);
                  } else {
                    return [...prev, tag.name];
                  }
                });
              }}
              className={`tag-filter-btn ${
                selectedTags.includes(tag.name) ? 'selected' : ''
              }`}
              style={{ backgroundColor: tag.color }}
            >
              {tag.name} ({tag.usage_count})
            </button>
          ))}
        </div>
        {selectedTags.length > 0 && (
          <div className="selected-tags-info">
            選択中のタグ: {selectedTags.join(', ')}
            <button 
              onClick={() => setSelectedTags([])}
              className="clear-tags-btn"
            >
              クリア
            </button>
          </div>
        )}
      </div>

      {/* 検索結果 */}
      {searchResultsWithTags.length > 0 && (
        <div className="search-results" style={{ position: 'relative', zIndex: 1, overflow: 'hidden', maxHeight: '80vh', overflowY: 'auto' }}>
          <div className="search-results-header">
            <h3>📁 検索結果 ({searchResultsWithTags.length}件)</h3>
            <div className="search-batch-controls">
              <button
                onClick={toggleSearchBatchMode}
                className={`batch-mode-btn ${isSearchBatchMode ? 'active' : ''}`}
              >
                {isSearchBatchMode ? 'バッチモード終了' : 'バッチモード'}
              </button>
            </div>
          </div>
          
          {isSearchBatchMode && (
            <div className="batch-controls-panel">
              <div className="batch-selection-info">
                選択中: {selectedSearchFileIds.length}件
                <button onClick={selectAllSearchFiles} className="select-all-btn">
                  すべて選択
                </button>
                <button onClick={clearSearchSelection} className="clear-selection-btn">
                  選択解除
                </button>
              </div>
              
              <div className="batch-actions">
                <div className="batch-tag-add">
                  <input
                    type="text"
                    value={newTagName}
                    onChange={(e) => setNewTagName(e.target.value)}
                    placeholder="タグ名"
                    className="batch-tag-input"
                  />
                  <div className="color-palette">
                    {COLOR_PALETTE.map((color) => (
                      <button
                        key={color}
                        onClick={() => setNewTagColor(color)}
                        className={`color-btn ${newTagColor === color ? 'selected' : ''}`}
                        style={{ backgroundColor: color }}
                        title={color}
                      />
                    ))}
                  </div>
                  <button onClick={searchBatchAddTag} className="batch-add-tag-btn">
                    タグ一括追加
                  </button>
                </div>
                
                <button onClick={searchBatchDeleteFiles} className="batch-delete-btn">
                  選択ファイルを一括削除
                </button>
              </div>
            </div>
          )}
          
          <div className="search-results-list">
            {searchResultsWithTags.map((fileWithTags) => {
              // データの整合性チェック
              if (!fileWithTags || !fileWithTags.file) {
                console.error('Invalid fileWithTags data:', fileWithTags);
                return null;
              }
              
              return (
              <div key={fileWithTags.file.id || Math.random()} className={`search-result-item ${isSearchBatchMode ? 'batch-mode' : ''}`}>
                <div className="search-result-grid">
                  {/* 1列目: ファイル名とショップ情報 */}
                  <div className="file-info-column">
                    <div className="file-name-row">
                      {isSearchBatchMode && (
                        <input
                          type="checkbox"
                          checked={selectedSearchFileIds.includes(fileWithTags.file.id!)}
                          onChange={() => toggleSearchFileSelection(fileWithTags.file.id!)}
                          className="file-checkbox"
                        />
                      )}
                      <div className="file-name">{fileWithTags.file.file_name || 'Unknown File'}</div>
                    </div>
                    <div className="booth-info-row">
                      {fileWithTags.file.booth_shop_name && (
                        <span className="shop-name">🏦 {fileWithTags.file.booth_shop_name}</span>
                      )}
                      {fileWithTags.file.booth_product_name && (
                        <span className="product-name">📦 {fileWithTags.file.booth_product_name}</span>
                      )}
                      {fileWithTags.file.booth_url && (
                        <a href={fileWithTags.file.booth_url} target="_blank" rel="noopener noreferrer" className="booth-url-link">
                          🔗 URL
                        </a>
                      )}
                    </div>
                  </div>
                  
                  {/* 2列目: アクションボタン */}
                  {!isSearchBatchMode && (
                    <div className="actions-column">
                      <div className="actions-row-top">
                        <button
                          onClick={() => onOpenFolder(fileWithTags.file.file_path)}
                          className="open-folder-btn"
                          title="フォルダを開く"
                        >
                          開く
                        </button>
                        <button
                          onClick={() => onSetGeneralModal({
                            isOpen: true,
                            type: 'tag-manage',
                            title: 'タグ管理',
                            fileId: fileWithTags.file.id!,
                            fileName: fileWithTags.file.file_name,
                            placeholder: 'タグ名を入力してください',
                            existingTags: fileWithTags.tags,
                            onConfirm: async (value) => {
                              if (value.trim()) {
                                await onAddTagToSearchFile(fileWithTags.file.id!, value.trim());
                                // モーダルを閉じて再度開くことで最新のタグ情報を表示
                                setTimeout(() => {
                                  setSelectedTags([...selectedTags]);
                                }, 500);
                              }
                            },
                            onDelete: async (tagName) => {
                              await onRemoveTagFromFile(fileWithTags.file.id!, tagName);
                              // タグ削除後に検索結果を再実行
                              if (selectedTags.length > 0 || searchQuery.trim()) {
                                setSelectedTags([...selectedTags]);
                              }
                              // モーダルを閉じて更新された結果を反映
                              setTimeout(() => {
                                onSetGeneralModal({ isOpen: false, type: 'tag-add', title: '', onConfirm: () => {} });
                              }, 1000);
                            }
                          })}
                          className="tag-manage-btn"
                          title="タグ管理"
                        >
                          タグ
                        </button>
                      </div>
                      <div className="actions-row-bottom">
                        <button
                          onClick={() => onSetGeneralModal({
                            isOpen: true,
                            type: 'url-edit',
                            title: 'BOOTH URL編集',
                            fileId: fileWithTags.file.id!,
                            fileName: fileWithTags.file.file_name,
                            currentValue: fileWithTags.file.booth_url || '',
                            placeholder: 'https://shop.booth.pm/items/12345',
                            onConfirm: (value) => {
                              onUpdateBoothUrl(fileWithTags.file.id!, value);
                            }
                          })}
                          className="edit-url-btn"
                          title="URL編集"
                        >
                          URL編集
                        </button>
                        <button
                          onClick={() => {
                            // モーダル削除確認を開く
                            onSetGeneralModal({
                              isOpen: true,
                              type: 'delete-confirm',
                              title: 'ファイル削除確認',
                              fileId: fileWithTags.file.id!,
                              fileName: fileWithTags.file.file_name,
                              placeholder: '',
                              onConfirm: async () => {
                                try {
                                  await onDeleteFile(fileWithTags.file.id!);
                                  // 削除成功後、検索結果から該当ファイルを除外
                                  setSearchResultsWithTags(prev => 
                                    prev.filter(f => f.file.id !== fileWithTags.file.id)
                                  );
                                  onRefreshTags();
                                  // モーダルを閉じる
                                  onSetGeneralModal({ isOpen: false, type: 'delete-confirm', title: '', onConfirm: () => {} });
                                  alert('✅ ファイルを削除しました。');
                                } catch (error) {
                                  console.error('削除エラー:', error);
                                  // エラー時もモーダルを閉じる
                                  onSetGeneralModal({ isOpen: false, type: 'delete-confirm', title: '', onConfirm: () => {} });
                                  alert('❌ ファイルの削除に失敗しました: ' + error);
                                }
                              }
                            });
                          }}
                          className="delete-file-btn"
                          title="削除"
                        >
                          削除
                        </button>
                      </div>
                    </div>
                  )}
                  
                  {/* 3列目: タグ表示 */}
                  <div className="tags-column">
                    <div className="file-tags-container">
                      {fileWithTags.tags.length > 0 ? (
                        <div className="file-tags">
                          {fileWithTags.tags.map((tag, index) => (
                            <div
                              key={index}
                              className="file-tag-wrapper"
                              onClick={() => {
                                // タグクリックで絞り込み検索に追加
                                setSelectedTags(prev => {
                                  if (!prev.includes(tag.name)) {
                                    return [...prev, tag.name];
                                  }
                                  return prev;
                                });
                              }}
                              title={`「${tag.name}」で絞り込む`}
                            >
                              <span
                                className="file-tag modern-tag"
                                style={{ 
                                  '--tag-color': tag.color,
                                  backgroundColor: tag.color
                                } as React.CSSProperties}
                              >
                                <span className="tag-icon">🏷️</span>
                                <span className="tag-text">{tag.name}</span>
                                <span className="tag-shine"></span>
                              </span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div className="no-tags-message">
                          <span className="no-tags-icon">📝</span>
                          <span className="no-tags-text">タグなし</span>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </div>
              );
            }).filter(Boolean)}
          </div>
        </div>
      )}
    </section>
  );
  } catch (error) {
    console.error('FileSearchSection rendering error:', error);
    return (
      <section className="search-section">
        <h2>🔍 ファイル検索</h2>
        <div style={{ padding: '20px', color: 'red' }}>
          エラーが発生しました: {String(error)}
        </div>
      </section>
    );
  }
}

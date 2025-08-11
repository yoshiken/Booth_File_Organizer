// FileSyncSection - ファイル同期機能の責務を分離
// TDDでApp.tsxから抽出してリファクタリング

import { useState } from 'react';
import { invoke } from "@tauri-apps/api/core";

// 型定義
interface SyncResult {
  total_files: number;
  missing_files: MissingFile[];
  orphaned_files: number;
  updated_files: number;
}

interface MissingFile {
  id: number;
  file_name: string;
  file_path: string;
  booth_shop_name?: string;
  booth_product_name?: string;
}

interface DeleteConfirmModal {
  isOpen: boolean;
  type: 'single' | 'batch';
  fileCount: number;
  fileName?: string;
  onConfirm: () => void;
}

export interface FileSyncSectionProps {
  onSetDeleteConfirmModal: (modal: DeleteConfirmModal) => void;
}

export function FileSyncSection({ onSetDeleteConfirmModal }: FileSyncSectionProps) {
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [selectedMissingFiles, setSelectedMissingFiles] = useState<number[]>([]);

  // ファイル同期機能
  const handleSyncFileSystem = async () => {
    setIsSyncing(true);
    try {
      const result = await invoke<SyncResult>('sync_file_system_db');
      setSyncResult(result);
    } catch (error) {
      console.error('Sync failed:', error);
      alert('ファイル同期に失敗しました: ' + String(error));
    } finally {
      setIsSyncing(false);
    }
  };

  const handleRemoveMissingFiles = async () => {
    if (selectedMissingFiles.length === 0) {
      alert('削除するファイルを選択してください。');
      return;
    }

    onSetDeleteConfirmModal({
      isOpen: true,
      type: 'batch',
      fileCount: selectedMissingFiles.length,
      onConfirm: async () => {
        try {
          const removedCount = await invoke<number>('remove_missing_files_db', {
            fileIds: selectedMissingFiles
          });
          
          // 同期結果を更新
          if (syncResult) {
            const updatedMissingFiles = syncResult.missing_files.filter(
              file => !selectedMissingFiles.includes(file.id)
            );
            setSyncResult({
              ...syncResult,
              missing_files: updatedMissingFiles
            });
          }
          
          setSelectedMissingFiles([]);
          alert(`${removedCount}個のファイル情報をデータベースから削除しました。`);
        } catch (error) {
          console.error('Remove missing files failed:', error);
          alert('ファイル情報の削除に失敗しました: ' + String(error));
        }
      }
    });
  };

  const toggleMissingFileSelection = (fileId: number) => {
    setSelectedMissingFiles(prev => 
      prev.includes(fileId) 
        ? prev.filter(id => id !== fileId)
        : [...prev, fileId]
    );
  };

  const selectAllMissingFiles = () => {
    if (syncResult) {
      setSelectedMissingFiles(syncResult.missing_files.map(f => f.id));
    }
  };

  const clearMissingFileSelection = () => {
    setSelectedMissingFiles([]);
  };

  return (
    <>
      {/* ファイル同期セクション */}
      <section className="sync-section">
        <h2>🔄 ファイル同期</h2>
        <p>データベースに登録されているファイルと実際のファイルシステムの同期状態を確認します。</p>
        
        <div className="sync-controls">
          <button
            onClick={handleSyncFileSystem}
            disabled={isSyncing}
            className="sync-btn"
          >
            {isSyncing ? '同期中...' : '同期状態をチェック'}
          </button>
        </div>

        {syncResult && (
          <div className="sync-results">
            <div className="sync-summary">
              <h3>同期結果</h3>
              <div className="sync-stats">
                <div className="stat-item">
                  <span className="stat-label">総ファイル数:</span>
                  <span className="stat-value">{syncResult.total_files}</span>
                </div>
                <div className="stat-item">
                  <span className="stat-label">見つからないファイル:</span>
                  <span className="stat-value error">{syncResult.missing_files.length}</span>
                </div>
              </div>
            </div>

            {syncResult.missing_files.length > 0 && (
              <div className="missing-files-section">
                <div className="missing-files-header">
                  <h4>⚠️ 見つからないファイル ({syncResult.missing_files.length}件)</h4>
                  <div className="missing-files-controls">
                    <button onClick={selectAllMissingFiles} className="select-all-btn">
                      すべて選択
                    </button>
                    <button onClick={clearMissingFileSelection} className="clear-selection-btn">
                      選択解除
                    </button>
                    <button 
                      onClick={handleRemoveMissingFiles} 
                      className="remove-missing-btn"
                      disabled={selectedMissingFiles.length === 0}
                    >
                      選択ファイルをデータベースから削除
                    </button>
                  </div>
                </div>
                
                <div className="missing-files-list">
                  {syncResult.missing_files.map((file) => (
                    <div key={file.id} className="missing-file-item">
                      <div className="missing-file-info">
                        <input
                          type="checkbox"
                          checked={selectedMissingFiles.includes(file.id)}
                          onChange={() => toggleMissingFileSelection(file.id)}
                          className="file-checkbox"
                        />
                        <div className="file-details">
                          <div className="file-name">📁 {file.file_name}</div>
                          <div className="file-path">{file.file_path}</div>
                          {(file.booth_shop_name || file.booth_product_name) && (
                            <div className="booth-info">
                              {file.booth_shop_name && <span>🏦 {file.booth_shop_name}</span>}
                              {file.booth_product_name && <span>📦 {file.booth_product_name}</span>}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {syncResult.missing_files.length === 0 && (
              <div className="sync-success">
                ✅ すべてのファイルが正常に同期されています。
              </div>
            )}
          </div>
        )}
      </section>
    </>
  );
}
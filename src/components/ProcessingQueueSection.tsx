// ProcessingQueueSection - ファイル処理キューの責務を分離
// TDDでApp.tsxから抽出してリファクタリング

import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

// 型定義
interface ProcessingFile {
  id: string;
  name: string;
  status: 'pending' | 'processing' | 'completed' | 'error';
  boothUrl?: string;
  shopName?: string;
  productName?: string;
  progress: number;
  error?: string;
  fullPath?: string;
  tags?: string[];
  tagInput?: string;
}

interface BoothProductInfo {
  shop_name: string;
  product_name: string;
  tags: string[];
}

interface Tag {
  id: number;
  name: string;
  color: string;
  category: string | null;
  parent_tag_id: number | null;
  usage_count: number;
  created_at: string;
}

export interface ProcessingQueueSectionProps {
  boothUrl: string;
  outputDir: string | null;
  boothProductInfo: BoothProductInfo | null;
  onLoadAvailableTags: () => Promise<void>;
}

export function ProcessingQueueSection({
  boothUrl,
  outputDir,
  boothProductInfo,
  onLoadAvailableTags
}: ProcessingQueueSectionProps) {
  const [files, setFiles] = useState<ProcessingFile[]>([]);
  const [popularTags, setPopularTags] = useState<Tag[]>([]);

  useEffect(() => {
    loadPopularTags();
  }, []);

  const loadPopularTags = async () => {
    try {
      const tags = await invoke<Tag[]>('get_all_tags_from_db');
      // 上位10件の使用頻度の高いタグを取得
      setPopularTags(tags.slice(0, 10));
    } catch (error) {
      console.error('Failed to load popular tags:', error);
    }
  };

  const processFiles = async () => {
    const pendingFiles = files.filter(f => f.status === 'pending');
    let hasAnySuccess = false;
    
    for (const file of pendingFiles) {
      setFiles(prev => prev.map(f => 
        f.id === file.id ? { ...f, status: 'processing' } : f
      ));
      
      try {
        const progressInterval = setInterval(() => {
          setFiles(prev => prev.map(f => {
            if (f.id === file.id && f.status === 'processing') {
              const newProgress = Math.min(f.progress + 10, 90);
              return { ...f, progress: newProgress };
            }
            return f;
          }));
        }, 200);

        const result = await invoke<{
          success: boolean;
          message?: string;
          shop_name?: string;
          product_name?: string;
        }>('process_zip_file', {
          zipPath: file.fullPath || file.name,
          boothUrl: boothUrl || null,
          outputDir: outputDir,
          tags: file.tags || null,
        });

        clearInterval(progressInterval);

        if (result.success) {
          setFiles(prev => prev.map(f => 
            f.id === file.id ? { 
              ...f, 
              status: 'completed', 
              progress: 100,
              shopName: result.shop_name,
              productName: result.product_name
            } : f
          ));
          
          hasAnySuccess = true;
        } else {
          setFiles(prev => prev.map(f => 
            f.id === file.id ? { 
              ...f, 
              status: 'error', 
              error: result.message 
            } : f
          ));
        }
      } catch (error) {
        setFiles(prev => prev.map(f => 
          f.id === file.id ? { ...f, status: 'error', error: String(error) } : f
        ));
      }
    }
    
    if (hasAnySuccess) {
      await onLoadAvailableTags();
    }
  };

  const clearFiles = () => {
    setFiles([]);
  };

  const handleFileSelect = async () => {
    try {
      const result = await invoke<{
        success: boolean;
        files: string[];
      }>('select_zip_files');
      
      if (result.success && result.files.length > 0) {
        const newFiles: ProcessingFile[] = result.files.map((filePath: string) => {
          const fileName = filePath.split(/[/\\]/).pop() || filePath;
          return {
            id: Date.now() + Math.random().toString(),
            name: fileName,
            status: 'pending',
            progress: 0,
            fullPath: filePath,
            tags: []
          };
        });
        
        setFiles(prev => [...prev, ...newFiles]);
      }
    } catch (error) {
      console.error('File selection failed:', error);
      alert('ファイルの選択に失敗しました: ' + String(error));
    }
  };

  return (
    <div className="file-processing-container">
      <section className="file-select-section">
        <h2>📁 ファイル選択</h2>
        <button 
          onClick={handleFileSelect}
          className="file-select-btn"
        >
          ZIPファイルを選択
        </button>
      </section>

      <section className="processing-section">
        <div className="section-header">
          <h2>処理キュー ({files.length})</h2>
          <div className="action-buttons">
            {files.some(f => f.status === 'pending') && (
              <button 
                onClick={processFiles}
                className="process-btn"
              >
                処理開始
              </button>
            )}
            {files.length > 0 && (
              <button 
                onClick={clearFiles}
                className="clear-btn"
              >
                クリア
              </button>
            )}
          </div>
        </div>
        
        <div className="file-list">
          {files.map((file) => (
            <div key={file.id} className={`file-item status-${file.status}`}>
              <div className="file-info">
                <div className="file-name">{file.name}</div>
                {file.shopName && file.productName && (
                  <div className="file-metadata">
                    {file.shopName} / {file.productName}
                  </div>
                )}
                <div className="file-status">
                  {file.status === 'pending' && '⏳ 待機中'}
                  {file.status === 'processing' && '⚙️ 処理中'}
                  {file.status === 'completed' && '✅ 完了'}
                  {file.status === 'error' && '❌ エラー'}
                </div>
              </div>
              
              {(file.status === 'pending' || file.status === 'completed') && (
                <div className="queue-tag-section">
                  <div className="queue-add-tag">
                    <input
                      type="text"
                      placeholder="タグ名（カンマ区切りで複数指定可能）"
                      value={file.tagInput || ''}
                      onChange={(e) => {
                        const tagInput = e.target.value;
                        const tags = tagInput.split(',').map(t => t.trim()).filter(t => t);
                        setFiles(prev => prev.map(f => 
                          f.id === file.id ? { ...f, tagInput, tags } : f
                        ));
                      }}
                      className="queue-tag-input"
                    />
                    
                    {/* タグ候補表示 */}
                    <div className="tag-suggestions">
                      {/* 商品情報からのタグ候補 */}
                      {boothProductInfo && boothProductInfo.tags && boothProductInfo.tags.length > 0 && (
                        <>
                          <div className="tag-suggestions-label">🎯 商品ページから取得したタグ：</div>
                          <div className="tag-suggestions-list">
                            {boothProductInfo.tags.map((suggestedTag, index) => (
                              <button
                                key={`booth-${index}`}
                                onClick={() => {
                                  const currentTags = file.tagInput ? file.tagInput.split(',').map(t => t.trim()).filter(t => t) : [];
                                  if (!currentTags.includes(suggestedTag)) {
                                    const newTagInput = currentTags.length > 0 
                                      ? `${file.tagInput}, ${suggestedTag}`
                                      : suggestedTag;
                                    const newTags = newTagInput.split(',').map(t => t.trim()).filter(t => t);
                                    setFiles(prev => prev.map(f => 
                                      f.id === file.id ? { ...f, tagInput: newTagInput, tags: newTags } : f
                                    ));
                                  }
                                }}
                                className={`tag-suggestion-btn ${
                                  file.tagInput && file.tagInput.split(',').map(t => t.trim()).includes(suggestedTag) 
                                    ? 'selected' : ''
                                }`}
                                disabled={!!(file.tagInput && file.tagInput.split(',').map(t => t.trim()).includes(suggestedTag))}
                              >
                                {suggestedTag}
                              </button>
                            ))}
                          </div>
                        </>
                      )}
                      
                      {/* 使用頻度の高いタグ候補 */}
                      {popularTags.length > 0 && (
                        <>
                          <div className="tag-suggestions-label" style={{ marginTop: '10px' }}>🔥 よく使われるタグ：</div>
                          <div className="tag-suggestions-list">
                            {popularTags.map((tag) => (
                              <button
                                key={`popular-${tag.id}`}
                                onClick={() => {
                                  const currentTags = file.tagInput ? file.tagInput.split(',').map(t => t.trim()).filter(t => t) : [];
                                  if (!currentTags.includes(tag.name)) {
                                    const newTagInput = currentTags.length > 0 
                                      ? `${file.tagInput}, ${tag.name}`
                                      : tag.name;
                                    const newTags = newTagInput.split(',').map(t => t.trim()).filter(t => t);
                                    setFiles(prev => prev.map(f => 
                                      f.id === file.id ? { ...f, tagInput: newTagInput, tags: newTags } : f
                                    ));
                                  }
                                }}
                                className={`tag-suggestion-btn ${
                                  file.tagInput && file.tagInput.split(',').map(t => t.trim()).includes(tag.name) 
                                    ? 'selected' : ''
                                }`}
                                disabled={!!(file.tagInput && file.tagInput.split(',').map(t => t.trim()).includes(tag.name))}
                                title={`使用回数: ${tag.usage_count}`}
                              >
                                {tag.name} ({tag.usage_count})
                              </button>
                            ))}
                          </div>
                        </>
                      )}
                      
                      <div className="tag-suggestions-help">
                        💡 クリックでタグを追加できます
                      </div>
                    </div>
                    
                    <span className="queue-tag-hint">
                      手動入力またはタグ候補をクリックして追加
                    </span>
                  </div>
                </div>
              )}
              
              {file.status === 'processing' && (
                <div className="progress-bar">
                  <div 
                    className="progress-fill"
                    style={{ width: `${file.progress}%` }}
                  />
                </div>
              )}
              
              {file.error && (
                <div className="error-message">{file.error}</div>
              )}
            </div>
          ))}
          
          {files.length === 0 && (
            <div className="empty-state">
              まだファイルがありません。左側でアーカイブファイルを選択してください。
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
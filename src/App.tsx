import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { UrlInputSection, BoothProductInfo } from "./components/UrlInputSection";
import { FileSearchSection } from "./components/FileSearchSection";
import { ProcessingQueueSection } from "./components/ProcessingQueueSection";
import { FileSyncSection } from "./components/FileSyncSection";


interface Tag {
  id?: number;
  name: string;
  color: string;
  category?: string;
  parent_tag_id?: number;
  usage_count: number;
  created_at?: string;
}





// 汎用入力モーダルコンポーネント
interface GeneralInputModalProps {
  isOpen: boolean;
  type: 'url-edit' | 'tag-add' | 'tag-manage' | 'delete-confirm';
  title: string;
  fileName?: string;
  currentValue?: string;
  placeholder?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
  onDelete?: (tagName: string) => void;
  existingTags?: Tag[];
}

function GeneralInputModal({
  isOpen,
  type,
  title,
  fileName,
  currentValue = '',
  placeholder = '',
  onConfirm,
  onDelete,
  existingTags = [],
  onCancel
}: GeneralInputModalProps) {
  const [inputValue, setInputValue] = useState(currentValue);
  const [isProcessing, setIsProcessing] = useState(false);
  const [processingAction, setProcessingAction] = useState<string>('');
  const [successMessage, setSuccessMessage] = useState<string>('');

  // モーダルが開いたときに初期値を設定
  useEffect(() => {
    if (isOpen) {
      setInputValue(currentValue);
    }
  }, [isOpen, currentValue]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isProcessing) return;
    
    setIsProcessing(true);
    setProcessingAction('追加中...');
    setSuccessMessage('');
    
    try {
      await new Promise(resolve => {
        onConfirm(inputValue);
        setTimeout(resolve, 500); // 少し待ってからフィードバック表示
      });
      
      setSuccessMessage('✅ タグを追加しました！');
      setInputValue(''); // 入力欄をクリア
      
      setTimeout(() => {
        setSuccessMessage('');
        setIsProcessing(false);
      }, 1500);
    } catch (error) {
      setIsProcessing(false);
      setProcessingAction('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onCancel();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onCancel} style={{ display: isOpen ? 'flex' : 'none' }}>
      <div className="modal-content general-input-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>
            {type === 'url-edit' ? '🔗' : type === 'tag-manage' ? '🏷️' : type === 'delete-confirm' ? '🗑️' : '🏷️'} {title}
          </h3>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {fileName && (
              <div className="file-name-display">
                📁 {fileName}
              </div>
            )}
            {type !== 'delete-confirm' && (
              <div className="input-group">
                <label htmlFor="modal-input">
                  {type === 'url-edit' ? 'BOOTH URL:' : type === 'tag-manage' ? '新しいタグ名:' : 'タグ名:'}
                </label>
                <input
                  id="modal-input"
                  type={type === 'url-edit' ? 'url' : 'text'}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  placeholder={placeholder}
                  className="modal-input"
                  onKeyDown={handleKeyDown}
                  autoFocus
                  required={type === 'tag-add'}
                />
              </div>
            )}
            {type === 'delete-confirm' && (
              <div className="delete-confirm-message">
                <p style={{ fontSize: '16px', color: '#d32f2f', margin: '20px 0' }}>
                  ⚠️ 本当に削除しますか？
                </p>
                <p style={{ fontSize: '14px', color: '#666', margin: '10px 0' }}>
                  この操作は取り消せません。
                </p>
              </div>
            )}
            {type === 'url-edit' && (
              <div className="help-text">
                <p>💡 正しいBOOTH商品ページのURLを入力してください</p>
                <p>例: https://shop.booth.pm/items/12345</p>
              </div>
            )}
            {type === 'tag-add' && (
              <div className="help-text">
                <p>💡 半角英数字、ひらがな、カタカナ、漢字が使用できます</p>
              </div>
            )}
            {type === 'tag-manage' && (
              <div className="existing-tags-section">
                <h4>現在のタグ:</h4>
                <div className="existing-tags-list">
                  {existingTags.map((tag, index) => (
                    <div key={index} className="existing-tag-item">
                      <span className="tag-name" style={{ backgroundColor: tag.color }}>
                        {tag.name}
                      </span>
                      <button
                        type="button"
                        onClick={async () => {
                          if (!onDelete || isProcessing) return;
                          
                          setIsProcessing(true);
                          setProcessingAction(`「${tag.name}」を削除中...`);
                          setSuccessMessage('');
                          
                          try {
                            await new Promise(resolve => {
                              onDelete(tag.name);
                              setTimeout(resolve, 500);
                            });
                            
                            setSuccessMessage(`🗑️ タグ「${tag.name}」を削除しました！`);
                            
                            setTimeout(() => {
                              setSuccessMessage('');
                              setIsProcessing(false);
                            }, 1500);
                          } catch (error) {
                            setIsProcessing(false);
                            setProcessingAction('');
                          }
                        }}
                        className="delete-tag-btn"
                        title="タグを削除"
                        disabled={isProcessing}
                      >
                        {isProcessing && processingAction.includes(tag.name) ? '⏳' : '×'}
                      </button>
                    </div>
                  ))}
                </div>
                <div className="help-text">
                  <p>💡 新しいタグを追加するか、既存のタグを削除できます</p>
                </div>
              </div>
            )}
            
            {/* フィードバックメッセージ表示エリア */}
            {(isProcessing || successMessage) && (
              <div className="feedback-area">
                {isProcessing && (
                  <div className="processing-message">
                    ⏳ {processingAction}
                  </div>
                )}
                {successMessage && (
                  <div className="success-message">
                    {successMessage}
                  </div>
                )}
              </div>
            )}
          </div>
          <div className="modal-footer">
            <button
              type="button"
              onClick={onCancel}
              className="cancel-btn"
            >
              {type === 'tag-manage' ? '閉じる' : 'キャンセル'}
            </button>
            <button
              type="submit"
              className={type === 'delete-confirm' ? 'delete-confirm-btn' : 'confirm-btn'}
              disabled={(type === 'tag-add' && !inputValue.trim()) || isProcessing}
            >
              {isProcessing && processingAction.includes('追加') ? (
                '⏳ 追加中...'
              ) : type === 'delete-confirm' ? (
                '削除する'
              ) : (
                type === 'url-edit' ? 'URL更新' : type === 'tag-manage' ? 'タグ追加' : 'タグ追加'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function App() {
  const [boothUrl, setBoothUrl] = useState("");
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [availableTags, setAvailableTags] = useState<Tag[]>([]);
  const [isValidatingUrl, setIsValidatingUrl] = useState(false);
  const [urlValidationResult, setUrlValidationResult] = useState<'valid' | 'invalid' | null>(null);
  const [boothProductInfo, setBoothProductInfo] = useState<BoothProductInfo | null>(null);
  
  
  // 削除確認モーダル用の状態
  const [deleteConfirmModal, setDeleteConfirmModal] = useState<{
    isOpen: boolean;
    type: 'single' | 'batch';
    fileCount: number;
    fileName?: string;
    onConfirm: () => void;
  }>({
    isOpen: false,
    type: 'single',
    fileCount: 0,
    onConfirm: () => {}
  });

  // 汎用モーダル用の状態
  const [generalModal, setGeneralModal] = useState<{
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
  }>({
    isOpen: false,
    type: 'url-edit',
    title: '',
    onConfirm: () => {}
  });

  // デバッグ用：モーダルの状態をログ出力
  useEffect(() => {
    console.log('General modal state:', generalModal.isOpen);
  }, [generalModal.isOpen]);

  


  const handleSelectOutputFolder = async () => {
    try {
      const result = await invoke<string | null>('select_output_folder');
      if (result) {
        setOutputDir(result);
        await invoke('save_output_folder', { outputFolder: result });
      }
    } catch (error) {
      console.error('フォルダ選択エラー:', error);
      alert('フォルダの選択に失敗しました: ' + error);
    }
  };

  const loadSavedOutputFolder = async () => {
    try {
      const result = await invoke<string | null>('load_output_folder');
      if (result) {
        setOutputDir(result);
      }
    } catch (error) {
      console.error('保存済みフォルダの読み込みエラー:', error);
    }
  };

  useEffect(() => {
    loadAvailableTags();
    loadSavedOutputFolder();
  }, []);

  const loadAvailableTags = async () => {
    try {
      const result = await invoke<Tag[]>('get_all_tags_from_db');
      setAvailableTags(result);
    } catch (error) {
      console.error('Failed to load tags:', error);
    }
  };

  const validateBoothUrl = async () => {
    if (!boothUrl.trim()) {
      setUrlValidationResult(null);
      setBoothProductInfo(null);
      return;
    }

    setIsValidatingUrl(true);
    setUrlValidationResult(null);
    setBoothProductInfo(null);

    try {
      // まず基本的なURL形式をチェック
      const isValidFormat = await invoke<boolean>('validate_booth_url', { url: boothUrl.trim() });
      
      if (!isValidFormat) {
        setUrlValidationResult('invalid');
        return;
      }
      
      // 商品情報を実際に取得してみる
      const productInfo = await invoke<BoothProductInfo>('fetch_booth_product_info', { 
        url: boothUrl.trim() 
      });
      
      // 商品情報が正しく取得できた場合のみvalid
      if (productInfo && productInfo.shop_name && productInfo.product_name) {
        console.log('商品情報を取得しました:', productInfo);
        console.log('タグ情報:', productInfo.tags);
        setUrlValidationResult('valid');
        setBoothProductInfo(productInfo);
      } else {
        setUrlValidationResult('invalid');
      }
    } catch (error) {
      console.error('URL validation error:', error);
      setUrlValidationResult('invalid');
    } finally {
      setIsValidatingUrl(false);
    }
  };




  const handleOpenFolder = async (filePath: string) => {
    try {
      await invoke('open_folder', { folderPath: filePath });
    } catch (error) {
      console.error('Failed to open folder:', error);
      alert('フォルダを開くことができませんでした: ' + String(error));
    }
  };



  const addTagToSearchFile = async (fileId: number, tagName: string, tagColor?: string) => {
    if (!tagName.trim()) return;
    
    try {
      await invoke('add_tag_to_file_db', {
        fileId,
        tagName: tagName.trim(),
        tagColor
      });
      
      await loadAvailableTags();
      alert(`タグ「${tagName}」を追加しました。`);
    } catch (error) {
      console.error('Failed to add tag:', error);
      alert('タグの追加に失敗しました: ' + error);
    }
  };

  const removeTagFromFile = async (fileId: number, tagName: string) => {
    try {
      await invoke('remove_tag_from_file_db', {
        fileId,
        tagName
      });
      await loadAvailableTags();
      alert(`タグ「${tagName}」を削除しました。`);
    } catch (error) {
      console.error('Failed to remove tag:', error);
      alert('タグの削除に失敗しました: ' + error);
    }
  };

  const deleteFile = async (fileId: number) => {
    try {
      const success = await invoke<boolean>('delete_file_and_folder', {
        fileId
      });
      
      if (success) {
        await loadAvailableTags();
      } else {
        throw new Error('削除処理が失敗しました');
      }
    } catch (error) {
      console.error('Delete file failed:', error);
      throw error;
    }
  };

  const updateBoothUrl = async (fileId: number, boothUrl: string) => {
    try {
      await invoke('update_file_booth_url_db', {
        fileId,
        boothUrl: boothUrl.trim() || null
      });
    } catch (error) {
      console.error('Failed to update BOOTH URL:', error);
      throw error;
    }
  };





  return (
    <main className="container">
      <header>
        <h1>📦 BOOTH File Organizer</h1>
        <p>VRChat用BOOTHアセットの自動整理ツール</p>
      </header>

      <UrlInputSection
        boothUrl={boothUrl}
        urlValidationResult={urlValidationResult}
        isValidatingUrl={isValidatingUrl}
        boothProductInfo={boothProductInfo}
        onUrlChange={(url) => {
          setBoothUrl(url);
          setUrlValidationResult(null);
          setBoothProductInfo(null);
        }}
        onValidateUrl={validateBoothUrl}
      />

      <section className="output-dir-section">
        <h2>出力先フォルダ</h2>
        <div className="output-dir-container">
          <div className="output-dir-display">
            {outputDir ? (
              <>
                <span className="folder-icon">📁</span>
                <span className="folder-path">{outputDir}</span>
              </>
            ) : (
              <span className="placeholder">デフォルト: デスクトップ/BOOTH_Organized</span>
            )}
          </div>
          <button 
            className="select-folder-btn"
            onClick={handleSelectOutputFolder}
          >
            フォルダを選択
          </button>
        </div>
      </section>

      <ProcessingQueueSection
        boothUrl={boothUrl}
        outputDir={outputDir}
        boothProductInfo={boothProductInfo}
        onLoadAvailableTags={loadAvailableTags}
      />


      <FileSearchSection
        availableTags={availableTags}
        onOpenFolder={handleOpenFolder}
        onSetGeneralModal={setGeneralModal}
        onDeleteFile={deleteFile}
        onAddTagToSearchFile={addTagToSearchFile}
        onRemoveTagFromFile={removeTagFromFile}
        onUpdateBoothUrl={updateBoothUrl}
        onRefreshTags={loadAvailableTags}
      />

      <FileSyncSection
        onSetDeleteConfirmModal={setDeleteConfirmModal}
      />

      {/* 削除確認モーダル */}
      {deleteConfirmModal.isOpen && (
        <div className="modal-overlay" style={{ display: deleteConfirmModal.isOpen ? 'flex' : 'none' }}>
          <div className="modal-content delete-confirm-modal">
            <div className="modal-header">
              <h3>⚠️ 削除の確認</h3>
            </div>
            <div className="modal-body">
              {deleteConfirmModal.type === 'single' ? (
                <div>
                  <p>以下のファイルを削除しますか？</p>
                  {deleteConfirmModal.fileName && (
                    <div className="file-name-display">
                      📁 {deleteConfirmModal.fileName}
                    </div>
                  )}
                  <div className="warning-text">
                    <p>⚠️ この操作により以下が削除されます：</p>
                    <ul>
                      <li>データベースからのファイル情報</li>
                      <li>関連するフォルダとサムネイル</li>
                      <li>付与されたタグ情報</li>
                    </ul>
                    <p><strong>この操作は取り消せません。</strong></p>
                  </div>
                </div>
              ) : (
                <div>
                  <p>選択された <strong>{deleteConfirmModal.fileCount}</strong> 個のファイルを削除しますか？</p>
                  <div className="warning-text">
                    <p>⚠️ この操作により以下が削除されます：</p>
                    <ul>
                      <li>データベースからのファイル情報</li>
                      <li>関連するフォルダとサムネイル</li>
                      <li>付与されたタグ情報</li>
                    </ul>
                    <p><strong>この操作は取り消せません。</strong></p>
                  </div>
                </div>
              )}
            </div>
            <div className="modal-footer">
              <button
                onClick={() => setDeleteConfirmModal(prev => ({ ...prev, isOpen: false }))}
                className="cancel-btn"
              >
                キャンセル
              </button>
              <button
                onClick={deleteConfirmModal.onConfirm}
                className="delete-confirm-btn"
              >
                削除する
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 汎用入力モーダル */}
      {generalModal.isOpen && (
        <GeneralInputModal
          isOpen={generalModal.isOpen}
          type={generalModal.type}
          title={generalModal.title}
          fileName={generalModal.fileName}
          currentValue={generalModal.currentValue}
          placeholder={generalModal.placeholder}
          onConfirm={generalModal.onConfirm}
          onCancel={() => setGeneralModal(prev => ({ ...prev, isOpen: false }))}
          onDelete={generalModal.onDelete}
          existingTags={generalModal.existingTags}
        />
      )}
    </main>
  );
}

export default App;
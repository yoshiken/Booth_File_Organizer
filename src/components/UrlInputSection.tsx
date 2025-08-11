// UrlInputSection - BOOTH URL入力と商品情報表示の責務を分離
// TDDでリファクタリング実施

import React from 'react';

export interface BoothProductInfo {
  product_id?: number;
  shop_name: string;
  product_name: string;
  price?: number;
  description?: string;
  thumbnail_url?: string;
  is_free: boolean;
  tags: string[];
}

export interface UrlInputSectionProps {
  boothUrl: string;
  urlValidationResult: 'valid' | 'invalid' | null;
  isValidatingUrl: boolean;
  boothProductInfo: BoothProductInfo | null;
  onUrlChange: (url: string) => void;
  onValidateUrl: () => void;
}

export function UrlInputSection({
  boothUrl,
  urlValidationResult,
  isValidatingUrl,
  boothProductInfo,
  onUrlChange,
  onValidateUrl
}: UrlInputSectionProps) {
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onUrlChange(e.target.value);
  };

  return (
    <section className="url-input-section">
      <h2>BOOTH商品URL</h2>
      <div className="url-input-container">
        <input
          type="url"
          value={boothUrl}
          onChange={handleInputChange}
          placeholder="https://shop.booth.pm/items/12345"
          className={`url-input ${urlValidationResult === 'valid' ? 'valid' : urlValidationResult === 'invalid' ? 'invalid' : ''}`}
        />
        <button 
          className="url-validate-btn"
          onClick={onValidateUrl}
          disabled={isValidatingUrl || !boothUrl.trim()}
        >
          {isValidatingUrl ? '検証中...' : '検証'}
        </button>
      </div>
      
      {urlValidationResult === 'valid' && (
        <div className="url-validation-success">
          ✅ 有効なBOOTH URLです
        </div>
      )}
      
      {urlValidationResult === 'invalid' && (
        <div className="url-validation-error">
          ❌ 無効なBOOTH URLまたは商品情報を取得できません。<br/>
          正しい商品ページのURLを入力してください。<br/>
          <small>例: https://shop.booth.pm/items/12345</small>
        </div>
      )}

      {boothProductInfo && (
        <div className="booth-product-info">
          <h3>商品情報</h3>
          <div className="product-info-grid">
            <div className="product-info-item">
              <label>ショップ名:</label>
              <span>{boothProductInfo.shop_name}</span>
            </div>
            <div className="product-info-item">
              <label>商品名:</label>
              <span>{boothProductInfo.product_name}</span>
            </div>
            {boothProductInfo.price !== undefined && (
              <div className="product-info-item">
                <label>価格:</label>
                <span>{boothProductInfo.is_free ? '無料' : `¥${boothProductInfo.price?.toLocaleString()}`}</span>
              </div>
            )}
            <div className="product-info-item">
              <label>取得したタグ:</label>
              <div className="product-tags">
                {boothProductInfo.tags && boothProductInfo.tags.length > 0 ? (
                  boothProductInfo.tags.map((tag, index) => (
                    <span
                      key={index}
                      className="product-tag"
                    >
                      {tag}
                    </span>
                  ))
                ) : (
                  <span className="no-tags">タグが見つかりませんでした</span>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
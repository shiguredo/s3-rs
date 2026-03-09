# s3cli にメタデータ系オプションを追加する

## 概要

cp / mv / sync サブコマンドにメタデータ関連のオプションを追加する。

## オプション

- `--cache-control` — Cache-Control ヘッダー
- `--content-disposition` — Content-Disposition ヘッダー
- `--content-encoding` — Content-Encoding ヘッダー
- `--content-language` — Content-Language ヘッダー
- `--expires` — Expires ヘッダー
- `--content-type` — Content-Type ヘッダー（cp では対応済み、mv / sync に拡張する）
- `--metadata` — カスタムメタデータ（`key=value` 形式）
- `--metadata-directive` — S3→S3 コピー時の COPY / REPLACE 指定

## 備考

- ライブラリ側の API は `CopyObject` に既に `content_type` / `cache_control` 等が存在する
- `PutObject` / `CreateMultipartUpload` にも同様のヘッダー追加が必要な場合がある
- カスタムメタデータ (`x-amz-meta-*`) はライブラリ側の API 追加が必要

## 解決方法

ライブラリ側:
- `CreateMultipartUpload` に content_encoding / content_disposition / content_language / cache_control / expires フィールドを追加
- `PutObject`, `CopyObject`, `CreateMultipartUpload` に `metadata()` メソッド (x-amz-meta-* ヘッダー) を追加
- `CopyObject` の metadata_directive バリデーションにカスタムメタデータも含める

s3cli 側:
- cp コマンドに --cache-control, --content-disposition, --content-encoding, --content-language, --expires, --metadata, --metadata-directive オプションを追加
- `UploadParams` 構造体にメタデータ系フィールドを統合

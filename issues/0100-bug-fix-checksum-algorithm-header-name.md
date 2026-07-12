# x-amz-checksum-algorithm ヘッダー名の sdk- 欠落を修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/fix-checksum-algorithm-header-name

## 目的

12 ファイルでチェックサムアルゴリズム指定ヘッダー名が `x-amz-checksum-algorithm`（誤）となっており、`sdk-` が欠落している。正しいヘッダー名は `x-amz-sdk-checksum-algorithm`。このバグにより該当 API ではチェックサムアルゴリズムの指定が S3 に認識されず、チェックサム機能が無効化される。

## 優先度根拠

実運用上のバグ。該当 API でチェックサムを指定しても S3 側で無視され、データ整合性検証が行われない。即時修正が必要。

## 現状

- `put_object.rs` と `upload_part.rs` では正しい `x-amz-sdk-checksum-algorithm` が使用されている
- 以下の 12 ファイル (13 箇所) で誤った `x-amz-checksum-algorithm` が使用されている:
  - `src/api/put_bucket_cors.rs:75`
  - `src/api/put_bucket_encryption.rs:89`
  - `src/api/put_bucket_lifecycle_configuration.rs:88`
  - `src/api/put_bucket_policy.rs:64`
  - `src/api/put_bucket_tagging.rs:80`
  - `src/api/put_bucket_versioning.rs:76`
  - `src/api/put_public_access_block.rs:100`
  - `src/api/put_object_tagging.rs:96`
  - `src/api/put_object_lock_configuration.rs:86`
  - `src/api/copy_object.rs:404`
  - `src/api/delete_objects.rs:92`
  - `src/api/create_multipart_upload.rs:250` + `create_multipart_upload.rs:356`

- CHANGES.md の `[CHANGE]` エントリ「`x-amz-checksum-algorithm` ヘッダー名を `x-amz-sdk-checksum-algorithm` に変更する」の反映漏れ

## 設計方針

全 13 箇所の文字列 `"x-amz-checksum-algorithm"` を `"x-amz-sdk-checksum-algorithm"` に置換する。

また、管理系 API（put_bucket_encryption, put_bucket_policy, put_bucket_versioning, put_bucket_lifecycle_configuration, put_public_access_block, put_bucket_tagging, put_bucket_cors, put_bucket_ownership_controls 等）はデータボディのアップロードを伴わないため、`checksum_algorithm` フィールド自体が不要。`put_object.rs` テンプレートの無批判なコピペにより混入したものであり、フィールドごと削除することも検討する。

## 完了条件

- 12 ファイル 13 箇所のヘッダー名が `x-amz-sdk-checksum-algorithm` に修正されていること
- CHANGES.md の該当 CHANGE エントリが全ファイルに反映されていること
- 既存のテストが全て通過すること

## 解決方法

1. 12 ファイルの `"x-amz-checksum-algorithm"` を `"x-amz-sdk-checksum-algorithm"` に一括置換する
2. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
3. `cargo test --workspace` で全テスト通過を確認する

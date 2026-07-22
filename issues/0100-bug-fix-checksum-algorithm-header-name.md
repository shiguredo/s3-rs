# x-amz-checksum-algorithm ヘッダー名の sdk- 欠落を修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Polished: 2026-07-23
- Branch: feature/fix-checksum-algorithm-header-name

## 目的

XML ボディを送信する API において、チェックサムアルゴリズム指定ヘッダー名が `x-amz-checksum-algorithm`（誤）となっており、`sdk-` が欠落している。正しいヘッダー名は `x-amz-sdk-checksum-algorithm`。このバグにより該当 API では SDK ボディチェックサムが S3 に認識されず、リクエストボディの整合性検証が行われない。

**注意**: `CopyObject` と `CreateMultipartUpload` の `x-amz-checksum-algorithm` は、リクエストボディを持たない API であり、オブジェクトに付与するチェックサムアルゴリズムを指定する API レベルのヘッダーとして正しい。本 issue の修正対象には含めない。

## 優先度根拠

実運用上のバグ。該当 API でチェックサムを指定しても S3 側で無視され、データ整合性検証が行われない。即時修正が必要。

## 現状

- `put_object.rs` と `upload_part.rs` では正しい `x-amz-sdk-checksum-algorithm` が使用されている
- 以下の 10 ファイル (10 箇所) で誤った `x-amz-checksum-algorithm` が使用されている:
  - `src/api/put_bucket_cors.rs:75`
  - `src/api/put_bucket_encryption.rs:89`
  - `src/api/put_bucket_lifecycle_configuration.rs:88`
  - `src/api/put_bucket_policy.rs:64`
  - `src/api/put_bucket_tagging.rs:80`
  - `src/api/put_bucket_versioning.rs:76`
  - `src/api/put_public_access_block.rs:100`
  - `src/api/put_object_tagging.rs:96`
  - `src/api/put_object_lock_configuration.rs:86`
  - `src/api/delete_objects.rs:92`

- `copy_object.rs:404` と `create_multipart_upload.rs:250,356` の `x-amz-checksum-algorithm` はリクエストボディを持たない API におけるオブジェクトチェックサム指定用の正しいヘッダーであり、修正不要
- CHANGES.md の `[CHANGE]` エントリ「`x-amz-checksum-algorithm` ヘッダー名を `x-amz-sdk-checksum-algorithm` に変更する」が PutObject / UploadPart のみに反映され、他ファイルに反映漏れがある
- issue 0098（DeleteObjects 固有の同ヘッダー名修正）は本 issue の部分集合であり、本 issue で一括修正される。0098 は close することを推奨する

## 設計方針

- 上記 10 ファイル 10 箇所の `"x-amz-checksum-algorithm"` 文字列を `"x-amz-sdk-checksum-algorithm"` に置換する
- 各 API が `x-amz-sdk-checksum-algorithm` をサポートしているかは AWS S3 API Reference で確認すること。確認できない API については `checksum_algorithm` フィールドごと削除する（リクエストボディの SDK チェックサムが不要な場合）
- `checksum_algorithm` フィールドの要否判断は本 issue のスコープ外とし、必要に応じて別 issue で対応する

## 完了条件

- 上記 10 ファイル 10 箇所のヘッダー名が `x-amz-sdk-checksum-algorithm` に修正されていること
- `x-amz-checksum-algorithm` が残っているのが CopyObject と CreateMultipartUpload のみであること（これらは正しい使用）
- 既存のテストが全て通過すること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること

## 解決方法

1. CopyObject / CreateMultipartUpload を除く 10 ファイルの `"x-amz-checksum-algorithm"` を `"x-amz-sdk-checksum-algorithm"` に置換する
2. `CHANGES.md` の `## develop` に `[FIX]` エントリを追加する
3. `cargo test --workspace` で全テスト通過を確認する

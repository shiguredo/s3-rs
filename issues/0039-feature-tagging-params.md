# PutObject / CopyObject / CreateMultipartUpload の tagging パラメータ対応

## 概要

オブジェクト操作に tagging パラメータを追加する。

## 対象 API とパラメータ

- PutObject: `tagging`
- CopyObject: `tagging`, `tagging_directive`
- CreateMultipartUpload: `tagging`

## 説明

オブジェクトのアップロードやコピー時にタグを設定する。`x-amz-tagging` ヘッダーで URL エンコードされたキーバリューペアを送信する。CopyObject の `tagging_directive` は `COPY` (コピー元のタグを引き継ぐ) または `REPLACE` (新しいタグに置き換え) を指定する。

## 現在の実装状況

- `PutObjectFluentBuilder` (`src/api/put_object.rs`) に `tagging` フィールドは未実装
- `CopyObjectFluentBuilder` (`src/api/copy_object.rs`) に `tagging` / `tagging_directive` フィールドは未実装
- `CreateMultipartUploadFluentBuilder` (`src/api/create_multipart_upload.rs`) に `tagging` フィールドは未実装
- バケットレベルのタグ操作 (`GetBucketTagging`, `PutBucketTagging`, `DeleteBucketTagging`) は実装済み

## 優先度

中

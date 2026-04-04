# PutObject / CopyObject / CreateMultipartUpload の tagging パラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

オブジェクト操作に tagging パラメータを追加する。

## 対象 API とパラメータ

- PutObject: `tagging`
- CopyObject: `tagging`, `tagging_directive`
- CreateMultipartUpload: `tagging`

## 説明

オブジェクトのアップロードやコピー時にタグを設定する。`x-amz-tagging` ヘッダーで URL エンコードされたキーバリューペアを送信する。CopyObject の `tagging_directive` は `COPY` (コピー元のタグを引き継ぐ) または `REPLACE` (新しいタグに置き換え) を指定する。

## 優先度

中

## 解決方法

- `PutObjectFluentBuilder` に `tagging` フィールドを追加し `x-amz-tagging` ヘッダーとして出力
- `CopyObjectFluentBuilder` に `tagging` と `tagging_directive` フィールドを追加し `x-amz-tagging` / `x-amz-tagging-directive` ヘッダーとして出力
- `CreateMultipartUploadFluentBuilder` に `tagging` フィールドを追加し `x-amz-tagging` ヘッダーとして出力

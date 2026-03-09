# PutObject / CompleteMultipartUpload の条件付き書き込みパラメータ対応

## 概要

PutObject と CompleteMultipartUpload に条件付き書き込み用パラメータを追加する。

## 対象 API とパラメータ

- PutObject: `if_match`, `if_none_match`
- CompleteMultipartUpload: `if_match`, `if_none_match`

## 説明

楽観的ロック (Optimistic Locking) を実現するためのパラメータ。`if_none_match=*` で「オブジェクトが存在しない場合のみ作成」、`if_match` で「特定バージョンの場合のみ上書き」が可能。

## 現在の実装状況

- `PutObjectFluentBuilder` (`src/api/put_object.rs`) に `if_match` / `if_none_match` フィールドは未実装
- `CompleteMultipartUploadFluentBuilder` (`src/api/complete_multipart_upload.rs`) に `if_match` / `if_none_match` フィールドは未実装
- GetObject / HeadObject では `if_match` / `if_none_match` が実装済み (#0032)。同じパターンで追加可能
- `Error::PreconditionFailed` バリアントは既に存在する

## 優先度

中

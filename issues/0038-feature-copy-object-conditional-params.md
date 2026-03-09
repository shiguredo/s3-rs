# CopyObject の条件付きコピーパラメータ対応

## 概要

CopyObject に条件付きコピー用のパラメータを追加する。

## 対象パラメータ

- `copy_source_if_match`
- `copy_source_if_none_match`
- `copy_source_if_modified_since`
- `copy_source_if_unmodified_since`

## 説明

コピー元オブジェクトの ETag や更新日時に基づいて条件付きコピーを行う。楽観的排他制御に使用する。

## 現在の実装状況

- `CopyObjectFluentBuilder` (`src/api/copy_object.rs`) に `copy_source_if_*` フィールドは未実装
- 対応する HTTP ヘッダー (`x-amz-copy-source-if-match` 等) の送信も未実装
- GetObject / HeadObject では同等の条件付きヘッダーが実装済み (#0032)。同じパターンで追加可能

## 優先度

中

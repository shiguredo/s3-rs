# CopyObject の条件付きコピーパラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

CopyObject に条件付きコピー用のパラメータを追加する。

## 対象パラメータ

- `copy_source_if_match`
- `copy_source_if_none_match`
- `copy_source_if_modified_since`
- `copy_source_if_unmodified_since`

## 説明

コピー元オブジェクトの ETag や更新日時に基づいて条件付きコピーを行う。楽観的排他制御に使用する。

## 優先度

中

## 解決方法

`CopyObjectFluentBuilder` に 4 つの条件付きコピーフィールドとセッターメソッドを追加し、対応する `x-amz-copy-source-if-*` ヘッダーとして出力するようにした。

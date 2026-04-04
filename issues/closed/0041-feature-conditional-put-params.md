# PutObject / CompleteMultipartUpload の条件付き書き込みパラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

PutObject と CompleteMultipartUpload に条件付き書き込み用パラメータを追加する。

## 対象 API とパラメータ

- PutObject: `if_match`, `if_none_match`
- CompleteMultipartUpload: `if_match`, `if_none_match`

## 説明

楽観的ロック (Optimistic Locking) を実現するためのパラメータ。`if_none_match=*` で「オブジェクトが存在しない場合のみ作成」、`if_match` で「特定バージョンの場合のみ上書き」が可能。

## 優先度

中

## 解決方法

`PutObjectFluentBuilder` と `CompleteMultipartUploadFluentBuilder` の両方に `if_match` と `if_none_match` フィールドとセッターメソッドを追加し、対応する HTTP ヘッダーとして出力するようにした。

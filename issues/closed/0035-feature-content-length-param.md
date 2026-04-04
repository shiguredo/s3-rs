# PutObject / UploadPart の content_length パラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

PutObject と UploadPart に明示的な `content_length` パラメータを追加する。

## 対象 API

- PutObject
- UploadPart

## 説明

通常は body から自動計算されるため不要だが、ストリーミングアップロードやプロキシ的な用途では明示的な Content-Length 指定が必要になる場合がある。

## 優先度

低

## 解決方法

`PutObjectFluentBuilder` と `UploadPartFluentBuilder` の両方に `content_length: Option<i64>` フィールドとセッターメソッドを追加し、指定時は `content-length` ヘッダーとして出力するようにした。

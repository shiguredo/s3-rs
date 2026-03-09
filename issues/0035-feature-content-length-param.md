# PutObject / UploadPart の content_length パラメータ対応

## 概要

PutObject と UploadPart に明示的な `content_length` パラメータを追加する。

## 対象 API

- PutObject
- UploadPart

## 説明

通常は body から自動計算されるため不要だが、ストリーミングアップロードやプロキシ的な用途では明示的な Content-Length 指定が必要になる場合がある。

## 現在の実装状況

- `PutObjectFluentBuilder` (`src/api/put_object.rs`) に `content_length` フィールドは未実装。body から Content-Length を自動計算している
- `UploadPartFluentBuilder` (`src/api/upload_part.rs`) に `content_length` フィールドは未実装。同様に body から自動計算

## 優先度

低

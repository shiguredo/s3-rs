# GetObject / HeadObject の checksum_mode パラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

GetObject と HeadObject に `checksum_mode` パラメータを追加する。

## 対象 API

- GetObject
- HeadObject

## 説明

`checksum_mode=ENABLED` を指定すると、レスポンスにオブジェクトのチェックサム値が含まれる。データ整合性の検証に使用する。

## 優先度

低

## 解決方法

- `GetObjectFluentBuilder` と `HeadObjectFluentBuilder` に `checksum_mode` フィールドとセッターメソッドを追加し、`x-amz-checksum-mode` ヘッダーとして出力するようにした
- `GetObjectOutput` と `HeadObjectOutput` に `checksum_crc32`, `checksum_crc32c`, `checksum_crc64nvme`, `checksum_sha1`, `checksum_sha256` フィールドを追加し、レスポンスヘッダーからパースするようにした

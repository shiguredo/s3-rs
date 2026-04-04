# 全 API に expected_bucket_owner / request_payer を追加する

Created: 2026-04-05
Model: Opus 4.6

## 根拠

- aws-sdk-rust の S3 builder はほぼ全操作で `expected_bucket_owner`（`x-amz-expected-bucket-owner`）と `request_payer`（`x-amz-request-payer`）を公開している
- AWS 公式 API 仕様でも同ヘッダーが各操作に定義されている
- 現在の shiguredo_s3 実装にはどちらも一切存在せず、Requester Pays バケットやオーナー検証が必要なユースケースに対応できない
- 個別 API の issue に分散して書くと同一内容が 10 箇所以上重複するため、横断 issue として一括対応する

## 概要

既存および今後追加する全 S3 API の builder に、以下の共通フィールドを追加する。

### `expected_bucket_owner`

- ヘッダー: `x-amz-expected-bucket-owner`
- 型: `Option<String>`（AWS アカウント ID）
- 用途: リクエスト対象バケットのオーナーが指定アカウントと一致しない場合に `403 Forbidden` を返す。バケット名の衝突や意図しないバケットへの操作を防ぐセキュリティ機構。

### `request_payer`

- ヘッダー: `x-amz-request-payer`
- 型: `Option<RequestPayer>`（値は `"requester"` のみ）
- 用途: Requester Pays バケットへのアクセス時にリクエスタが料金を負担することを明示する。

## 対象範囲

既存実装済み API（GetObject, PutObject, HeadObject, DeleteObject, ListObjectsV2, CopyObject 等）および今後追加する全 API。

## AWS 公式ドキュメント

- [Using Requester Pays buckets](https://docs.aws.amazon.com/AmazonS3/latest/userguide/RequesterPaysBuckets.html)
- 各 API リファレンスの Request Headers セクションに `x-amz-expected-bucket-owner` / `x-amz-request-payer` の記載がある

## 補足

- aws-sdk-rust では `RequestPayer` は enum で定義されている。同じ型を用意すること。
- レスポンスに `x-amz-request-charged` が返る API もあるが、まずリクエスト側から対応する。

## 優先度

中（個別 API の機能追加と並行して対応可能）

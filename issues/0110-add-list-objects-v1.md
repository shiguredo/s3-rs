# ListObjects を追加する

- Priority: High
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-list-objects-v1
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust が提供する ListObjects v1 と互換の Sans I/O API を追加し、既存の ListObjectsV2 へ移行できない利用者も同じ API surface で扱えるようにする。

## 現状

src/api/ には ListObjectsV2 は存在するが、ListObjects v1 に対応する builder、入力型、出力型、リクエスト構築、レスポンスパースが存在しない。

## 設計方針

- aws-sdk-rust の ListObjectsInput / ListObjectsOutput と同じフィールド名・型を採用する。
- marker、delimiter、encoding_type、max_keys、prefix、expected_bucket_owner、request_payer、optional_object_attributes を仕様どおりに扱う。
- ListBucketResult の Contents、CommonPrefixes、NextMarker、EncodingType、request charged を既存の共通 XML パース方針で処理する。
- 実際の S3 互換サーバーを使う統合テストを追加する。

## 完了条件

- client.list_objects() からリクエストを構築できる。
- ListObjects v1 のレスポンスを aws-sdk-rust 互換の output としてパースできる。
- ページングに必要な NextMarker と全ての入力項目を検証する統合テストが通る。

## AWS S3 API Reference

- ListObjects: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html

> Returns some or all (up to 1,000) of the objects in a bucket.

# Object Annotation API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-object-annotation
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetObjectAnnotation、PutObjectAnnotation、DeleteObjectAnnotation、ListObjectAnnotations を追加し、aws-sdk-rust が提供する object annotation API と互換にする。

## 現状

src/api/ に object annotation 用の operation、入力・出力型、annotation の XML / header 処理が存在しない。

## 設計方針

- aws-sdk-rust の operation input / output と同じ API 名、フィールド名、型を採用する。
- object version、request payer、expected bucket owner、checksum の扱いを各 API の仕様に合わせる。
- annotation の XML またはレスポンス形式を公式 API と SDK の実装から確認し、共通 XML パーサーに無理な特例を追加しない。

## 完了条件

- 4 operation が client から利用できる。
- annotation の取得、設定、削除、一覧取得を正しい HTTP method、URI、header、body で扱える。
- 実際の S3 互換サーバー、または実レスポンスを用いた Sans I/O テストが通る。

## AWS S3 API Reference

- GetObjectAnnotation: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAnnotation.html

> GetObjectAnnotation

- PutObjectAnnotation: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectAnnotation.html

> PutObjectAnnotation

- DeleteObjectAnnotation: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectAnnotation.html

> DeleteObjectAnnotation

- ListObjectAnnotations: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectAnnotations.html

> ListObjectAnnotations

# HeadObject の response override parameter を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-head-object-response-overrides
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust の HeadObjectInput に存在する response_* query parameter を追加し、GetObject と HeadObject の API surface を揃える。

## 現状

GetObject には response_cache_control、response_content_disposition、response_content_encoding、response_content_language、response_content_type、response_expires が実装済みである。一方、src/api/head_object.rs の HeadObjectFluentBuilder には存在しない。

## 設計方針

- GetObject と同じ builder method 名・値の型・query parameter 名を採用する。
- build_request と presigned の両方へ反映する。
- URL encoding と署名対象 query の扱いを GetObject と一致させる。
- 既存の GetObject response override issue と重複しないよう HeadObject のみを対象にする。

## 完了条件

- 6 項目の fluent method と set_* method が追加される。
- 通常 request と presigned request の URI が SDK と一致する。
- 空文字列、予約文字、複数項目を含む署名検証テストが通る。

## AWS S3 API Reference

- HeadObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html

> HEAD object responses contain the same headers as GET object responses, except that there is no response body.

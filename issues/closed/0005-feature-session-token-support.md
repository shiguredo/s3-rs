# 一時クレデンシャル (session token) 未対応

## 優先度

P1

## 概要

`Credential` 構造体に `session_token` フィールドがなく、
署名時に `x-amz-security-token` ヘッダー、presigned URL 生成時に `X-Amz-Security-Token` クエリパラメータを付与していない。

## 影響

- STS / AssumeRole / ECS Task Role / Lambda など一時クレデンシャル環境で認証できない

## 該当箇所

- `src/credential.rs:3-8` - `Credential` 構造体
- `src/api/mod.rs:181-188` - `build_signed_request` で `x-amz-security-token` 未付与
- `src/api/mod.rs:312-318` - `build_presigned_url` で `X-Amz-Security-Token` 未付与

## 修正方針

1. `Credential` に `session_token: Option<String>` を追加する
2. `build_signed_request` で `session_token` があれば `x-amz-security-token` ヘッダーを追加する
3. `build_presigned_url` で `session_token` があれば `X-Amz-Security-Token` クエリパラメータを追加する

## 完了

- `Credential` に `session_token: Option<String>` フィールドを追加
- `Credential::with_session_token()` コンストラクタを追加
- `build_signed_request` / `build_signed_service_request` で `x-amz-security-token` ヘッダーを付与
- `build_presigned_url` で `X-Amz-Security-Token` クエリパラメータを付与

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/RESTCommonRequestHeaders.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html

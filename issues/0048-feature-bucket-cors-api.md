# バケット CORS API を追加する

Created: 2026-03-27
Model: Opus 4.6

## 概要

GetBucketCors / PutBucketCors / DeleteBucketCors の 3 API を追加する。

## 根拠

ブラウザから S3 互換ストレージ（R2 等）に Presigned URL 経由で直接アクセスする場合、バケットに CORS 設定が必要。現状 shiguredo_s3 には CORS 管理 API がなく、利用者は別途 AWS CLI 等で設定する必要がある。

S3 標準 API であり、Cloudflare R2 でもサポートされている。

## 対象 API

### GetBucketCors

- `GET /{Bucket}?cors`
- CORS 設定を取得する
- レスポンス: `<CORSConfiguration>` XML

### PutBucketCors

- `PUT /{Bucket}?cors`
- CORS 設定を作成・更新する
- リクエスト: `<CORSConfiguration>` XML
- `Content-MD5` ヘッダーが必須

### DeleteBucketCors

- `DELETE /{Bucket}?cors`
- CORS 設定を削除する

## 追加する型

- `CorsRule`: AllowedHeaders, AllowedMethods, AllowedOrigins, ExposeHeaders, MaxAgeSeconds, ID
- `CorsConfiguration`: CorsRule のリスト
- `GetBucketCorsOutput`
- `PutBucketCorsOutput`
- `DeleteBucketCorsOutput`

## 実装上の注意

- CORSRule 内で AllowedMethod, AllowedOrigin 等の同名要素が複数出現する
- `xml.rs` の `ChildElements` に同名要素を全て取得する `get_all` メソッドの追加が必要
- PutBucketCors の XML 生成では同一タグ名の要素を複数回書く必要がある
- ルールは最大 100 個、XML は 64KB 以下（S3 の制約）

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketCors.html

## 優先度

中

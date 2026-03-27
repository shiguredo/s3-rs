# s3-rs

[![shiguredo_s3](https://img.shields.io/crates/v/shiguredo_s3.svg)](https://crates.io/crates/shiguredo_s3)
[![Documentation](https://docs.rs/shiguredo_s3/badge.svg)](https://docs.rs/shiguredo_s3)
[![GitHub Actions](https://github.com/shiguredo/s3-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/s3-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## 概要

> [!IMPORTANT]
> **Amazon S3 を利用する場合は、AWS 公式の Rust SDK である [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust) の利用を推奨します。**
> aws-sdk-rust は Amazon が公式にメンテナンスしており、S3 の全機能に対応しています。
>
> このライブラリは必要最低限の S3 API のみを実装し、依存クレートを最小限に抑えることを目的としています。
> aws-sdk-rust は非常に多機能ですが、依存クレートが多く、ビルド時間やバイナリサイズが大きくなります。
> shiguredo_s3 は S3 互換オブジェクトストレージ (MinIO, Cloudflare R2, Akamai Cloud Object Storage など) への接続や、
> 限られた S3 操作のみを必要とする軽量な用途に向いています。
>
> API は aws-sdk-rust スタイルに合わせているため、将来的に aws-sdk-rust へ移行する際にも違和感なく移行できます。

依存を最小限に抑えた Sans I/O な Rust 向け Amazon S3 クライアントライブラリです。

## 特徴

- [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust) スタイルの Fluent Builder API
- Sans I/O アーキテクチャ
- AWS Signature V4 署名
- 一時クレデンシャル対応 (STS / IAM ロール / ECS Task Role / Lambda)
- Presigned リクエスト対応 (全オペレーション)
- マルチパートアップロード対応
- Flexible Checksums 対応 (CRC32 / CRC32C / CRC64NVME / SHA-1 / SHA-256、デフォルト CRC32 自動計算)
- SSE-C key MD5 自動計算
- HTTP / HTTPS 対応 (endpoint のスキームで自動判定)
- HTTPS 時のドット付きバケット名で path-style に自動フォールバック
- IPv6 エンドポイント対応
- S3 互換サービス対応 (パススタイルアクセス、カスタムエンドポイント)

## 依存ライブラリ

### 署名計算・チェックサム計算 (SHA-256 / SHA-1 / HMAC-SHA256)

feature flags で暗号ライブラリを切り替えられます。
署名計算 (SHA-256, HMAC-SHA256) とチェックサム計算 (SHA-256, SHA-1) の両方に使用します。

| feature | クレート | 備考 |
|---|---|---|
| `rust-crypto` (デフォルト) | sha2 / sha1 / hmac | pure Rust 実装 |
| `aws-lc-rs` | aws-lc-rs | rustls で aws-lc-rs を使用する場合はこちらを指定すること |

```toml
# デフォルト (rust-crypto)
shiguredo_s3 = "<version>"

# aws-lc-rs を使用する場合
shiguredo_s3 = { version = "<version>", default-features = false, features = ["aws-lc-rs"] }
```

### チェックサム計算 (CRC32 / CRC32C / CRC64NVME)

- crc-fast を使用 (feature に依存しない共通依存)

### その他

- Content-MD5 / SSE-C key MD5 計算は md-5 + base64 を使用
- XML パースは xml を使用

## Fluent Builder API (aws-sdk-rust スタイル)

### オブジェクト操作

| オペレーション | メソッド |
|---|---|
| GetObject | `client.get_object()` |
| HeadObject | `client.head_object()` |
| PutObject | `client.put_object()` |
| DeleteObject | `client.delete_object()` |
| DeleteObjects | `client.delete_objects()` |
| CopyObject | `client.copy_object()` |
| ListObjectsV2 | `client.list_objects_v2()` |
| GetObjectTagging | `client.get_object_tagging()` |
| PutObjectTagging | `client.put_object_tagging()` |
| DeleteObjectTagging | `client.delete_object_tagging()` |

### バケット操作

| オペレーション | メソッド |
|---|---|
| HeadBucket | `client.head_bucket()` |
| CreateBucket | `client.create_bucket()` |
| DeleteBucket | `client.delete_bucket()` |
| ListBuckets | `client.list_buckets()` |

### マルチパートアップロード

| オペレーション | メソッド |
|---|---|
| CreateMultipartUpload | `client.create_multipart_upload()` |
| UploadPart | `client.upload_part()` |
| CompleteMultipartUpload | `client.complete_multipart_upload()` |
| AbortMultipartUpload | `client.abort_multipart_upload()` |
| ListParts | `client.list_parts()` |
| ListMultipartUploads | `client.list_multipart_uploads()` |

### バケット設定

| オペレーション | メソッド |
|---|---|
| GetBucketVersioning | `client.get_bucket_versioning()` |
| PutBucketVersioning | `client.put_bucket_versioning()` |
| GetBucketTagging | `client.get_bucket_tagging()` |
| PutBucketTagging | `client.put_bucket_tagging()` |
| DeleteBucketTagging | `client.delete_bucket_tagging()` |
| GetBucketPolicy | `client.get_bucket_policy()` |
| PutBucketPolicy | `client.put_bucket_policy()` |
| DeleteBucketPolicy | `client.delete_bucket_policy()` |
| GetPublicAccessBlock | `client.get_public_access_block()` |
| PutPublicAccessBlock | `client.put_public_access_block()` |
| DeletePublicAccessBlock | `client.delete_public_access_block()` |
| GetBucketLifecycleConfiguration | `client.get_bucket_lifecycle_configuration()` |
| PutBucketLifecycleConfiguration | `client.put_bucket_lifecycle_configuration()` |
| DeleteBucketLifecycle | `client.delete_bucket_lifecycle()` |

メソッド名や引数の渡し方を [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust) スタイルにしています。aws-sdk-rust へ移行する際に違和感なく移行できることを目的としています。

### Presigned リクエスト

`presigned(expires)` は `PresignedRequest` を返します。
`PresignedRequest` は `url`、`method`、`body` を持ちます。
CompleteMultipartUpload のように POST ボディが必要なオペレーションでは `body` に XML が入ります。

| オペレーション | メソッド |
|---|---|
| Presigned GetObject | `client.get_object().presigned(expires)` |
| Presigned HeadObject | `client.head_object().presigned(expires)` |
| Presigned PutObject | `client.put_object().presigned(expires)` |
| Presigned DeleteObject | `client.delete_object().presigned(expires)` |
| Presigned UploadPart | `client.upload_part().presigned(expires)` |
| Presigned CreateMultipartUpload | `client.create_multipart_upload().presigned(expires)` |
| Presigned CompleteMultipartUpload | `client.complete_multipart_upload().presigned(expires)` |
| Presigned AbortMultipartUpload | `client.abort_multipart_upload().presigned(expires)` |

## Sans I/O

全ての API は Sans I/O で設計されています。
`build_request()` で署名済みの `S3Request` を構築し、`parse_response()` で `S3Response` をパースします。
HTTP の通信とエンコード/デコードは利用者が自由に実装できます。

```rust
use shiguredo_s3::{S3Request, S3Response};

// 署名済みリクエストを構築する (Sans I/O)
let s3_request = client.get_object()
    .bucket("my-bucket")
    .key("my-key")
    .build_request()?;

// S3Request のフィールド:
//   s3_request.method   - HTTP メソッド (GET, PUT, DELETE, POST, HEAD)
//   s3_request.uri      - リクエスト URI (パス + クエリ文字列)
//   s3_request.headers  - HTTP リクエストヘッダー (署名済み)
//   s3_request.body     - リクエストボディ
//   s3_request.host     - 接続先ホスト名
//   s3_request.port     - 接続先ポート番号
//   s3_request.https    - HTTPS を使用するかどうか

// S3Request を使って HTTP 通信を行う (I/O: 利用者が実装する)
// s3_request.host:s3_request.port に接続してリクエストを送信し、レスポンスを受信する

// レスポンスから S3Response を構築する (利用者が実装する)
let s3_response = S3Response {
    status_code: 200,
    headers: vec![("etag".into(), "\"...\"".into())],
    body: response_body,
};

// レスポンスをパースする (Sans I/O)
let output = shiguredo_s3::api::GetObjectFluentBuilder::parse_response(&s3_response)?;
```

## 使い方

以下の例では `execute()` は利用者が実装する HTTP 通信関数で、
`S3Request` を送信して `S3Response` を返すものとします。

### S3 クライアントの作成

```rust
use shiguredo_s3::{S3Client, S3Config, Credential};

let config = S3Config::builder()
    .region("ap-northeast-1")
    .credential(Credential::new("YOUR_ACCESS_KEY_ID", "YOUR_SECRET_ACCESS_KEY"))
    .build()?;

let client = S3Client::new(config);
```

### 一時クレデンシャル (STS / IAM ロール)

```rust
use shiguredo_s3::{S3Client, S3Config, Credential};

let config = S3Config::builder()
    .region("ap-northeast-1")
    .credential(Credential::with_session_token(
        "YOUR_ACCESS_KEY_ID",
        "YOUR_SECRET_ACCESS_KEY",
        "YOUR_SESSION_TOKEN",
    ))
    .build()?;

let client = S3Client::new(config);
```

### GetObject

```rust
let s3_request = client.get_object()
    .bucket("my-bucket")
    .key("my-key")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::GetObjectFluentBuilder::parse_response(&s3_response)?;

println!("body: {} bytes", output.body.len());
```

Range や PartNumber も指定できます。

```rust
let s3_request = client.get_object()
    .bucket("my-bucket")
    .key("my-key")
    .range("bytes=0-1023")
    .build_request()?;
```

### PutObject

```rust
let s3_request = client.put_object()
    .bucket("my-bucket")
    .key("my-key")
    .body(b"Hello, World!".to_vec())
    .content_type("text/plain")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::PutObjectFluentBuilder::parse_response(&s3_response)?;
```

### DeleteObject

```rust
let s3_request = client.delete_object()
    .bucket("my-bucket")
    .key("my-key")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response(&s3_response)?;
```

### BucketVersioning

```rust
// バージョニングを有効にする
let s3_request = client.put_bucket_versioning()
    .bucket("my-bucket")
    .status("Enabled")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::PutBucketVersioningFluentBuilder::parse_response(&s3_response)?;

// バージョニング設定を取得する
let s3_request = client.get_bucket_versioning()
    .bucket("my-bucket")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::GetBucketVersioningFluentBuilder::parse_response(&s3_response)?;
println!("status: {:?}", output.status); // Some("Enabled")
```

### BucketLifecycleConfiguration

```rust
use shiguredo_s3::types::{
    ExpirationStatus, LifecycleExpiration, LifecycleRule, LifecycleRuleFilter,
    AbortIncompleteMultipartUpload,
};

// 30 日後にオブジェクトを削除し、7 日後に不完全なマルチパートアップロードを中止するルールを設定する
let s3_request = client.put_bucket_lifecycle_configuration()
    .bucket("my-bucket")
    .rule(LifecycleRule {
        id: Some("expire-30-days".to_string()),
        status: ExpirationStatus::Enabled,
        filter: Some(LifecycleRuleFilter {
            prefix: Some("logs/".to_string()),
            ..Default::default()
        }),
        expiration: Some(LifecycleExpiration {
            days: Some(30),
            ..Default::default()
        }),
        transitions: None,
        noncurrent_version_transitions: None,
        noncurrent_version_expiration: None,
        abort_incomplete_multipart_upload: Some(AbortIncompleteMultipartUpload {
            days_after_initiation: Some(7),
        }),
    })
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::PutBucketLifecycleConfigurationFluentBuilder::parse_response(&s3_response)?;

// ライフサイクル設定を取得する
let s3_request = client.get_bucket_lifecycle_configuration()
    .bucket("my-bucket")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::GetBucketLifecycleConfigurationFluentBuilder::parse_response(&s3_response)?;
for rule in &output.rules {
    println!("rule: {:?}", rule.id);
}

// ライフサイクル設定を削除する
let s3_request = client.delete_bucket_lifecycle()
    .bucket("my-bucket")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::DeleteBucketLifecycleFluentBuilder::parse_response(&s3_response)?;
```

### Presigned リクエスト

```rust
use shiguredo_s3::PresignedRequest;

// 1 時間有効な GetObject Presigned リクエストを生成する
let presigned = client.get_object()
    .bucket("my-bucket")
    .key("my-key")
    .presigned(3600)?;
println!("url: {}", presigned.url);

// PutObject Presigned リクエスト
let presigned = client.put_object()
    .bucket("my-bucket")
    .key("my-key")
    .presigned(3600)?;
```

### マルチパートアップロード

```rust
use shiguredo_s3::types::{CompletedPart, CompletedMultipartUpload};

// 1. マルチパートアップロードを開始する
let s3_request = client.create_multipart_upload()
    .bucket("my-bucket")
    .key("my-key")
    .content_type("application/octet-stream")
    .build_request()?;
let s3_response = execute(s3_request).await?;
let create_output = shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response(&s3_response)?;
let upload_id = create_output.upload_id.unwrap();

// 2. パートをアップロードする
let s3_request = client.upload_part()
    .bucket("my-bucket")
    .key("my-key")
    .upload_id(&upload_id)
    .part_number(1)
    .body(part1_data)
    .build_request()?;
let s3_response = execute(s3_request).await?;
let part1 = shiguredo_s3::api::UploadPartFluentBuilder::parse_response(&s3_response)?;

// 3. マルチパートアップロードを完了する
let s3_request = client.complete_multipart_upload()
    .bucket("my-bucket")
    .key("my-key")
    .upload_id(&upload_id)
    .multipart_upload(CompletedMultipartUpload {
        parts: Some(vec![
            CompletedPart {
                e_tag: part1.e_tag,
                part_number: Some(1),
            },
        ]),
    })
    .build_request()?;
let s3_response = execute(s3_request).await?;
let output = shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response(&s3_response)?;
```

### S3 互換サービス (Cloudflare R2)

```rust
let config = S3Config::builder()
    .region("auto")
    .credential(Credential::new("YOUR_ACCESS_KEY_ID", "YOUR_SECRET_ACCESS_KEY"))
    .endpoint("https://<ACCOUNT_ID>.r2.cloudflarestorage.com")
    .use_path_style(true)
    .build()?;

let client = S3Client::new(config);
```

### S3 互換サービス (Akamai Cloud Object Storage)

```rust
let config = S3Config::builder()
    .region("us-east-1")
    .credential(Credential::new("YOUR_ACCESS_KEY_ID", "YOUR_SECRET_ACCESS_KEY"))
    .endpoint("https://<CLUSTER_ID>.linodeobjects.com")
    .use_path_style(true)
    .build()?;

let client = S3Client::new(config);
```

### S3 互換サービス (RustFS、HTTP)

endpoint のスキーム (`http://` / `https://`) から `S3Request.https` フラグを自動設定します。
スキームを省略した場合は HTTPS がデフォルトです。

```rust
let config = S3Config::builder()
    .region("us-east-1")
    .credential(Credential::new("rustfsadmin", "rustfsadmin"))
    .endpoint("http://localhost:9000")
    .use_path_style(true)
    .build()?;

let client = S3Client::new(config);
```

## サンプル (tokio + rustls + shiguredo_http11)

[examples/s3cli/](examples/s3cli/) に tokio + rustls + shiguredo_http11 を使った
aws s3 互換の CLI ツールの実装があります。
Sans I/O ライブラリの具体的な利用方法の参考にしてください。

### S3Request のエンコード

`S3Request` は HTTP メソッド、URI、ヘッダー、ボディを持つ構造体です。
これを shiguredo_http11 の `Request` に変換して HTTP/1.1 バイト列にエンコードします。

```rust
fn encode_request(s3_request: &S3Request) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut request = shiguredo_http11::Request::new(&s3_request.method, &s3_request.uri);
    for (name, value) in &s3_request.headers {
        request.add_header(name, value);
    }
    if !s3_request.body.is_empty() {
        request.body = s3_request.body.clone();
    }
    Ok(request.try_encode()?)
}
```

### S3Response への変換

HTTP レスポンスを受信したら、shiguredo_http11 の `ResponseDecoder` でデコードし、
`S3Response` に変換します。

```rust
fn into_s3_response(response: shiguredo_http11::Response) -> S3Response {
    S3Response {
        status_code: response.status_code,
        headers: response.headers,
        body: response.body,
    }
}
```

### HTTP 通信 (execute 関数)

`S3Request` の接続情報 (`host`, `port`, `https`) を使って TCP 接続を行い、
`https` フラグに応じて TLS をかぶせます。

```rust
async fn execute(
    tls_config: &Arc<rustls::ClientConfig>,
    s3_request: S3Request,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", s3_request.host, s3_request.port);
    let tcp = tokio::net::TcpStream::connect(&addr).await?;

    // S3Request を HTTP/1.1 バイト列にエンコードする
    let encoded = encode_request(&s3_request)?;

    // HEAD リクエストはレスポンスにボディがない
    let mut decoder = shiguredo_http11::ResponseDecoder::new();
    if s3_request.expect_no_body {
        decoder.set_expect_no_body(true);
    }

    if s3_request.https {
        // HTTPS: TLS 接続
        let server_name = rustls::pki_types::ServerName::try_from(s3_request.host.clone())?;
        let connector = tokio_rustls::TlsConnector::from(Arc::clone(tls_config));
        let mut tls = connector.connect(server_name, tcp).await?;
        tls.write_all(&encoded).await?;
        read_response(&mut tls, &mut decoder).await
    } else {
        // HTTP: 平文 TCP 接続
        let (mut reader, mut writer) = tokio::io::split(tcp);
        writer.write_all(&encoded).await?;
        read_response(&mut reader, &mut decoder).await
    }
}
```

### レスポンスの読み取り

ストリームからレスポンスを読み取り、`ResponseDecoder` でデコードして `S3Response` に変換します。

```rust
async fn read_response<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
    decoder: &mut shiguredo_http11::ResponseDecoder,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            decoder.mark_eof();
            if let Some(response) = decoder.decode()? {
                return Ok(into_s3_response(response));
            }
            return Err("unexpected EOF".into());
        }
        decoder.feed(&buf[..n])?;
        if let Some(response) = decoder.decode()? {
            return Ok(into_s3_response(response));
        }
    }
}
```

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

# 署名対象ヘッダー値の空白エンコード処理

## 概要

S3 各 API ページの `Important` 注記に "You must URL encode any signed header values that contain spaces." と明記されている。
一般 SigV4 仕様 (trim + fold) との並立であり、どちらが優先されるか実機未検証のため保留。

## 該当箇所

- `src/signing.rs:140` (`fold_whitespace` 関数)
- `src/signing.rs:208` (通常署名の canonical headers 構築)
- `src/signing.rs:274` (presigned 署名の canonical headers 構築)
- `src/api/put_object.rs:97` (`Content-Disposition` など空白を含み得るヘッダーを署名対象に追加)
- `src/api/copy_object.rs:112` (同上)

## 一次資料

| 資料 | 内容 |
|------|------|
| [IAM SigV4](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html) L167-L172 | canonical headers の値は trim + fold のみ、URL encode の記述なし |
| [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html) L73-L86 | "You must URL encode any signed header values that contain spaces." |
| [UploadPart](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPart.html) L81-L99 | 同上 |
| [CompleteMultipartUpload](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html) L69-L85 | 同上 |

## 確認事項

1. AWS SDK (Go v2 / Python boto3 / Java v2) が `Content-Disposition: attachment; filename=hello world` のような値を signed header に入れるとき、canonical headers でどう処理しているか
2. 実機で `Content-Disposition` に空白を含む値を PutObject したとき、現行実装 (`fold_whitespace` のみ) で `SignatureDoesNotMatch` が返るかどうか

## 修正方針 (確定後)

`fold_whitespace` の後、空白を `%20` に変換する処理を追加する。
または `fold_whitespace` 自体を廃止して URL encode に置き換える。

## 解決方法

**対応不要と判断し、close する。**

AWS 公式 SDK for Rust (aws-sigv4 クレート) の実装を確認した結果、スペースの URL エンコードは行われていないことを確認した。

### 根拠

`aws-sdk-rust/sdk/aws-sigv4/src/http_request/canonical_request.rs` の実装:

```rust
fn normalize_header_value(header_value: &str) -> Result<HeaderValue, CanonicalRequestError> {
    let trimmed_value = trim_all(header_value);
    HeaderValue::from_str(&trimmed_value).map_err(CanonicalRequestError::from)
}
```

`trim_all` は以下のみを行う:
- 前後のスペース除去 (`trim_matches(' ')`)
- 連続スペースを単一スペースに畳む

スペースを `%20` に変換する処理は存在しない。

### 結論

| 実装 | 処理 |
|------|------|
| AWS SDK for Rust (aws-sigv4) | trim + fold のみ、URL encode なし |
| 現行 s3-rs `fold_whitespace` | trim + fold のみ（同じ動作） |

現行の `fold_whitespace` は AWS 公式 SDK と同じ動作をしているため変更不要。
S3 API ページの注記は、raw リクエストを手組みするユーザー向けの案内であり、SigV4 署名ライブラリが対応すべき仕様ではないと判断する。

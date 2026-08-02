# SSE-C ヘッダー構築ロジックの重複を解消する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-sse-c-header-construction-duplication
- Polished: 2026-08-02

## 目的

SSE-C (Server-Side Encryption with Customer-provided keys) のヘッダー構築ロジックが 9 ファイル 17 箇所で重複している。共通ヘルパー関数として抽出する。

### 参照仕様

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html

> x-amz-server-side-encryption-customer-algorithm: Specifies the algorithm to use to when encrypting the object (for example, AES256).
> x-amz-server-side-encryption-customer-key: Specifies the customer-provided encryption key for Amazon S3 to use in encrypting data.
> x-amz-server-side-encryption-customer-key-MD5: Specifies the 128-bit MD5 digest of the encryption key according to RFC 1321.

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html

> x-amz-copy-source-server-side-encryption-customer-algorithm: Specifies the algorithm to use to when decrypting the source object (for example, AES256).

## 優先度根拠

同一コードのコピペによる修正漏れ（特に MD5 自動計算ロジックの変更時）のリスクが高い。コードベースの保守性を向上させる。

## 現状

以下の 9 ファイルで以下のパターンが重複（algorithm → key → MD5 の順）:

```rust
if let Some(ref v) = self.sse_customer_algorithm {
    extra_headers.push((
        "x-amz-server-side-encryption-customer-algorithm",
        v.as_str(),
    ));
}
let computed_key_md5;
if let Some(ref v) = self.sse_customer_key {
    extra_headers.push(("x-amz-server-side-encryption-customer-key", v.as_str()));
    computed_key_md5 = super::compute_sse_c_key_md5(v)?;
    extra_headers.push(("x-amz-server-side-encryption-customer-key-md5", &computed_key_md5));
}
```

| ファイル | 箇所数 | 備考 |
|----------|--------|------|
| `put_object.rs` | 2 | build_request + presigned |
| `upload_part.rs` | 2 | build_request + presigned |
| `complete_multipart_upload.rs` | 2 | build_request + presigned |
| `get_object.rs` | 2 | build_request + presigned |
| `head_object.rs` | 2 | build_request + presigned |
| `create_multipart_upload.rs` | 2 | build_request + presigned |
| `copy_object.rs` | 2 | 標準 SSE-C + コピー元 SSE-C（`x-amz-copy-source-server-side-encryption-customer-*`） |
| `upload_part_copy.rs` | 2 | 標準 SSE-C + コピー元 SSE-C |
| `list_parts.rs` | 1 | build_request のみ |

`copy_object.rs` と `upload_part_copy.rs` のコピー元 SSE-C はヘッダー名が `x-amz-copy-source-server-side-encryption-customer-*` と異なるが、ロジックの構造は同一である。

## 設計方針

`src/api/mod.rs` に以下のヘルパー関数を追加する:

```rust
pub(crate) fn add_sse_c_headers(
    extra_headers: &mut Vec<(&str, &str)>,
    sse_customer_algorithm: Option<&str>,
    sse_customer_key: Option<&str>,
    computed_key_md5: &mut Option<String>,
    copy_source: bool,
) -> Result<(), Error>
```

- `computed_key_md5` は呼び出し側で宣言した `Option<String>` を `&mut` で渡す（MD5 計算結果のライフタイムを呼び出し側が管理する。関数が `String` を返すと vec 内の参照がダングリングするため。タプル返し等の代替案も検討したが、vec 内に `&str` 参照を push する既存パターンとの整合性から `&mut Option<String>` が最も自然である）
- コピー元 SSE-C 用には `copy_source: bool` フラグで切り替える（ヘッダー名は静的文字列で選択する。動的な文字列生成は避ける）。`copy_source: true` の場合は `x-amz-copy-source-server-side-encryption-customer-*`、`false` の場合は `x-amz-server-side-encryption-customer-*` を使用する
- 呼び出し側では `self.sse_customer_algorithm.as_deref()` のように `Option<String>` から `Option<&str>` に変換して渡す

## 完了条件

- SSE-C ヘッダー構築がヘルパー関数に集約されていること
- 全 9 ファイル 17 箇所がヘルパー関数を呼び出す形に変更されていること（コピー元 SSE-C の 2 箇所を含む）
- 挙動が変更前後で同一であること
- 既存のテストが全て通過すること

## 解決方法

1. `src/api/mod.rs` に `add_sse_c_headers` 関数を実装する（コピー元 SSE-C 対応を含む）
2. 全 9 ファイル 17 箇所の重複コードをヘルパー関数呼び出しに置き換える
3. CHANGES.md の `## develop` の `### misc` にエントリを追加する

## 他 issue との依存関係

- 0106（api/mod.rs 分離）が本 issue の成果物（`add_sse_c_headers`）を `src/api/util.rs` に移動する予定。0106 より先に本 issue を実装する方が手戻りが少ない

# CopyObject のメタデータ事前検証

## 分類

設計判断

## 概要

`CopyObjectFluentBuilder::build_request` はメタデータ系フィールド（content_type, content_encoding, metadata 等）が設定されている場合、`metadata_directive` が `"REPLACE"` でなければクライアント側でエラーを返している。

## 現状

```rust
if has_metadata_override && self.metadata_directive.as_deref() != Some("REPLACE") {
    return Err(Error::InvalidInput(
        "metadata_directive must be \"REPLACE\" when metadata fields are set".to_string(),
    ));
}
```

## AWS SDK との差分

- aws-sdk-rust は `metadata` と `metadata_directive` を独立して受け付け、クライアント側でバリデーションしない
- S3 サーバは `COPY` ディレクティブ（デフォルト）のとき、ユーザー指定のメタデータをサイレントに無視する

## 論点

- 事前検証は「メタデータを設定したのに反映されない」という利用者の混乱を防ぐ
- しかし aws-sdk-rust では通る入力がこちらでは落ちるため、独自の入力契約になる
- 互換性を優先するならバリデーションを除去すべき
- 利用者保護を優先するなら残すが、その方針転換を明示すべき

## 現在の実装箇所

- `src/api/copy_object.rs:199-211` — `build_request` 内で `has_metadata_override && metadata_directive != Some("REPLACE")` のとき `Error::InvalidInput` を返している
- 検査対象: `content_type`, `content_encoding`, `content_disposition`, `content_language`, `cache_control`, `expires`, `metadata`

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html

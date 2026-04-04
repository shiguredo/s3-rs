# CreateBucket の LocationConstraint 自動付与

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 分類

設計判断

## 概要

`CreateBucketFluentBuilder::build_request` はクライアント設定リージョンが `us-east-1` 以外の場合、`LocationConstraint` を自動的に XML ボディに注入している。

## 現状

```rust
let body = if config.region == "us-east-1" {
    Vec::new()
} else {
    let mut w = crate::xml::XmlWriter::new();
    w.start_ns("CreateBucketConfiguration", crate::xml::S3_NS);
    w.element("LocationConstraint", config.region);
    w.end();
    w.finish().into_bytes()
};
```

## AWS SDK との差分

- aws-sdk-rust は `create_bucket_configuration` を `Option` として利用者入力で受け取り、`None` のときは空ボディを送信する
- 自動補完は行わない

## 論点

- LocationConstraint を省略すると us-east-1 にバケットが作成されるため、自動付与は利用者を助ける
- しかし aws-sdk-rust 互換の観点では独自仕様であり、利用者が新しく覚えるコストが発生する
- 互換性を優先するなら `create_bucket_configuration` または `location_constraint` をオプションフィールドとして追加し、利用者に明示的に指定させるべき

## 現在の実装箇所

- `src/api/create_bucket.rs:37-44` — `build_request` 内で `config.region != "us-east-1"` の場合に `XmlWriter` で `CreateBucketConfiguration` XML を自動生成している
- `CreateBucketFluentBuilder` は `bucket` フィールドのみで `location_constraint` や `create_bucket_configuration` フィールドを持たない

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucket.html

## 解決方法

aws-sdk-rust 互換を優先し、自動付与を廃止した。

- `CreateBucketConfiguration` 型を `types.rs` に追加（`location_constraint: Option<String>`）
- `CreateBucketFluentBuilder` に `create_bucket_configuration()` メソッドを追加
- 自動 XML 生成を削除し、利用者が明示的に `CreateBucketConfiguration` を渡す設計に変更
- s3cli (`cmd_mb`) でリージョンが `us-east-1` 以外の場合に `CreateBucketConfiguration` を設定するよう修正

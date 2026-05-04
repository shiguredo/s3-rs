# DeleteObjects の入力を Delete 構造体経由に変更する

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- 現状の `DeleteObjectsFluentBuilder` は `.bucket(...)`, `.object(ObjectIdentifier)`, `.quiet(bool)`, `.checksum_algorithm(...)` の形でビルダーに直接フィールドを足している (`src/api/delete_objects.rs:14-54`)。
- aws-sdk-rust の対応 API は `.delete(Delete)` という単一メソッドで、`Delete { objects: Vec<ObjectIdentifier>, quiet: Option<bool> }` を受ける構造になっている。
- AWS S3 公式 API でも、`DeleteObjects` のリクエストボディは `<Delete>` 要素配下に `<Object>` と `<Quiet>` を持つ XML 構造であり、aws-sdk-rust は API ドキュメントの構造を素直にミラーリングしている。
- `AGENTS.md` / `CLAUDE.md` 「API 名、メソッド名、型名、フィールド名は aws-sdk-rust に合わせる」「Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先」の方針に整合させる。
- 利用者が aws-sdk-rust から移行する際に、`.delete(Delete::builder().objects(...).quiet(...).build())` の書き換えが必要になり違和感が大きい。
- 現在の利用箇所は `examples/s3cli/src/ops.rs:380`, `tests/minio.rs:624`, `tests/rustfs.rs:565` の 3 箇所のみで、移行コストは限定的。

## 変更内容

### 1. `Delete` 型の新設

`src/types.rs` に以下を追加する。

```rust
/// DeleteObjects のリクエストボディ
#[derive(Debug, Clone)]
pub struct Delete {
    /// 削除対象オブジェクトの一覧
    pub objects: Vec<ObjectIdentifier>,
    /// quiet モードを有効にすると、エラーのあったオブジェクトのみレスポンスに含まれる
    pub quiet: Option<bool>,
}

impl Delete {
    pub fn builder() -> DeleteBuilder {
        DeleteBuilder::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeleteBuilder {
    objects: Vec<ObjectIdentifier>,
    quiet: Option<bool>,
}

impl DeleteBuilder {
    /// 削除対象オブジェクトを追加する
    pub fn objects(mut self, object: ObjectIdentifier) -> Self {
        self.objects.push(object);
        self
    }

    /// 削除対象オブジェクトの配列を設定する
    pub fn set_objects(mut self, objects: Vec<ObjectIdentifier>) -> Self {
        self.objects = objects;
        self
    }

    /// quiet モードを設定する
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = Some(quiet);
        self
    }

    /// Delete を構築する
    pub fn build(self) -> Delete {
        Delete {
            objects: self.objects,
            quiet: self.quiet,
        }
    }
}
```

aws-sdk-rust と同じ命名規則で、`objects()` (単数追加) と `set_objects()` (一括設定) の両方を提供する。

### 2. `DeleteObjectsFluentBuilder` の刷新

`src/api/delete_objects.rs:14-21` を以下に変更する。

```rust
pub struct DeleteObjectsFluentBuilder<'a> {
    client: &'a S3Client,
    bucket: Option<String>,
    delete: Option<Delete>,
    checksum_algorithm: Option<ChecksumAlgorithm>,  // issue 0059 で型化
}

impl<'a> DeleteObjectsFluentBuilder<'a> {
    pub fn delete(mut self, delete: Delete) -> Self {
        self.delete = Some(delete);
        self
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self { ... }

    pub fn checksum_algorithm(mut self, algorithm: impl Into<ChecksumAlgorithm>) -> Self { ... }

    pub fn build_request(&self, now: SystemTime) -> Result<S3Request, Error> { ... }
}
```

### 3. 旧メソッドの削除

- `DeleteObjectsFluentBuilder::object(ObjectIdentifier)` を削除する。
- `DeleteObjectsFluentBuilder::quiet(bool)` を削除する。
- 互換 alias は提供しない (破壊的変更)。

### 4. 利用箇所の書き換え

旧:
```rust
let mut builder = client.delete_objects().bucket(bucket);
for key in &keys {
    builder = builder.object(ObjectIdentifier { key: key.clone(), version_id: None });
}
let request = builder.build_request().unwrap();
```

新:
```rust
let delete = Delete::builder()
    .set_objects(
        keys.iter()
            .map(|k| ObjectIdentifier { key: k.clone(), version_id: None })
            .collect(),
    )
    .build();
let request = client
    .delete_objects()
    .bucket(bucket)
    .delete(delete)
    .build_request(now)
    .unwrap();
```

### 5. `lib.rs` の `pub use` 追加

```rust
pub use types::{Delete, DeleteBuilder, ObjectIdentifier};
```

## AWS S3 API Reference

- [DeleteObjects](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html)

  > In the request, you must include an XML document with the keys of the objects that you want to delete. You can specify a maximum of 1,000 keys. The XML document must specify a Delete element that contains a list of Object elements (one per key to be deleted), and an optional Quiet element.

  > Quiet - Element to enable quiet mode for the request. When you add this element, you must set its value to true. Type: Boolean.

  > Object - Container element that describes the delete request for an object. Required: Yes.

  XML 構造としては以下:

  > ```xml
  > <Delete xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  >    <Object>
  >       <Key>string</Key>
  >       <VersionId>string</VersionId>
  >    </Object>
  >    ...
  >    <Quiet>boolean</Quiet>
  > </Delete>
  > ```

  この XML 構造を Rust で表現するなら、ルート要素 `<Delete>` に対応する `Delete` 構造体を持つのが最も自然であり、aws-sdk-rust もそれを採用している。

## 影響範囲

- `src/api/delete_objects.rs:14-141` の `DeleteObjectsFluentBuilder` 全面書き換え。
- `src/types.rs` に `Delete`, `DeleteBuilder` 追加。
- `examples/s3cli/src/ops.rs:380` の呼び出し書き換え。
- `tests/minio.rs:624-630` の呼び出し書き換え。
- `tests/rustfs.rs:565` の呼び出し書き換え。

## 依存関係

- 本 issue は issue 0059 (`ChecksumAlgorithm` enum 化) に依存する。`checksum_algorithm` フィールドの型を文字列から enum に揃えるため。

## 優先度

中 (利用箇所が少ないため移行コストは小、ただし破壊的変更のため早期に実施したい)

## CHANGES.md への記載

- `[CHANGE] DeleteObjects の入力を Delete 構造体経由に変更する`
- `[ADD] Delete / DeleteBuilder 型を追加する`
- `[CHANGE] DeleteObjectsFluentBuilder::object / quiet メソッドを削除する`

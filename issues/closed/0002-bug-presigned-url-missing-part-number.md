# GetObject と HeadObject の presigned() で partNumber が欠落する

## 優先度

P2

## 概要

`build_request()` では `part_number` が指定されていれば `partNumber` クエリパラメータを追加するが、
`presigned()` では常に `&[]` を渡しており `part_number` を完全に無視している。

## 影響

- `part_number` を指定して presigned URL を生成しても、生成された URL にはパラメータが含まれない
- 生成された URL は意図と異なるリソース (オブジェクト全体) を指す

## 該当箇所

- `src/api/get_object.rs:111` - `presigned()` で `&[]` を渡している
- `src/api/head_object.rs:107` - 同様

## 修正方針

`presigned()` 内で `self.part_number` があれば `("partNumber", ...)` を `extra_query_params` に含める。

## 完了

- `GetObjectFluentBuilder::presigned` で `self.part_number` があれば `("partNumber", ...)` を `extra_query_params` に含めるよう修正
- `HeadObjectFluentBuilder::presigned` で同様に修正

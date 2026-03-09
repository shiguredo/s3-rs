# HeadBucket の parse_response が失敗ステータスでも Ok を返す

## 概要

`HeadBucket` の `parse_response` は HTTP ステータスコードに関係なく `Ok` を返す。
S3 API の `HeadBucket` は成功時 `200 OK`、失敗時 `400` / `403` / `404` を返す API であり、
現実装は API の成功/失敗の意味論と異なる。

## 現状の設計意図

`x-amz-bucket-region` header は `200` / `301` / `403` いずれでも返されるため、
region 探索用途でエラーステータスでもこの header を取得したいという事情がある。
`HeadBucketOutput` に `status_code` を含め、呼び出し側で判断する設計になっている。

## 仕様差分

AWS 公式の `HeadBucket` は存在確認・権限確認用の API であり、
bucket が無い / 権限が無い場合はエラーを返すのが正しい意味論。
現実装は「意図的設計」だが「S3 API の意味論とは異なる」仕様差分として残る。

## 設計判断

通常の `parse_response` は非 200 でエラーを返し、
region 探索用途には専用メソッド (例: `parse_response_with_region`) を分けるのが筋。

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadBucket.html

## 解決方法

aws-sdk-rust の設計に合わせて修正した。

1. `parse_response` は非 200 で `head_error_from_status` を使い `Err` を返すようにした (HeadObject と同じパターン)
2. `HeadBucketOutput` から `status_code` フィールドを削除し、`bucket_region` のみ持たせた
3. region 探索用の別メソッドは作らない — aws-sdk-rust と同様に region 探索はユーザー API の責務ではない

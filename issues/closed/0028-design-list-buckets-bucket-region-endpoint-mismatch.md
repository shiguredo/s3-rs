# ListBuckets の bucket-region と endpoint region の整合チェックがない

## 概要

`ListBucketsFluentBuilder` の `bucket_region` setter は単にクエリパラメータを付与するだけで、
接続先 endpoint の region との整合チェックがない。
`service_host` のデフォルトは `s3.{region}.amazonaws.com` (regional endpoint) であり、
`client.region = us-east-1` かつ `bucket_region = ap-northeast-1` のような
unsupported な request を普通に組み立てられる。

## 仕様根拠

AWS 公式は `bucket-region` について「bucket-region と異なる Regional endpoint への
request はサポートされない」と明記している。

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html

## 設計判断

以下のいずれかの対応が考えられる:

1. `bucket_region` が設定されている場合、`config.region` との一致を検証してエラーにする
2. `bucket_region` が設定されている場合、endpoint を自動的に合わせる
3. ドキュメントで注意喚起し、ライブラリ側では制約を課さない

互換オブジェクトストレージではこの制約が当てはまらない場合もあるため、
endpoint (custom) が設定されている場合はチェックをスキップする等の考慮が必要。

## 解決方法

aws-sdk-rust を調査した結果、aws-sdk-rust も bucket-region と endpoint region の整合チェックを実装していないことが判明した。
AWS ドキュメントには警告があるが、SDK 側では検証しない設計を採用している。
現在の s3-rs の実装は aws-sdk-rust と同じ挙動であり、仕様差分ではないためそのままクローズする。

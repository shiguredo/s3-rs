# ドット付きバケット名で HTTPS が壊れる

## 優先度

P2

## 概要

`use_path_style = false` のとき、バケット名に `.` を含む場合（例: `foo.bar`）、
仮想ホスト形式で `foo.bar.s3.us-east-1.amazonaws.com` というホスト名が生成される。
AWS のワイルドカード証明書 `*.s3.us-east-1.amazonaws.com` は単一レベルのサブドメインにしかマッチしないため、
TLS 証明書検証が失敗する。

## 影響

- ドット付きバケット名で HTTPS 接続が不可能

## 該当箇所

- `src/api/mod.rs` - `host_for_bucket` 関数
- `src/api/mod.rs` - `path_for_key` 関数

## 修正方針

1. HTTPS かつバケット名に `.` を含む場合は path-style にフォールバックする
2. `host_for_bucket` と `path_for_key` の両方で同じ判定ロジックを使う
3. 判定を共通化するヘルパー関数を導入する

## 完了

- `use_path_style_for_bucket` ヘルパー関数を導入し、HTTPS かつドット付きバケットで path-style にフォールバック
- `host_for_bucket` と `path_for_key` の両方で同じ判定を使用

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/userguide/bucketnamingrules.html

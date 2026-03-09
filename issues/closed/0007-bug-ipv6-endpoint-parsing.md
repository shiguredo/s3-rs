# IPv6 エンドポイントを誤って分解する

## 優先度

P2

## 概要

`extract_connect_host` が `split(':')` でホストとポートを分離しているため、
`[::1]:9000` のような IPv6 エンドポイントでホスト名が `[` になる。
`extract_port` も `rsplit(':')` を使っており、IPv6 アドレスのコロンと混同する。

## 影響

- IPv6 環境の S3 互換サービス（MinIO 等）で接続失敗

## 該当箇所

- `src/api/mod.rs` - `extract_connect_host` 関数
- `src/api/mod.rs` - `extract_port` 関数

## 修正方針

1. `[...]` で囲まれた IPv6 アドレスを正しく解析する
2. `[::1]:9000` → host=`[::1]`, port=`9000` のように分解する
3. ポートなしの `[::1]` も正しく処理する

## 完了

- `extract_connect_host` を IPv6 ブラケット記法対応に書き換え
- `extract_port` を `parse_port_from_authority` ヘルパーに分離して IPv6 対応
- `[::1]:9000` → host=`[::1]`, port=`9000` を正しく処理

## 参考

- RFC 3986 Section 3.2.2 (Host)
- RFC 6874 (IPv6 Zone IDs in URIs)

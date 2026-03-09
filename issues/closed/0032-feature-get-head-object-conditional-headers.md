# GetObject / HeadObject に条件付きリクエストヘッダーを追加する

## 概要

GetObject と HeadObject に AWS S3 の条件付きリクエストヘッダー (`If-Match`, `If-None-Match`, `If-Modified-Since`, `If-Unmodified-Since`) を追加する。

aws-sdk-rust では標準的にサポートされているパラメータであり、キャッシュ制御や楽観的ロックなど基本的なユースケースで必要になる。

## 追加するパラメータ

| パラメータ | HTTP ヘッダー | 型 | 説明 |
|-----------|-------------|---|------|
| `if_match` | `If-Match` | `String` (ETag 値) | ETag が一致する場合のみ返す。不一致なら 412 |
| `if_none_match` | `If-None-Match` | `String` (ETag 値) | ETag が異なる場合のみ返す。一致なら 304 |
| `if_modified_since` | `If-Modified-Since` | `HttpDate` (IMF-fixdate) | 指定時刻以降に変更されていれば返す。未変更なら 304 |
| `if_unmodified_since` | `If-Unmodified-Since` | `HttpDate` (IMF-fixdate) | 指定時刻以降に変更されていなければ返す。変更済みなら 412 |

## 設計判断

### 日時型

`HttpDate` 型を新設した。RFC 9110 Section 5.6.7 の IMF-fixdate 形式に対応する。

- `HttpDate::from_unix_timestamp(secs)` — UNIX タイムスタンプから生成
- `HttpDate::from_imf_fixdate(s)` — S3 レスポンスの `Last-Modified` ヘッダー値を再利用可能

### 304 / 412 レスポンス

`Error` 型に専用バリアントを追加した。

- `Error::NotModified` — 304 Not Modified
- `Error::PreconditionFailed` — 412 Precondition Failed

## ユースケース

- **キャッシュ制御**: `If-None-Match` + ETag で変更がなければボディ転送を省略
- **差分同期**: `If-Modified-Since` で変更のあったオブジェクトのみ取得
- **楽観的ロック**: `If-Match` で読み取り後に他者が変更していないことを検証

## 解決方法

- `types.rs` に `HttpDate` 型を追加 (RFC 9110 Section 5.6.7 IMF-fixdate)
- `error.rs` に `Error::NotModified`, `Error::PreconditionFailed` バリアントを追加
- `GetObjectFluentBuilder` / `HeadObjectFluentBuilder` に 4 種の条件付きヘッダーメソッドを追加
- `parse_response` で 304/412 を専用バリアントとして返すように変更
- `head_error_from_status` も 304/412 で専用バリアントを返すように変更

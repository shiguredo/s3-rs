# GetObject の response_* オーバーライドパラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

GetObject の Presigned URL でレスポンスヘッダーを上書きするための `response-*` クエリパラメータに対応する。

## 対象パラメータ

- `response_cache_control`
- `response_content_disposition`
- `response_content_encoding`
- `response_content_language`
- `response_content_type`
- `response_expires`

## 用途

Presigned URL 経由でオブジェクトをダウンロードする際に、Content-Disposition や Content-Type を上書きすることでブラウザのダウンロード挙動を制御できる。

## 優先度

低

## 解決方法

`GetObjectFluentBuilder` に 6 つの `response_*` フィールドとセッターメソッドを追加し、`build_request` と `presigned` の両方でクエリパラメータとして出力するようにした。

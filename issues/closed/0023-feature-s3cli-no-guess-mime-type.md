# s3cli に --no-guess-mime-type オプションを追加する

## 概要

cp / sync サブコマンドに `--no-guess-mime-type` オプションを追加する。

## 動作

- デフォルトではファイル拡張子から MIME タイプを推測して `Content-Type` を設定する
- `--no-guess-mime-type` を指定すると MIME タイプの推測を行わず `application/octet-stream` を使用する

## 備考

- クライアント側の機能であり S3 API には依存しない
- 現状の s3cli は MIME タイプの自動推測を行っていないため、まず自動推測機能を実装してからこのオプションを追加する

## 解決方法

- `guess_mime_type()` で拡張子ベースの MIME タイプ推測を実装した
- `resolve_content_type()` で明示指定 > MIME 推測 > None の優先順を制御する
- cp コマンドに `--no-guess-mime-type` オプションを追加した
- `upload_recursive` でも各ファイルごとに MIME タイプを推測するようにした
- mv コマンドでもアップロード時に MIME タイプを推測するようにした

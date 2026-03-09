# s3cli に --quiet / --only-show-errors オプションを追加する

## 概要

cp / mv / rm / sync サブコマンドに出力制御オプションを追加する。

## オプション

- `--quiet` — 操作メッセージを全て非表示にする
- `--only-show-errors` — エラーメッセージのみ表示する
- `--no-progress` — 進捗表示を抑制する（将来進捗バーを追加した場合に使用する）

## 解決方法

- `--quiet` と `--only-show-errors` フラグを cp / mv / rm サブコマンドに追加した
- `upload_file`、`download_file` に `quiet: bool` パラメータを追加した
- `RecursiveUploadParams` に `quiet` フィールドを追加した
- `download_recursive`、`copy_recursive`、`delete_recursive` に `quiet` パラメータを追加した
- quiet が true の場合、全ての eprintln 操作メッセージを抑制する
- `--no-progress` は将来進捗バーを追加した際に実装する

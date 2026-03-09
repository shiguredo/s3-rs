# s3cli に sync サブコマンドを追加する

## 概要

`aws s3 sync` 相当の差分同期機能を s3cli に追加する。

## 対応パス形式

- `<LocalPath> <S3Uri>`
- `<S3Uri> <LocalPath>`
- `<S3Uri> <S3Uri>`

## 必須オプション

- `--delete` — 送信先にのみ存在するファイルを削除する
- `--size-only` — サイズのみで同期判定する（タイムスタンプを無視する）
- `--exact-timestamps` — タイムスタンプが完全一致した場合のみスキップする
- `--dryrun` — 実行内容をプレビューする（実際には実行しない）
- `--exclude` / `--include` — ファイルフィルタリング（0014 と連動）

## 同期ロジック

1. 送信元と送信先のファイルリストをソート済みで取得する
2. キーを比較して以下を判定する:
   - 同一キー: サイズと LastModified で比較し、差分があれば転送する
   - 送信元のみ: 転送する
   - 送信先のみ: `--delete` 指定時のみ削除する

## 備考

- aws s3 sync の中核機能であり優先度が高い
- マルチパートアップロード対応は既存の `upload_multipart` を流用する

## 解決方法

- `cmd_sync` 関数を追加し、ローカル→S3 / S3→ローカル / S3→S3 の3パターンに対応した
- `collect_local_files` でローカルファイルのパス・サイズ・更新日時をソート済みで収集する
- `collect_s3_objects` で S3 オブジェクトのキー・サイズ・LastModified をソート済みで収集する
- `should_skip_sync` で差分判定する（サイズ比較 + タイムスタンプ比較）
- `parse_s3_timestamp` で ISO 8601 形式の日時を SystemTime に変換する
- `--delete` で送信先にのみ存在するファイルを削除する
- `--size-only` でサイズのみ比較する
- `--exact-timestamps` でタイムスタンプ完全一致時のみスキップする
- `--dryrun` / `--quiet` / `--exclude` / `--include` も対応した

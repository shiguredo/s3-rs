# s3cli に --follow-symlinks / --no-follow-symlinks オプションを追加する

## 概要

cp / sync サブコマンドの再帰的アップロード時にシンボリックリンクの扱いを制御するオプションを追加する。

## オプション

- `--follow-symlinks` — シンボリックリンクを辿る（デフォルト）
- `--no-follow-symlinks` — シンボリックリンクをスキップする

## 備考

- クライアント側の機能であり S3 API には依存しない
- 現状の再帰走査はシンボリックリンクを考慮していない

## 解決方法

- `upload_recursive` に `follow_symlinks` パラメータを追加した
- シンボリックリンクかつ `follow_symlinks` が false の場合はスキップする
- cp コマンドに `--no-follow-symlinks` オプションを追加した（デフォルトは follow）

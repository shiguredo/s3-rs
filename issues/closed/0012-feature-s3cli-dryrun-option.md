# s3cli に --dryrun オプションを追加する

## 概要

cp / mv / rm / sync サブコマンドに `--dryrun` オプションを追加する。

## 動作

- 実際の転送・削除を行わず、実行予定の操作を標準出力に表示する
- aws s3 と同様の出力形式にする:
  - `(dryrun) upload: ./file.txt to s3://bucket/file.txt`
  - `(dryrun) copy: s3://src/key to s3://dst/key`
  - `(dryrun) delete: s3://bucket/key`

## 備考

- sync コマンド（0010）と合わせて対応するのが効率的

## 解決方法

- `--dryrun` フラグを cp / mv / rm サブコマンドに追加した
- dryrun 時は実際の I/O を行わず `(dryrun)` プレフィックス付きのメッセージを表示する
- upload_file、download_file に `dryrun: bool` パラメータを追加し、早期リターンする
- RecursiveUploadParams に `dryrun` フィールドを追加した
- download_recursive、copy_recursive、delete_recursive にも `dryrun` パラメータを追加した
- mv コマンドでは dryrun 時にソースの削除もスキップする

# s3cli に --exclude / --include フィルタを追加する

## 概要

cp / mv / rm / sync サブコマンドに `--exclude` / `--include` パターンフィルタを追加する。

## 動作

- fnmatch ベースのグロブパターンでファイルをフィルタリングする
- ルールは指定順序が重要で、後の指定が前の指定を上書きする
- デフォルトでは全ファイルが対象

## 使用例

```bash
# .txt ファイルのみを対象にする
s3cli cp . s3://bucket/ --recursive --exclude "*" --include "*.txt"

# .log ファイルを除外する
s3cli sync . s3://bucket/ --exclude "*.log"
```

## 備考

- sync コマンド（0010）では必須の機能
- 複数回指定可能にする

## 解決方法

- `FilterRule` enum (`Exclude`/`Include`) と `should_include` 関数で fnmatch ベースのフィルタリングを実装した
- `parse_filters` 関数で `--exclude`/`--include` オプションを指定順序を保持して複数回パースする
- `RecursiveUploadParams` に `filters` フィールドを追加し、`upload_recursive` でファイルごとにフィルタを適用する
- `download_recursive`、`copy_recursive`、`delete_recursive` にも `filters` パラメータを追加してフィルタを適用する
- `cmd_cp`、`cmd_mv`、`cmd_rm` の各サブコマンドで `--exclude`/`--include` オプションをサポートする

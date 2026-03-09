# s3cli

shiguredo_s3 の Sans I/O API を使った aws s3 互換の CLI ツールです。

I/O 層の実装に tokio + rustls + shiguredo_http11 を使用しています。

## 依存ライブラリ

| ライブラリ | 用途 |
|---|---|
| shiguredo_s3 | S3 API (Sans I/O) |
| shiguredo_http11 | HTTP/1.1 リクエストのエンコード / レスポンスのデコード |
| tokio | 非同期ランタイム、TCP 通信、ファイル I/O |
| rustls | TLS |
| tokio-rustls | tokio + rustls の統合 |
| rustls-platform-verifier | OS のルート証明書を使った証明書検証 |
| noargs | コマンドライン引数パーサ |

## サブコマンド

| サブコマンド | 説明 |
|---|---|
| `cp` | ファイルのコピー (ローカル <-> S3、S3 <-> S3) |
| `mv` | ファイルの移動 (ローカル <-> S3、S3 <-> S3) |
| `ls` | S3 オブジェクトの一覧表示 |
| `rm` | S3 オブジェクトの削除 |
| `presign` | Presigned URL の生成 |

## 環境変数

| 環境変数 | 説明 |
|---|---|
| `AWS_ACCESS_KEY_ID` | アクセスキー ID (必須) |
| `AWS_SECRET_ACCESS_KEY` | シークレットアクセスキー (必須) |
| `AWS_DEFAULT_REGION` | リージョン (デフォルト: `ap-northeast-1`) |
| `AWS_ENDPOINT_URL_S3` | カスタムエンドポイント |
| `S3CLI_PATH_STYLE` | `1` でパススタイルアクセスを有効化 |

## 使い方

```bash
# ファイルのアップロード
s3cli cp local-file.txt s3://my-bucket/path/to/file.txt

# ファイルのダウンロード
s3cli cp s3://my-bucket/path/to/file.txt local-file.txt

# ディレクトリの再帰アップロード
s3cli cp ./local-dir s3://my-bucket/prefix --recursive

# オブジェクト一覧
s3cli ls s3://my-bucket/prefix --recursive --human-readable --summarize

# オブジェクトの削除
s3cli rm s3://my-bucket/path/to/file.txt

# 再帰削除
s3cli rm s3://my-bucket/prefix --recursive

# Presigned URL の生成
s3cli presign s3://my-bucket/path/to/file.txt --expires-in 3600
```

## 特徴

- 8 MB 以上のファイルは自動的にマルチパートアップロードを使用する
- マルチパートアップロードは Semaphore + JoinSet による並行アップロード (デフォルト 8 並行)
- アップロード失敗時は AbortMultipartUpload で中止する
- 再帰操作は ListObjectsV2 のページネーションに対応
- 再帰削除は DeleteObjects で 1000 件ずつ一括削除する

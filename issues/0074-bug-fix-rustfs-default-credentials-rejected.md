# RustFS 統合テストが `rustfs/rustfs:latest` の更新でデフォルト資格情報拒否により全件失敗する

- Created: 2026-05-22
- Model: Opus 4.7 1M

## 概要

`tests/rustfs.rs` の RustFS 統合テスト 18 件が、`rustfs/rustfs:latest` の 2026-05-21 ビルドで全件失敗するようになった。コンテナ起動直後に `[FATAL]` で exit するため、testcontainers 側では `WaitContainer(StartupTimeout)` もしくは `WaitContainer(HttpWait(NoExposedPortsForHttpWait))` として観測される。

## 根本原因

RustFS の新バージョンで「`rustfsadmin` / `rustfsadmin` というデフォルト資格情報を non-loopback リスナーで使うことを禁止する」セキュリティチェックが追加された。`RUSTFS_ACCESS_KEY` / `RUSTFS_SECRET_KEY` 環境変数で明示的に `rustfsadmin` を渡してもデフォルト値と判定されて拒否される。

コンテナログ:

```
Initializing data directories: /data
Initializing log directory: /logs
!!!WARNING: Default credentials are only allowed on loopback or with explicit insecure local-dev opt-in.
Starting: /usr/bin/rustfs  /data
[FATAL] Server encountered an error and is shutting down: Default root credentials are not allowed on non-loopback listeners; set RUSTFS_ACCESS_KEY and RUSTFS_SECRET_KEY to non-default values, bind to loopback, or set RUSTFS_ALLOW_INSECURE_DEFAULT_CREDENTIALS=true for local development only
```

## 再現手順

1. ローカルの `rustfs/rustfs:latest` イメージを削除する: `docker image rm rustfs/rustfs:latest`
2. `cargo test --test rustfs` を実行する
3. 18 件すべてが `failed to start RustFS container: WaitContainer(...)` で panic する

## 影響範囲

- スケジュール実行の CI ジョブ「RustFS integration tests」が 2026-05-21 03:17 UTC の run (`26203329132`) から失敗継続
- ローカルでは古い `latest` イメージがキャッシュされている開発者には再現しない
- 機能的なリグレッションはなく、テスト環境側の問題

## 関連情報

- 失敗した CI run: <https://github.com/shiguredo/s3-rs/actions/runs/26203329132>
- 該当箇所: `tests/rustfs.rs:51`, `tests/rustfs.rs:55`（`ACCESS_KEY` / `SECRET_KEY` 定数）
- 上流の `docker-compose.yml` では `devadmin` / `devadmin` が使われている

## 対応方針

`tests/rustfs.rs` の `ACCESS_KEY` / `SECRET_KEY` 定数を `rustfsadmin` から `devadmin` に変更し、上流リポジトリの `docker-compose.yml` で使用されている値に揃える。`RUSTFS_ALLOW_INSECURE_DEFAULT_CREDENTIALS=true` での回避策は採用しない（unsafe opt-in 依存を増やさないため）。

## 補足

本 issue は AWS S3 API 仕様ではなく RustFS テスト基盤の問題のため、`AWS S3 API Reference` の URL は記載しない。

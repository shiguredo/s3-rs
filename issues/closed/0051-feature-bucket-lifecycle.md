# GetBucketLifecycleConfiguration / PutBucketLifecycleConfiguration / DeleteBucketLifecycle の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- ログ・バックアップ等の **自動失効や階層移動**は実運用の基本であり、バケットライフサイクル API が必要になる。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットのライフサイクル設定（期限切れ・ストレージクラス遷移等）の取得・設定・削除を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketLifecycleConfigurationFluentBuilder` / `PutBucketLifecycleConfigurationFluentBuilder` / `DeleteBucketLifecycleFluentBuilder` を新規作成
- `LifecycleRule`, `LifecycleRuleFilter`, `LifecycleRuleAndOperator`, `LifecycleExpiration`, `LifecycleTransition`, `NoncurrentVersionExpiration`, `NoncurrentVersionTransition`, `AbortIncompleteMultipartUpload` 型を `types.rs` に追加
- XML のネスト構造 (Rule > Filter > And, Rule > Expiration, Rule > Transition 等) を EventReader ベースのステートマシンでパースする実装
- `PutBucketLifecycleConfigurationFluentBuilder` に `transition_default_minimum_object_size` ヘッダー対応を追加
- `S3Client` にファクトリメソッドを追加
- fuzz ターゲットに `GetBucketLifecycleConfigurationFluentBuilder` を追加

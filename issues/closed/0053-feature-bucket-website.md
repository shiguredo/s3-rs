# GetBucketWebsite / PutBucketWebsite / DeleteBucketWebsite の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- 静的サイト配信・ドキュメント公開で **ウェブサイトエンドポイント設定**をバケット単位で管理する需要がある。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

静的ウェブサイトホスティング（インデックスドキュメント・エラードキュメント等）の取得・設定・無効化を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketWebsiteFluentBuilder` / `PutBucketWebsiteFluentBuilder` / `DeleteBucketWebsiteFluentBuilder` を新規作成
- `IndexDocument`, `ErrorDocument`, `RedirectAllRequestsTo`, `RoutingRule`, `RoutingRuleCondition`, `RoutingRuleRedirect` 型を `types.rs` に追加
- WebsiteConfiguration XML のネスト構造 (RoutingRules > RoutingRule > Condition/Redirect) を EventReader でパース
- `S3Client` にファクトリメソッドを追加
- fuzz ターゲットに `GetBucketWebsiteFluentBuilder` を追加

# String 型 Output / 設定フィールドの enum 化

- Priority: Medium
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/add-typed-enums

## 目的

issue 0059 で主要 8 種を enum 化したが、Object Lock / Versioning / Ownership 等の String フィールドが残っている。aws-sdk-rust 互換の型安全性を完成させる。

## 優先度根拠

無効な文字列が builder / parse 段階で検出されず、S3 サーバーエラーまで到達する。型付けは移行時の IDE 支援にも効く。

## 現状

| フィールド | 現状 |
|-----------|------|
| `OwnershipControlsRule.object_ownership` | `String` |
| `GetBucketVersioningOutput.status` | `Option<String>` |
| `GetBucketVersioningOutput.mfa_delete` | `Option<String>` |
| `ObjectLockLegalHold.status` | `String` |
| `ObjectLockRetention.mode` | `String` |
| `DefaultRetention.mode` | `String` |
| `GetObjectLegalHoldOutput` | 空 Status で `Some` 生成しうる |

追加検討: `RequestCharged`, `ChecksumType`, `BucketLocationType`, `TransitionDefaultMinimumObjectSize`, `ObjectOwnership`, `BucketVersioningStatus`, `MfaDeleteStatus`

## 設計方針

- aws-sdk-rust の enum 名・variants に合わせる
- `From<&str>` / `as_str()` を提供
- builder に `set_*` バリアントを追加
- 無効値は `InvalidInput` または parse 時 `InvalidResponse`

## AWS S3 API Reference

- GetBucketVersioning: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketVersioning.html>
- GetBucketOwnershipControls: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketOwnershipControls.html>

> Status: The versioning state of the bucket. Valid Values: Enabled | Suspended

## 完了条件

- 上記 String フィールドが型付き enum になる
- `GetObjectLegalHold` が空 Status を `None` 扱いする
- CHANGES.md に `[CHANGE]` エントリ（String → enum）
- 統合テストが通る

## 解決方法

1. `types.rs` に enum 定義
2. builder / parse / XML serialize を更新
3. `lib.rs` re-export 追加

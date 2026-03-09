# PutBucketVersioning の MfaDelete / x-amz-mfa 未対応

## 分類

機能不足

## 概要

`PutBucketVersioningFluentBuilder` は `status` のみ対応しており、`MfaDelete` フィールドと `x-amz-mfa` ヘッダーを送信できない。

## AWS SDK との差分

- aws-sdk-rust は `VersioningConfiguration` に `mfa_delete` フィールドを持ち、`PutBucketVersioningInput` に `mfa` ヘッダーも対応している
- 公式 API でも `MfaDelete` は deprecated ではなく現行機能

## pending にした理由

- MFA Delete はバケットオーナーかつ MFA デバイス必須という特殊な運用要件があり、即時対応の優先度は低い
- 互換性不足は事実だが、利用頻度を考慮して保留とする

## 対応方針

必要になった時点で以下を追加する:

- `PutBucketVersioningFluentBuilder` に `mfa_delete` フィールド追加
- `PutBucketVersioningFluentBuilder` に `mfa` フィールド追加（`x-amz-mfa` ヘッダー用）
- `VersioningConfiguration` の XML に `MfaDelete` 要素を条件付きで出力

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketVersioning.html

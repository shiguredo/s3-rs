# SigV4 の canonical header で連続空白を畳んでいない

## 優先度

P2

## 概要

SigV4 仕様では canonical header の値について、前後の空白を trim するだけでなく、
値の中間にある連続する空白を単一のスペースに畳む必要がある。
現実装は `trim()` のみで連続空白の畳み込みを行っていない。

## 影響

- ヘッダー値に連続空白を含む場合 (例: `Content-Disposition: attachment;  filename="test"`) に署名不一致を起こす
- 通常の S3 利用では稀だが仕様逸脱

## 該当箇所

- `src/signing.rs:189` - `compute_authorization` 内の canonical header 構築
- `src/signing.rs:255` - `compute_presigned_signature` 内の canonical header 構築

## 修正方針

`value.trim()` に加えて、連続する空白文字を単一のスペースに置換する処理を追加する。

## 参考

- https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html

## 完了

- `signing.rs` に `fold_whitespace()` 関数を追加。前後の空白を trim し、連続する ASCII 空白文字を単一のスペースに畳む
- `compute_authorization` と `compute_presigned_signature` の canonical header 構築で `value.trim()` を `fold_whitespace(value)` に置換

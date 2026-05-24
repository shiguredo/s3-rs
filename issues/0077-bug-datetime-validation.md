# datetime モジュールの日時検証・オーバーフロー不備

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-datetime-validation

## 目的

`src/datetime.rs` と `validate_imf_fixdate` が不正日時を黙って受理したり、極端な入力でオーバーフローする。S3 レスポンス日時の解釈ミスやパニックを防ぐ。

## 優先度根拠

XML レスポンスの `LastModified` / `CreationDate` 等は `parse_iso8601` / `parse_imf_fixdate` 経由で多数の API が利用する。2 月 31 日のような存在しない日付が別日付に正規化されると、条件付きリクエストや表示が誤る。

## 現状

確認済みの問題:

1. **`unix_timestamp_from_civil`**: `day <= 31` のみ検証。`2024-02-31` が `2024-03-02` に正規化される
2. **`parse_iso8601`**: `"2024-01-15T12:30:45abcZ"` のような Z 直前ゴミを受理しうる
3. **`civil_from_unix_timestamp`**: `u64::MAX` 付近で `year as i32` がオーバーフローしうる
4. **`unix_timestamp_from_civil`**: `(days as u64) * 86400 + ...` がオーバーフローしうる
5. **閏秒**: `second == 60` を 59 に切り捨て（93-120 行付近）
6. **`validate_imf_fixdate`**: 数値部分・区切り文字の形式検証不足（`"Mon, XY Jan 2024 12:30:45 GMT"` 等）

## 設計方針

- civil → unix 変換後にラウンドトリップ検証し、入力と一致しなければ `InvalidInput`
- 算術は `checked_mul` / `checked_add` を使用
- 実用上限（例: year <= 9999）を設ける
- `second > 59` はエラーに統一（閏秒非対応を doc comment で明記）
- `parse_iso8601` は Z の位置と形式を厳密検証

## AWS S3 API Reference

- GetObject (Last-Modified ヘッダー): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

> Returns the date and time that the object was last modified.

## 完了条件

- 存在しない暦日・不正 ISO8601・極端な timestamp が `Error::InvalidInput` または `InvalidResponse` になる
- 正常系の既知日時テストが通る
- `tests/test_datetime.rs` または PBT で境界値を検証する

## 解決方法

1. `unix_timestamp_from_civil` にラウンドトリップ検証を追加
2. オーバーフロー対策と year 上限を追加
3. `parse_iso8601` / `validate_imf_fixdate` の形式検証を強化
4. 単体テストを `tests/test_datetime.rs` に移行（issue 0085 と連携可）

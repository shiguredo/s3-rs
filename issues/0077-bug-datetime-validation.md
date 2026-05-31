# datetime モジュールの日時検証・オーバーフロー不備

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

`src/datetime.rs` と `validate_imf_fixdate` が不正日時を黙って受理したり、極端な入力でオーバーフローする。S3 レスポンス日時の解釈ミスやパニックを防ぐ。

## 優先度根拠

XML レスポンスの `LastModified` / `CreationDate` 等は `parse_iso8601` / `parse_imf_fixdate` 経由で多数の API が利用する。2 月 31 日のような存在しない日付が別日付に正規化されると、条件付きリクエストや表示が誤る。

## 現状

確認済みの問題:

1. **`unix_timestamp_from_civil`**: `day <= 31` のみ検証。`2024-02-31` が `2024-03-02` に正規化される
2. **`parse_iso8601`**: `s[19..]` に何が来ても `s.ends_with('Z')` だけで通過する。`"2024-01-15T12:30:45abcZ"` のような Z 直前ゴミを受理する
3. **`civil_from_unix_timestamp`**: `secs` が約 5.85 × 10^11 秒以上の場合、`year as i32` がオーバーフローする
4. **`unix_timestamp_from_civil`**: `days` が約 2.13 × 10^14 以上の場合、`(days as u64) * 86400` がオーバーフローする
5. **閏秒**: `second == 60` を 59 に切り捨て（93-120 行付近）
6. **`validate_imf_fixdate`**: 数値部分・区切り文字の形式検証不足（`"Mon, XY Jan 2024 12:30:45 GMT"` 等）

## 設計方針

### 存在しない暦日の検証

`unix_timestamp_from_civil` で civil → unix 変換後に逆変換でラウンドトリップ検証し、入力と一致しなければ `Error::InvalidInput` を返す。これにより `2024-02-31` のような存在しない日付を検出する。

### オーバーフロー対策

算術は `checked_mul` / `checked_add` を使用し、オーバーフロー時に `Error::InvalidInput` を返す。実用上限として `year <= 9999` を設ける（ISO 8601 の 4 桁年形式の上限）。`civil_from_unix_timestamp` の戻り値を `Result<CivilDateTime, Error>` に変更し、出力年の範囲チェックを追加する（`y as i32` のトランケーションを防ぐ）。

### 閏秒の扱い

`second > 59` はエラーに統一する。閏秒非対応であることを doc comment で明記する。これは後方互換のない変更であるため、`CHANGES.md` に `[CHANGE]` として記載する。

### ISO8601 の厳密検証

`parse_iso8601` は Z の位置（`s[19]`）と形式を厳密検証する。S3 は小数秒を返す可能性があるため、`s[19]` が小数点の場合は小数秒部分をスキップし、末尾が `Z` であることを確認する。`s[19..]` が `".{digits}Z"` または `"Z"` 以外の場合はエラーを返す。小数点の後に数字がない場合（`"2024-01-15T12:30:45.Z"`）もエラーとする。

### IMF-fixdate の厳密検証

`validate_imf_fixdate` で数値部分（日付、時間）と区切り文字の形式を厳密検密検証する。検証対象の位置:

- `s[5..7]` (日) が ASCII 数字であること
- `s[7]` がスペースであること（日と月の間）
- `s[8..11]` (月名) が有効な月名であること
- `s[11]` がスペースであること
- `s[12..16]` (年) が ASCII 数字であること
- `s[16]` がスペースであること
- `s[17..19]` (時) が ASCII 数字であること
- `s[19]` が `:` であること
- `s[20..22]` (分) が ASCII 数字であること
- `s[22]` が `:` であること
- `s[23..25]` (秒) が ASCII 数字であること

### エラー種別

datetime モジュールの関数は純粋なパーサーであるため、エラー種別は呼び出し側で決定する。`parse_iso8601` / `parse_imf_fixdate` は `Error::InvalidInput` を返し、S3 レスポンス用の呼び出し側で `map_err` して `Error::InvalidResponse` に変換する。

## AWS S3 API Reference

- GetObject (Last-Modified ヘッダー): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
  - > Returns the date and time that the object was last modified.
- HeadObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html>
- ListObjectsV2: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>
- RFC 9110 Section 5.6.7 (IMF-fixdate): <https://www.rfc-editor.org/rfc/rfc9110#section-5.6.7>
- ISO 8601: <https://www.iso.org/standard/70907.html>

## 完了条件

- 存在しない暦日・不正 ISO8601・極端な timestamp が `Error::InvalidInput` または `InvalidResponse` になる
- 正常系の既知日時テストが通る
- `tests/test_datetime.rs` に単体テストを新規作成し、境界値を検証する
- `pbt/tests/prop_datetime.rs` に PBT テストを新規作成し、ラウンドトリップを検証する

## 解決方法

1. `unix_timestamp_from_civil` にラウンドトリップ検証を追加
2. オーバーフロー対策と year 上限 (9999) を追加
3. `civil_from_unix_timestamp` の戻り値を `Result<CivilDateTime, Error>` に変更し、呼び出し側 (`format_imf_fixdate`, `UtcDateTime::from_unix_timestamp`) を更新
4. `parse_iso8601` の Z 位置検証を強化（小数秒対応、小数点後の数字必須）
5. `validate_imf_fixdate` の数値・区切り検証を強化
6. `second > 59` をエラーに変更し、閏秒非対応を doc comment で明記
7. S3 レスポンス用の呼び出し側で `map_err` して `InvalidInput` → `InvalidResponse` に変換する
8. `format_iso8601` (extended 形式: `YYYY-MM-DDTHH:MM:SSZ`) を新規実装し、PBT ラウンドトリップに使用する
9. `tests/test_datetime.rs` を新規作成し、以下の境界値テストを追加:
   - 存在しない暦日 (`2024-02-31`, `2024-04-31`)
   - 閏年判定 (`2024-02-29` OK, `2023-02-29` NG)
   - 不正 ISO8601 (`"2024-01-15T12:30:45abcZ"`, `"2024-01-15T12:30:45"`)
   - 小数秒付き ISO8601 (`"2024-01-15T12:30:45.123Z"`)
   - 極端な timestamp (`u64::MAX`, 0)
   - 不正 IMF-fixdate (`"Mon, XY Jan 2024 12:30:45 GMT"`)
   - year 境界値 (9999 OK, 10000 NG)
   - `unix_timestamp_from_civil` の `days * 86400` オーバーフロー
   - 閏秒 (`second = 60` でエラー)
10. `pbt/tests/prop_datetime.rs` を新規作成し、ラウンドトリップ PBT を追加:
    - `unix_timestamp_from_civil` → `civil_from_unix_timestamp` → 元に戻る
    - `parse_iso8601` → `format_iso8601` → 元に戻る

# datetime モジュールの日時検証・オーバーフロー不備

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-06-06
- Completed: 2026-06-06
- Branch: feature/fix-datetime-validation-overflow

## 目的

`src/datetime.rs` の各関数と `validate_imf_fixdate` が不正日時を黙って受理したり、極端な入力でオーバーフローする。S3 レスポンス日時の解釈ミスやパニックを防ぐ。

## 優先度根拠

XML レスポンスの `LastModified` / `CreationDate` 等は `parse_iso8601` / `parse_imf_fixdate` 経由で多数の API が利用する。2 月 31 日のような存在しない日付が別日付に正規化されると、条件付きリクエストや表示が誤る。

## 現状

確認済みの問題:

1. **`unix_timestamp_from_civil`** (`src/datetime.rs:84`): `day <= 31` のみ検証。`2024-02-31` が `2024-03-02` に正規化される
2. **`parse_iso8601`** (`src/datetime.rs:281`): 小数秒部を「スキップ」するだけで文字列検証なし。`"2024-01-15T12:30:45abcZ"` のような Z 直前のゴミを受理する。小数点の後に数字がない場合 (`"2024-01-15T12:30:45.Z"`) も受理してしまう
3. **`civil_from_unix_timestamp`** (`src/datetime.rs:56`): `y as i32` で暗黙トランケーション。約 5.85 × 10^11 秒以上の入力でオーバーフロー
4. **`unix_timestamp_from_civil`** (`src/datetime.rs:117`): `(days as u64) * 86400` がオーバーフローしうる (days ≧ 約 2.13 × 10^14)
5. **閏秒**: `src/datetime.rs:93` で `second > 60` のみエラー (60 を許容)。`line 120` で `second.min(59)` により 60 が 59 にサイレント切り捨て
6. **`validate_imf_fixdate`** (`src/types.rs:16-60`): 長さ・曜日名・月名・GMT 末尾のみ検証。数字部分 (`s[5..7]`, `s[12..16]`, `s[17..19]`, `s[20..22]`, `s[23..25]`) と区切り文字 (`s[7]`, `s[11]`, `s[16]`, `s[19]`, `s[22]`) の検証がない

## 設計方針

### 存在しない暦日の検証

`unix_timestamp_from_civil` で civil → unix 変換後に逆変換 (`civil_from_unix_timestamp`) でラウンドトリップ検証し、入力と一致しなければ `Error::InvalidInput` を返す。これにより `2024-02-31` のような存在しない日付を検出する。

### オーバーフロー対策

算術は `checked_mul` / `checked_add` を使用し、オーバーフロー時に `Error::InvalidInput` を返す。実用上限として `year <= 9999` を設ける。`civil_from_unix_timestamp` の戻り値を `Result<CivilDateTime, Error>` に変更し、`y as i32` のトランケーションを防ぐ（`i32::try_from(y)` で検出）。

### 閏秒の扱い

`second > 59` をエラーに変更する。これにより `second == 60` も拒否され、`second.min(59)` を削除できる。閏秒非対応であることを doc comment で明記する。後方互換のない変更であるため、CHANGES.md に `[CHANGE]` として記載する。

### ISO8601 の厳密検証

`parse_iso8601` は `s[19]` 以降の文字列を厳密検証する。末尾が `Z` であることが確認済みであることを前提に、以下を検証する:
- `s[19]` が `.` の場合は小数秒形式 (`.{digits}Z`)。小数点の直後に数字があることを確認し、数字部分をスキップ
- `s[19]` が `Z` の場合はそのまま受理
- それ以外の文字はエラー

### IMF-fixdate の厳密検証

`validate_imf_fixdate` で数値部分と区切り文字の形式を追加検証する。検証項目:

- `s[5..7]` (日) が ASCII 数字
- `s[7]` がスペース
- `s[8..11]` (月名) が有効な月名 (既存)
- `s[11]` がスペース
- `s[12..16]` (年) が ASCII 数字
- `s[16]` がスペース
- `s[17..19]` (時) が ASCII 数字
- `s[19]` が `:`
- `s[20..22]` (分) が ASCII 数字
- `s[22]` が `:`
- `s[23..25]` (秒) が ASCII 数字

### エラー種別

datetime モジュールの関数は純粋なパーサーであるため `Error::InvalidInput` を返す。S3 レスポンス用の呼び出し側 (API パーサー) で必要に応じて `map_err` して `Error::InvalidResponse` に変換する。この変換は各 API の実装側の責務であり、本 issue の対象外とする (issue 0080 で対応予定か、実装時に各 API で自然に処理される)。

### 影響範囲

`civil_from_unix_timestamp` の戻り値を `Result` に変更する影響:
- `src/datetime.rs` 内: `format_imf_fixdate` (L175), `UtcDateTime::from_unix_timestamp` (L143) — `?` で伝播、`from_unix_timestamp` のシグネチャも `Result` に変更
- `src/datetime.rs` 内: `UtcDateTime::from_system_time` (L139) — 呼び出し規約変更
- `src/api/mod.rs` 内: `UtcDateTime::from_system_time` の呼び出し元 (`build_signed_request_inner`, `build_presigned_url`) — `?` で伝播

## AWS S3 API Reference

- GetObject (Last-Modified ヘッダー): <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>
  - > Returns the date and time that the object was last modified.
- HeadObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html>
- ListObjectsV2: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html>
- RFC 9110 Section 5.6.7 (IMF-fixdate): <https://www.rfc-editor.org/rfc/rfc9110#section-5.6.7>
- ISO 8601: <https://www.iso.org/standard/70907.html>

## 完了条件

- 存在しない暦日が `Error::InvalidInput` になる
- 不正 ISO8601 (Z 前にゴミ文字、小数点のみ) が `Error::InvalidInput` になる
- 極端な timestamp でオーバーフローせず `Error::InvalidInput` が返る
- `validate_imf_fixdate` が数字・区切り文字の位置を検証する
- `second == 60` がエラーになる
- 正常系の既知日時テストが通る
- `tests/test_datetime.rs` に単体テストを新規作成し、以下の境界値を検証する:
  - 存在しない暦日 (`2024-02-31`, `2024-04-31`)
  - 閏年判定 (`2024-02-29` OK, `2023-02-29` NG)
  - 不正 ISO8601 (`"2024-01-15T12:30:45abcZ"`, `"2024-01-15T12:30:45.Z"`)
  - 小数秒付き ISO8601 (`"2024-01-15T12:30:45.123Z"`)
  - 極端な timestamp (`u64::MAX`, 0)
  - 不正 IMF-fixdate (`"Mon, XY Jan 2024 12:30:45 GMT"` — 数字部が非 ASCII)
  - year 境界値 (9999 OK, 10000 NG)
  - 閏秒 (`second = 60` でエラー)
- `pbt/tests/prop_datetime.rs` に PBT テストを新規作成し、ラウンドトリップを検証する:
  - `unix_timestamp_from_civil` → `civil_from_unix_timestamp` → 元に戻る
  - `parse_iso8601` → `format_iso8601` → 元に戻る (`format_iso8601` (extended 形式: `YYYY-MM-DDTHH:MM:SSZ`) を新規実装)

## 解決方法

1. `unix_timestamp_from_civil`: year 上限 9999 追加、second > 59 エラー化、checked_mul/checked_add でオーバーフロー対策、ラウンドトリップ検証で存在しない暦日を検出
2. `civil_from_unix_timestamp`: 戻り値を Result に変更、`i32::try_from(y)` でオーバーフロー検出
3. `parse_iso8601`: s[19] 以降の小数秒形式を厳密検証（`.` + 数字 + `Z`）
4. `validate_imf_fixdate`: 数字部分と区切り文字の形式検証を追加
5. `UtcDateTime::from_unix_timestamp`: 戻り値を Result に変更
6. 呼び出し側の修正: `delete_objects.rs`、`signing.rs` のテスト
7. datetime モジュールに存在しない暦日、うるう年、閏秒、year 境界値の単体テストを追加

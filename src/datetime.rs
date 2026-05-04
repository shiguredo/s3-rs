//! 日時ユーティリティモジュール
//!
//! `std::time::SystemTime` と日時文字列 (IMF-fixdate, ISO 8601) の相互変換を提供する。
//! 内部は副作用を持たない純粋関数で構成される (Sans I/O 原則)。
//!
//! - 署名計算で使う `UtcDateTime` (ISO 8601 basic / date stamp 出力)
//! - HTTP ヘッダー入出力で使う `format_imf_fixdate` / `parse_imf_fixdate`
//! - XML レスポンスの ISO 8601 拡張形式パース用 `parse_iso8601`

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::Error;

/// 曜日名 (IMF-fixdate 用)
pub(crate) const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// 月名 (IMF-fixdate 用)
pub(crate) const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// UNIX タイムスタンプから分解した UTC 日時の各フィールド
pub(crate) struct CivilDateTime {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) minute: u32,
    pub(crate) second: u32,
    /// UNIX epoch (1970-01-01) からの経過日数
    pub(crate) days_since_epoch: i64,
}

/// UNIX タイムスタンプから UTC 日時の各フィールドを計算する
///
/// Howard Hinnant の civil_from_days アルゴリズムを使用する。
pub(crate) fn civil_from_unix_timestamp(secs: u64) -> CivilDateTime {
    let days = (secs / 86400) as i64;
    let time_of_day = (secs % 86400) as u32;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    CivilDateTime {
        year: y as i32,
        month: m,
        day: d,
        hour,
        minute,
        second,
        days_since_epoch: days,
    }
}

/// UTC 日時 (年, 月, 日, 時, 分, 秒) から UNIX タイムスタンプ (秒) を計算する
///
/// Howard Hinnant の days_from_civil アルゴリズムを使用する。
/// UNIX epoch より前の日付や 1970 年より前の年は `Err(Error::InvalidInput)` を返す。
pub(crate) fn unix_timestamp_from_civil(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<u64, Error> {
    if year < 1970 {
        return Err(Error::InvalidInput(format!("year must be >= 1970: {year}")));
    }
    if !(1..=12).contains(&month) {
        return Err(Error::InvalidInput(format!("invalid month: {month}")));
    }
    if !(1..=31).contains(&day) {
        return Err(Error::InvalidInput(format!("invalid day: {day}")));
    }
    if hour > 23 {
        return Err(Error::InvalidInput(format!("invalid hour: {hour}")));
    }
    if minute > 59 {
        return Err(Error::InvalidInput(format!("invalid minute: {minute}")));
    }
    if second > 60 {
        return Err(Error::InvalidInput(format!("invalid second: {second}")));
    }

    // Howard Hinnant の days_from_civil
    let y = if month <= 2 {
        year as i64 - 1
    } else {
        year as i64
    };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let d = day as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe as i64 - 719468;

    if days < 0 {
        return Err(Error::InvalidInput(format!(
            "date is before UNIX epoch: {year:04}-{month:02}-{day:02}"
        )));
    }

    let secs = (days as u64) * 86400
        + (hour as u64) * 3600
        + (minute as u64) * 60
        + (second.min(59) as u64);
    Ok(secs)
}

/// UTC 日時 (署名計算用)
pub(crate) struct UtcDateTime {
    civil: CivilDateTime,
}

impl UtcDateTime {
    /// `SystemTime` から UTC 日時を生成する
    ///
    /// `SystemTime::duration_since(UNIX_EPOCH)` が `Err` (UNIX epoch 前) の場合は
    /// `Error::InvalidInput` を返す。
    pub(crate) fn from_system_time(now: SystemTime) -> Result<Self, Error> {
        let secs = now
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::InvalidInput("now is before UNIX epoch".to_string()))?
            .as_secs();
        Ok(Self::from_unix_timestamp(secs))
    }

    /// UNIX タイムスタンプから UTC 日時を生成する (テスト用)
    pub(crate) fn from_unix_timestamp(secs: u64) -> Self {
        Self {
            civil: civil_from_unix_timestamp(secs),
        }
    }

    /// ISO 8601 形式 (YYYYMMDDTHHMMSSZ) で返す
    pub(crate) fn iso8601(&self) -> String {
        let c = &self.civil;
        format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            c.year, c.month, c.day, c.hour, c.minute, c.second
        )
    }

    /// 日付スタンプ (YYYYMMDD) で返す
    pub(crate) fn date_stamp(&self) -> String {
        let c = &self.civil;
        format!("{:04}{:02}{:02}", c.year, c.month, c.day)
    }
}

/// `SystemTime` を IMF-fixdate 文字列にフォーマットする
///
/// 例: `Thu, 01 Jan 1970 00:00:00 GMT`
///
/// `SystemTime` が UNIX epoch 前の場合は `Error::InvalidInput` を返す。
pub(crate) fn format_imf_fixdate(t: SystemTime) -> Result<String, Error> {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::InvalidInput("date is before UNIX epoch".to_string()))?
        .as_secs();
    let c = civil_from_unix_timestamp(secs);
    let weekday = ((c.days_since_epoch.rem_euclid(7) + 4) % 7) as usize;
    Ok(format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        WEEKDAY_NAMES[weekday],
        c.day,
        MONTH_NAMES[(c.month - 1) as usize],
        c.year,
        c.hour,
        c.minute,
        c.second
    ))
}

/// IMF-fixdate 形式の文字列を `SystemTime` にパースする
///
/// 例: `Thu, 01 Jan 1970 00:00:00 GMT`
///
/// 形式不正、または UNIX epoch 前の日付は `Error::InvalidInput` を返す。
pub(crate) fn parse_imf_fixdate(s: &str) -> Result<SystemTime, Error> {
    crate::types::validate_imf_fixdate(s)?;

    // バリデーション済み: ASCII 29 バイト, "Day, DD Mon YYYY HH:MM:SS GMT"
    let day: u32 = s[5..7]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid day in IMF-fixdate: {}", &s[5..7])))?;

    let month_name = &s[8..11];
    let month = (MONTH_NAMES
        .iter()
        .position(|&m| m == month_name)
        .ok_or_else(|| Error::InvalidInput(format!("invalid month: {month_name}")))?
        + 1) as u32;

    let year: i32 = s[12..16]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid year: {}", &s[12..16])))?;
    let hour: u32 = s[17..19]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid hour: {}", &s[17..19])))?;
    let minute: u32 = s[20..22]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid minute: {}", &s[20..22])))?;
    let second: u32 = s[23..25]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid second: {}", &s[23..25])))?;

    let secs = unix_timestamp_from_civil(year, month, day, hour, minute, second)?;
    Ok(UNIX_EPOCH + Duration::from_secs(secs))
}

/// ISO 8601 拡張形式の日時文字列を `SystemTime` にパースする
///
/// 受け付ける形式 (S3 が返す代表的な 2 形式に対応):
/// - `YYYY-MM-DDTHH:MM:SS.fffZ` (XML レスポンスの `<LastModified>` 等)
/// - `YYYY-MM-DDTHH:MM:SSZ`
///
/// 小数秒は秒精度に切り捨てる (`SystemTime` は本 crate 内ではミリ秒以下を扱わない方針)。
/// 形式不正、または UNIX epoch 前の日付は `Error::InvalidInput` を返す。
pub(crate) fn parse_iso8601(s: &str) -> Result<SystemTime, Error> {
    if !s.is_ascii() {
        return Err(Error::InvalidInput(
            "ISO 8601 must be ASCII only".to_string(),
        ));
    }
    if s.len() < 20 {
        return Err(Error::InvalidInput(format!(
            "ISO 8601 too short: {s} ({} bytes)",
            s.len()
        )));
    }
    if !s.ends_with('Z') {
        return Err(Error::InvalidInput(format!(
            "ISO 8601 must end with 'Z': {s}"
        )));
    }
    if s.as_bytes()[4] != b'-' || s.as_bytes()[7] != b'-' || s.as_bytes()[10] != b'T' {
        return Err(Error::InvalidInput(format!(
            "ISO 8601 separator mismatch: {s}"
        )));
    }
    if s.as_bytes()[13] != b':' || s.as_bytes()[16] != b':' {
        return Err(Error::InvalidInput(format!(
            "ISO 8601 time separator mismatch: {s}"
        )));
    }

    let year: i32 = s[0..4]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid year: {}", &s[0..4])))?;
    let month: u32 = s[5..7]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid month: {}", &s[5..7])))?;
    let day: u32 = s[8..10]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid day: {}", &s[8..10])))?;
    let hour: u32 = s[11..13]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid hour: {}", &s[11..13])))?;
    let minute: u32 = s[14..16]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid minute: {}", &s[14..16])))?;
    let second: u32 = s[17..19]
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid second: {}", &s[17..19])))?;

    // 残り (`Z` を除いた小数秒部) は秒精度に切り捨てるためスキップする
    let secs = unix_timestamp_from_civil(year, month, day, hour, minute, second)?;
    Ok(UNIX_EPOCH + Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_from_unix_timestamp_epoch() {
        let c = civil_from_unix_timestamp(0);
        assert_eq!(c.year, 1970);
        assert_eq!(c.month, 1);
        assert_eq!(c.day, 1);
        assert_eq!(c.hour, 0);
        assert_eq!(c.minute, 0);
        assert_eq!(c.second, 0);
        assert_eq!(c.days_since_epoch, 0);

        let dt = UtcDateTime::from_unix_timestamp(0);
        assert_eq!(dt.iso8601(), "19700101T000000Z");
        assert_eq!(dt.date_stamp(), "19700101");
    }

    #[test]
    fn test_civil_from_unix_timestamp_known_date() {
        // 2024-01-15 12:30:45 UTC
        let c = civil_from_unix_timestamp(1705321845);
        assert_eq!(c.year, 2024);
        assert_eq!(c.month, 1);
        assert_eq!(c.day, 15);
        assert_eq!(c.hour, 12);
        assert_eq!(c.minute, 30);
        assert_eq!(c.second, 45);

        let dt = UtcDateTime::from_unix_timestamp(1705321845);
        assert_eq!(dt.iso8601(), "20240115T123045Z");
    }

    #[test]
    fn test_unix_timestamp_from_civil_round_trip() {
        // いくつかの代表時刻でラウンドトリップを確認する
        for &secs in &[0u64, 1, 86399, 86400, 1_705_321_845, 4_102_444_800] {
            let c = civil_from_unix_timestamp(secs);
            let back =
                unix_timestamp_from_civil(c.year, c.month, c.day, c.hour, c.minute, c.second)
                    .unwrap();
            assert_eq!(secs, back, "round-trip failed for {secs}");
        }
    }

    #[test]
    fn test_format_imf_fixdate_epoch() {
        let result = format_imf_fixdate(UNIX_EPOCH).unwrap();
        assert_eq!(result, "Thu, 01 Jan 1970 00:00:00 GMT");
    }

    #[test]
    fn test_format_imf_fixdate_known_date() {
        // 2024-01-15 12:30:45 UTC は月曜日
        let t = UNIX_EPOCH + Duration::from_secs(1_705_321_845);
        let result = format_imf_fixdate(t).unwrap();
        assert_eq!(result, "Mon, 15 Jan 2024 12:30:45 GMT");
    }

    #[test]
    fn test_parse_imf_fixdate_epoch() {
        let t = parse_imf_fixdate("Thu, 01 Jan 1970 00:00:00 GMT").unwrap();
        assert_eq!(t, UNIX_EPOCH);
    }

    #[test]
    fn test_parse_imf_fixdate_known_date() {
        let t = parse_imf_fixdate("Mon, 15 Jan 2024 12:30:45 GMT").unwrap();
        assert_eq!(t, UNIX_EPOCH + Duration::from_secs(1_705_321_845));
    }

    #[test]
    fn test_parse_imf_fixdate_round_trip() {
        let original = UNIX_EPOCH + Duration::from_secs(1_705_321_845);
        let formatted = format_imf_fixdate(original).unwrap();
        let parsed = parse_imf_fixdate(&formatted).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_parse_imf_fixdate_invalid() {
        assert!(parse_imf_fixdate("invalid").is_err());
        assert!(parse_imf_fixdate("Thu, 32 Jan 1970 00:00:00 GMT").is_err()); // 32 日
    }

    #[test]
    fn test_parse_iso8601_with_milliseconds() {
        let t = parse_iso8601("2024-01-15T12:30:45.123Z").unwrap();
        assert_eq!(t, UNIX_EPOCH + Duration::from_secs(1_705_321_845));
    }

    #[test]
    fn test_parse_iso8601_no_fraction() {
        let t = parse_iso8601("2024-01-15T12:30:45Z").unwrap();
        assert_eq!(t, UNIX_EPOCH + Duration::from_secs(1_705_321_845));
    }

    #[test]
    fn test_parse_iso8601_invalid() {
        assert!(parse_iso8601("invalid").is_err());
        assert!(parse_iso8601("2024-01-15 12:30:45Z").is_err()); // T が空白
        assert!(parse_iso8601("2024-01-15T12:30:45").is_err()); // Z 抜け
    }
}

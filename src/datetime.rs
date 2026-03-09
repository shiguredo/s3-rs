//! 日時ユーティリティモジュール
//!
//! UNIX タイムスタンプから UTC 日時フィールドへの変換を提供する。
//! 署名計算 (`UtcDateTime`) と HTTP 日時 (`HttpDate`) の両方で使用する。

use std::time::{SystemTime, UNIX_EPOCH};

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

/// UTC 日時 (署名計算用)
pub(crate) struct UtcDateTime {
    civil: CivilDateTime,
}

impl UtcDateTime {
    /// 現在の UTC 日時を取得する
    pub(crate) fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_secs();
        Self::from_unix_timestamp(secs)
    }

    /// UNIX タイムスタンプから UTC 日時を生成する
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
}

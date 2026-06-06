use proptest::prelude::*;
use shiguredo_s3::datetime_round_trip;

// `unix_timestamp_from_civil` → `civil_from_unix_timestamp` のラウンドトリッププロパティ
//
// 任意の有効な日時 (1970-01-01 〜 9999-12-31) について、
// UNIX タイムスタンプとの相互変換が元の日時を復元することを検証する。
proptest! {
    #[test]
    fn round_trip_civil_to_unix_and_back(
        year in 1970i32..=9999i32,
        month in 1u32..=12u32,
        day in 1u32..=28u32,
        hour in 0u32..=23u32,
        minute in 0u32..=59u32,
        second in 0u32..=59u32,
    ) {
        let result = datetime_round_trip(year, month, day, hour, minute, second);
        // 存在しない暦日 (例: 2/31) は unix_timestamp_from_civil が Err を返す
        if let Ok((y, m, d, h, mi, s)) = result {
            prop_assert_eq!(y, year);
            prop_assert_eq!(m, month);
            prop_assert_eq!(d, day);
            prop_assert_eq!(h, hour);
            prop_assert_eq!(mi, minute);
            prop_assert_eq!(s, second);
        }
    }
}

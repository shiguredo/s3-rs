use shiguredo_s3::datetime_round_trip;

// `unix_timestamp_from_civil` → `civil_from_unix_timestamp` のラウンドトリッププロパティ
//
// 任意の有効な日時 (1970-01-01 〜 9999-12-31) について、
// UNIX タイムスタンプとの相互変換が元の日時を復元することを検証する。
#[test]
fn round_trip_civil_to_unix_and_back() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time("S3_PBT_SEED")?;
    let round_trip_count = std::cell::Cell::new(0usize);
    let mut runner = noprop::Runner::new(seed);

    runner.run(256, |ctx| {
        // 境界値 (範囲の端・月の日数の端・うるう年の 2/29 相当) に 1/5 の確率を与える
        let year = noprop::sample_with_boundaries(
            ctx,
            &[1970i32, 9999i32],
            noprop::Ratio::one_nth(5),
            |ctx| noprop::sample_usize_in(ctx, 1970..=9999) as i32,
        );
        let month = noprop::sample_with_boundaries(
            ctx,
            &[1u32, 2u32, 12u32],
            noprop::Ratio::one_nth(5),
            |ctx| noprop::sample_usize_in(ctx, 1..=12) as u32,
        );
        // 日 29/30/31 は月によって不正な暦日 (例: 2/31) になりうるため Err を返す
        let day = noprop::sample_with_boundaries(
            ctx,
            &[1u32, 28u32, 29u32, 30u32, 31u32],
            noprop::Ratio::one_nth(5),
            |ctx| noprop::sample_usize_in(ctx, 1..=31) as u32,
        );
        let hour =
            noprop::sample_with_boundaries(ctx, &[0u32, 23u32], noprop::Ratio::one_nth(5), |ctx| {
                noprop::sample_usize_in(ctx, 0..=23) as u32
            });
        let minute =
            noprop::sample_with_boundaries(ctx, &[0u32, 59u32], noprop::Ratio::one_nth(5), |ctx| {
                noprop::sample_usize_in(ctx, 0..=59) as u32
            });
        let second =
            noprop::sample_with_boundaries(ctx, &[0u32, 59u32], noprop::Ratio::one_nth(5), |ctx| {
                noprop::sample_usize_in(ctx, 0..=59) as u32
            });

        let result = datetime_round_trip(year, month, day, hour, minute, second);
        // 存在しない暦日 (例: 2/31) は Err を返すので、成功時のみ復元値を検証する
        if let Ok((y, m, d, h, mi, s)) = result {
            assert_eq!(y, year, "年が復元されない: {year}");
            assert_eq!(m, month, "月が復元されない: {month}");
            assert_eq!(d, day, "日が復元されない: {day}");
            assert_eq!(h, hour, "時が復元されない: {hour}");
            assert_eq!(mi, minute, "分が復元されない: {minute}");
            assert_eq!(s, second, "秒が復元されない: {second}");
            round_trip_count.set(round_trip_count.get() + 1);
        }
        Ok(())
    })?;

    assert!(
        round_trip_count.get() > 0,
        "ラウンドトリップが成功したケースがない\n{runner}"
    );
    Ok(())
}

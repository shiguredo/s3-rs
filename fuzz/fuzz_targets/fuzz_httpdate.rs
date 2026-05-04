#![no_main]

//! IMF-fixdate バリデーションとパースの fuzz ターゲット
//!
//! 任意の文字列を `validate_imf_fixdate` および `parse_imf_fixdate` に渡し、
//! パニックしないことを検証する。
//!
//! HttpDate 構造体は廃止されたため、本ターゲットも関数 API を直接呼び出す形に
//! 書き換えている (issue 0060)。

use libfuzzer_sys::fuzz_target;
use shiguredo_s3::validate_imf_fixdate;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // バリデーション関数がパニックしないことを検証する
        let _ = validate_imf_fixdate(s);
    }
});

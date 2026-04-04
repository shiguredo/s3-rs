#![no_main]

//! HttpDate バリデーションの fuzz ターゲット
//!
//! 任意の文字列を try_from_imf_fixdate に渡し、パニックしないことを検証する。

use libfuzzer_sys::fuzz_target;
use shiguredo_s3::types::HttpDate;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // バリデーション付きコンストラクタがパニックしないことを検証する
        let _ = HttpDate::try_from_imf_fixdate(s);

        // バリデーションなしコンストラクタもパニックしないことを検証する
        let date = HttpDate::from_imf_fixdate(s);
        let _ = date.as_str();
        let _ = format!("{date}");
    }
});

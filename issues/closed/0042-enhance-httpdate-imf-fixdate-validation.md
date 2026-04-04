# HttpDate::from_imf_fixdate() のフォーマット検証追加

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

`src/types.rs` の `HttpDate::from_imf_fixdate()` は IMF-fixdate 形式の文字列をバリデーションなしでそのまま保持している。
不正なフォーマットの文字列を受け入れてしまう。

## 優先度

低

## 解決方法

既存の `from_imf_fixdate()` は後方互換のため残し、バリデーション付きの `try_from_imf_fixdate()` を追加した。検証内容:

- 長さチェック (29 文字)
- 曜日の妥当性チェック (Sun/Mon/Tue/Wed/Thu/Fri/Sat)
- 月の妥当性チェック (Jan/Feb/Mar/.../Dec)
- 区切り文字チェック (", " と末尾 " GMT")

// -------------------------------------------------------
// ユーティリティ
// -------------------------------------------------------

use shiguredo_s3::{Client, Config, Credentials};

/// `SystemTime::now()` を呼び出すヘルパ
///
/// shiguredo_s3 は Sans I/O のため `build_request` / `presigned` に
/// 現在時刻を引数で渡す。サンプル側で副作用を 1 箇所に閉じ込める目的。
pub(crate) fn now() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// ファイル拡張子から MIME タイプを推測する
pub(crate) fn guess_mime_type(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?.to_ascii_lowercase();
    let ext = ext.as_str();
    Some(match ext {
        // テキスト
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "svg" => "image/svg+xml",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        "md" => "text/markdown",
        "wasm" => "application/wasm",
        // 画像
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        // 動画
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        // 音声
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        // アーカイブ
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "br" => "application/x-brotli",
        "zst" | "zstd" => "application/zstd",
        // フォント
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        // ドキュメント
        "pdf" => "application/pdf",
        _ => return None,
    })
}

/// アップロード時の Content-Type を決定する
///
/// 明示指定 > MIME 推測 > None の優先順で決定する
pub(crate) fn resolve_content_type<'a>(
    explicit: Option<&'a str>,
    path: &str,
    no_guess: bool,
) -> Option<&'a str> {
    if explicit.is_some() {
        return explicit;
    }
    if no_guess {
        return None;
    }
    // guess_mime_type は 'static を返すので 'a にキャストして安全
    guess_mime_type(path)
}

/// fnmatch 互換のグロブパターンマッチ
///
/// `*` は `/` を含む任意の文字列にマッチする (aws s3 の挙動と同じ)
pub(crate) fn fnmatch(pattern: &str, text: &str) -> bool {
    fnmatch_inner(pattern.as_bytes(), text.as_bytes())
}

fn fnmatch_inner(pattern: &[u8], text: &[u8]) -> bool {
    let (mut pi, mut ti) = (0, 0);
    let (mut star_pi, mut star_ti) = (usize::MAX, 0);

    while ti < text.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == text[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star_pi = pi;
            star_ti = ti;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }
    pi == pattern.len()
}

/// フィルタルール (--exclude / --include)
#[derive(Clone)]
pub(crate) enum FilterRule {
    Exclude(String),
    Include(String),
}

/// フィルタルールに基づいてパスがマッチするか判定する
///
/// デフォルトではすべてのファイルが対象。ルールは指定順に評価され、最後にマッチしたルールが優先される。
pub(crate) fn should_include(path: &str, filters: &[FilterRule]) -> bool {
    let mut included = true;
    for rule in filters {
        match rule {
            FilterRule::Exclude(pattern) => {
                if fnmatch(pattern, path) {
                    included = false;
                }
            }
            FilterRule::Include(pattern) => {
                if fnmatch(pattern, path) {
                    included = true;
                }
            }
        }
    }
    included
}

/// --exclude / --include オプションを複数回指定順でパースする
pub(crate) fn parse_filters(args: &mut noargs::RawArgs) -> Vec<FilterRule> {
    let mut filters = Vec::new();
    loop {
        // exclude と include を交互に試し、どちらかがあれば追加する
        let exclude: Option<String> = noargs::opt("exclude")
            .doc("Exclude pattern (fnmatch)")
            .ty("PATTERN")
            .take(args)
            .present_and_then(|o| o.value().parse())
            .unwrap_or(None);
        if let Some(pattern) = exclude {
            filters.push(FilterRule::Exclude(pattern));
            continue;
        }
        let include: Option<String> = noargs::opt("include")
            .doc("Include pattern (fnmatch)")
            .ty("PATTERN")
            .take(args)
            .present_and_then(|o| o.value().parse())
            .unwrap_or(None);
        if let Some(pattern) = include {
            filters.push(FilterRule::Include(pattern));
            continue;
        }
        break;
    }
    filters
}

/// S3 URI (s3://bucket/key) をパースする
pub(crate) fn parse_s3_uri(uri: &str) -> Option<(String, String)> {
    let path = uri.strip_prefix("s3://")?;
    let (bucket, key) = match path.find('/') {
        Some(pos) => (path[..pos].to_string(), path[pos + 1..].to_string()),
        None => (path.to_string(), String::new()),
    };
    if bucket.is_empty() {
        return None;
    }
    Some((bucket, key))
}

/// S3 URI かどうかを判定する
pub(crate) fn is_s3_uri(path: &str) -> bool {
    path.starts_with("s3://")
}

/// 環境変数から Client を構築する
pub(crate) fn build_client(args: &noargs::RawArgs) -> noargs::Result<Client> {
    let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
        .map_err(|_| noargs::Error::other(args, "AWS_ACCESS_KEY_ID is not set"))?;
    let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
        .map_err(|_| noargs::Error::other(args, "AWS_SECRET_ACCESS_KEY is not set"))?;
    let region =
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "ap-northeast-1".to_string());
    let endpoint = std::env::var("AWS_ENDPOINT_URL_S3").ok();
    let use_path_style = std::env::var("S3CLI_PATH_STYLE")
        .map(|v| v == "1")
        .unwrap_or(false);
    let ignore_cert_check = std::env::var("S3CLI_IGNORE_CERT_CHECK")
        .map(|v| v == "1")
        .unwrap_or(false);

    let mut builder = Config::builder()
        .region(region)
        .credentials_provider(Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "s3cli",
        ))
        .force_path_style(use_path_style)
        .ignore_cert_check(ignore_cert_check);
    if let Some(ep) = endpoint {
        builder = builder.endpoint(ep);
    }
    let config = builder.build().map_err(|e| format!("{e}"))?;

    Ok(Client::from_conf(config))
}

/// 人間が読みやすいサイズ表示に変換する
pub(crate) fn human_readable_size(size: i64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = size as f64;
    for unit in UNITS {
        if value < 1024.0 {
            if *unit == "B" {
                return format!("{value:.0} {unit}");
            }
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{value:.1} PiB")
}

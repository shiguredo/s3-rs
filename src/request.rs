// -------------------------------------------------------
// ワイヤー型 (Sans I/O)
// -------------------------------------------------------

use std::collections::HashMap;

/// Presigned リクエスト
///
/// URL だけでなく、リクエストに必要なボディも保持する。
/// GET / HEAD / DELETE など body が不要な場合は `body` は空。
/// CompleteMultipartUpload のように POST body が必要な場合は XML 等が入る。
#[derive(Debug, Clone)]
pub struct PresignedRequest {
    /// Presigned URL
    pub url: String,
    /// HTTP メソッド
    pub method: String,
    /// リクエスト時に付与が必要な header (署名対象に含まれる)
    pub headers: Vec<(String, String)>,
    /// リクエストボディ (不要な場合は空)
    pub body: Vec<u8>,
}

/// 署名済み S3 リクエスト
///
/// `build_request()` で構築する。
/// 利用者は各フィールドを使って任意の HTTP クライアントでリクエストを送信する。
#[derive(Debug, Clone)]
pub struct S3Request {
    /// HTTP メソッド (GET, PUT, DELETE, POST, HEAD)
    pub method: String,
    /// リクエスト URI (パス + クエリ文字列)
    pub uri: String,
    /// HTTP リクエストヘッダー (名前, 値) のリスト (署名済み)
    pub headers: Vec<(String, String)>,
    /// リクエストボディ
    pub body: Vec<u8>,
    /// 接続先ホスト名
    pub host: String,
    /// 接続先ポート番号
    pub port: u16,
    /// HTTPS を使用するかどうか
    pub https: bool,
    /// TLS 証明書の検証を無視する (テスト環境向け)
    pub ignore_cert_check: bool,
    /// レスポンスにボディがないことを期待するか (HEAD リクエスト)
    pub expect_no_body: bool,
}

/// S3 レスポンス
///
/// HTTP レスポンスのステータスコード、ヘッダー、ボディを保持する。
/// 利用者が任意の HTTP クライアントから構築して `parse_response()` に渡す。
#[derive(Debug, Clone)]
pub struct S3Response {
    /// HTTP ステータスコード
    pub status_code: u16,
    /// HTTP レスポンスヘッダー (名前, 値) のリスト
    pub headers: Vec<(String, String)>,
    /// レスポンスボディ
    pub body: Vec<u8>,
}

impl S3Response {
    /// レスポンスが成功 (2xx) かどうかを返す
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// 指定した名前のヘッダー値を返す (大文字小文字を区別しない)
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Content-Length ヘッダーの値を返す
    pub fn content_length(&self) -> Option<u64> {
        self.get_header("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// x-amz-meta-* ヘッダーからカスタムメタデータを抽出する
    ///
    /// メタデータが存在しない場合は None を返す。
    /// aws-sdk-rust と同様に、キーの大文字小文字は元のヘッダー名を保持する。
    pub fn extract_metadata(&self) -> Option<HashMap<String, String>> {
        let prefix = "x-amz-meta-";
        let prefix_len = prefix.len();
        let map: HashMap<String, String> = self
            .headers
            .iter()
            .filter_map(|(k, v)| {
                k.to_ascii_lowercase()
                    .strip_prefix(prefix)
                    .map(|_| (k[prefix_len..].to_string(), v.clone()))
            })
            .collect();
        if map.is_empty() { None } else { Some(map) }
    }
}

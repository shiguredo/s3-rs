use std::time::SystemTime;

/// AWS クレデンシャル
///
/// `Debug` 実装ではシークレットアクセスキーとセッショントークンをマスクする。
/// aws-credential-types の `Credentials` と同等の API を提供する。
#[derive(Clone)]
pub struct Credentials {
    /// アクセスキー ID
    pub(crate) access_key_id: String,
    /// シークレットアクセスキー
    pub(crate) secret_access_key: String,
    /// セッショントークン (一時クレデンシャル用)
    pub(crate) session_token: Option<String>,
    /// クレデンシャルの有効期限 (情報目的、署名計算では未使用)
    pub(crate) expires_after: Option<SystemTime>,
    /// クレデンシャルの取得元プロバイダー名 (情報目的、署名計算では未使用)
    pub(crate) provider_name: &'static str,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"**REDACTED**")
            .field(
                "session_token",
                &self.session_token.as_ref().map(|_| "**REDACTED**"),
            )
            .field("expires_after", &self.expires_after)
            .field("provider_name", &self.provider_name)
            .finish()
    }
}

impl Credentials {
    /// クレデンシャルを作成する
    ///
    /// `aws-credential-types::Credentials::new` と同じシグネチャ。
    /// 一時クレデンシャル (STS / IAM ロール) を扱うときは `session_token` と
    /// `expires_after` を `Some` で指定する。`provider_name` はデバッグ用途で
    /// 利用される静的文字列。
    pub fn new(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: Option<String>,
        expires_after: Option<SystemTime>,
        provider_name: &'static str,
    ) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            session_token,
            expires_after,
            provider_name,
        }
    }

    /// アクセスキー ID を返す
    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    /// シークレットアクセスキーを返す
    pub fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    /// セッショントークンを返す
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// 有効期限を返す
    pub fn expires_after(&self) -> Option<SystemTime> {
        self.expires_after
    }

    /// プロバイダー名を返す
    pub fn provider_name(&self) -> &'static str {
        self.provider_name
    }
}

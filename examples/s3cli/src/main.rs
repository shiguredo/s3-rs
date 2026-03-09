/// aws s3 互換の S3 CLI ツール
///
/// 使い方:
///   s3cli cp <src> <dst> [--content-type TYPE] [--recursive]
///   s3cli mv <src> <dst> [--recursive]
///   s3cli sync <src> <dst> [--delete] [--size-only] [--exact-timestamps]
///   s3cli ls [s3://bucket[/prefix]] [--recursive] [--human-readable] [--summarize]
///   s3cli ls                         (バケット一覧)
///   s3cli rm <s3://bucket/key> [--recursive]
///   s3cli mb s3://bucket-name
///   s3cli rb s3://bucket-name [--force]
///   s3cli presign <s3://bucket/key> [--expires-in SECONDS]
///
/// 環境変数:
///   AWS_ACCESS_KEY_ID       - アクセスキー ID
///   AWS_SECRET_ACCESS_KEY   - シークレットアクセスキー
///   AWS_DEFAULT_REGION      - リージョン (デフォルト: ap-northeast-1)
///   AWS_ENDPOINT_URL_S3     - カスタムエンドポイント
///   S3CLI_PATH_STYLE        - パススタイルアクセス (1 で有効)
///   S3CLI_IGNORE_CERT_CHECK - TLS 証明書の検証を無視する (1 で有効)
mod commands;
mod ops;
mod params;
mod transport;
mod upload;
mod util;

use std::process::ExitCode;
use std::sync::Arc;

// -------------------------------------------------------
// エントリポイント
// -------------------------------------------------------

fn run() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "s3cli";
    args.metadata_mut().app_description = "aws s3 compatible CLI tool";

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("s3cli {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    noargs::HELP_FLAG.take_help(&mut args);

    let ignore_cert_check = std::env::var("S3CLI_IGNORE_CERT_CHECK")
        .map(|v| v == "1")
        .unwrap_or(false);
    let tls_config = Arc::new(transport::build_tls_config(ignore_cert_check));

    let rt = tokio::runtime::Runtime::new().unwrap();

    if noargs::cmd("cp")
        .doc("Copy files to/from S3")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_cp(args, tls_config))
    } else if noargs::cmd("mv")
        .doc("Move files to/from S3")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_mv(args, tls_config))
    } else if noargs::cmd("ls")
        .doc("List S3 objects")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_ls(args, tls_config))
    } else if noargs::cmd("rm")
        .doc("Delete S3 objects")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_rm(args, tls_config))
    } else if noargs::cmd("sync")
        .doc("Sync files between local and S3")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_sync(args, tls_config))
    } else if noargs::cmd("mb")
        .doc("Create an S3 bucket")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_mb(args, tls_config))
    } else if noargs::cmd("rb")
        .doc("Remove an S3 bucket")
        .take(&mut args)
        .is_present()
    {
        rt.block_on(commands::cmd_rb(args, tls_config))
    } else if noargs::cmd("presign")
        .doc("Generate a presigned URL")
        .take(&mut args)
        .is_present()
    {
        commands::cmd_presign(args)
    } else if let Some(help) = args.finish()? {
        print!("{help}");
        Ok(())
    } else {
        Ok(())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#?}");
            ExitCode::FAILURE
        }
    }
}

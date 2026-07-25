use anyhow::bail;
use clap::Parser;
use obscuravpn_api::types::AccountId;
use obscuravpn_api::{Client, ClientError};

const API_URL: &str = "https://v1.api.prod.obscura.net/api";
const ALTERNATIVE_HOST: &str = "crimsonlance.net";

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[clap(long, default_value = "123")]
    account_no: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let account_id = AccountId::from_string_unchecked(args.account_no);
    let alternative_hosts = vec![ALTERNATIVE_HOST.to_string()];

    let client = Client::new(
        API_URL,
        alternative_hosts,
        account_id,
        "block test cli client",
        None,
        #[cfg(target_os = "linux")]
        None,
        None,
    )?;
    match client.acquire_auth_token().await {
        Ok(_) => println!("not blocked"),
        Err(error) => match error {
            ClientError::ApiError(error) => {
                println!("api_error: {}", error);
                println!("not blocked");
            }
            ClientError::ProtocolError(error) => {
                println!("protocol_error: {}", error);
                println!("not blocked");
            }
            ClientError::RequestExecError(error) => {
                println!("request execution error: {:?}", error);
                println!("maybe blocked");
            }
            ClientError::InvalidHeaderValue => {
                bail!("invalid request header value");
            }
            ClientError::ResponseTooLarge => {
                bail!("response too large");
            }
            ClientError::ProofOfWorkTimeout => {
                bail!("proof of work timed out");
            }
            ClientError::ProofOfWork(error) => {
                bail!("proof of work failed: {error}");
            }
            ClientError::Other(error) => {
                bail!("other error: {:?}", error);
            }
        },
    };
    Ok(())
}

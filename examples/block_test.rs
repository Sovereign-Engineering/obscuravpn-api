use anyhow::bail;
use clap::Parser;
use obscuravpn_api::types::AccountId;
use obscuravpn_api::{Client, ClientError};

const API_URL: &str = "https://v1.api.prod.obscura.net/api";

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[clap(long, default_value = "123")]
    account_no: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let account_id = AccountId::from_string_unchecked(args.account_no);

    let client = Client::new(API_URL, account_id, "block test cli client")?;
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
            ClientError::Other(error) => {
                bail!("other error: {}", error);
            }
        },
    };
    Ok(())
}

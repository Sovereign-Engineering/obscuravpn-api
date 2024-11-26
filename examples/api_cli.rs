use anyhow::bail;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::{Parser, Subcommand};
use obscuravpn_api::cmd::*;
use obscuravpn_api::types::{TunnelConfig, WgPubkey};
use obscuravpn_api::wg_conf::build_wg_conf;
use obscuravpn_api::Client;
use qrcode::QrCode;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[clap(long, default_value = "https://v1.api.prod.obscura.net/api")]
    base_url: String,
    #[clap(long)]
    account_no: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    ListExits,
    ListTunnels,
    ListRelays,
    CreateObfuscatedTunnel {
        /// use specific relay
        #[clap(long)]
        relay: Option<String>,
        /// use specific exit
        #[clap(long)]
        exit: Option<String>,
    },
    CreateStaticTunnel {
        /// print WireGuard configuration to stdout (JSON to stderr)
        #[clap(long)]
        wg_conf: bool,
        /// use specific relay
        #[clap(long)]
        relay: Option<String>,
        /// use specific exit
        #[clap(long)]
        exit: Option<String>,
    },
    DeleteAllTunnels,
    TopUp {
        #[clap(long)]
        months: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let url = args.base_url;
    let account_id = args.account_no;

    let client = Client::new(url, account_id, "example cli client")?;

    eprintln!("Get account info");
    let account_info = client.run(GetAccountInfo()).await?;
    eprintln!("{:#?}", account_info);

    match args.command {
        Commands::ListRelays => {
            eprintln!("Get all relays");
            let relays = client.run(ListRelays {}).await?;
            println!("{}", serde_json::to_string_pretty(&relays)?);
        }
        Commands::ListExits => {
            eprintln!("Get all exits");
            let exits = client.run(ListExits {}).await?;
            println!("{:#?}", exits);
        }
        Commands::ListTunnels => {
            eprintln!("Get all existing tunnels");
            let tunnels = client.run(ListTunnels {}).await?;
            println!("{:#?}", tunnels);
        }
        Commands::CreateObfuscatedTunnel { relay, exit } => {
            eprintln!("Creating new tunnel");
            let sk = StaticSecret::random_from_rng(OsRng);
            eprintln!("Created private key");
            eprintln!("{}", STANDARD.encode(sk.as_bytes()));
            let pk = PublicKey::from(&sk);
            let wg_pubkey = WgPubkey(pk.to_bytes());
            let tunnel = client
                .run(CreateTunnel::Obfuscated {
                    id: None,
                    wg_pubkey,
                    relay,
                    exit,
                })
                .await?;

            eprintln!("Created tunnel {}", &tunnel.id);
            println!("{}", serde_json::to_string_pretty(&tunnel)?);
        }
        Commands::CreateStaticTunnel { wg_conf, relay, exit } => {
            eprintln!("Creating new tunnel");
            let sk = StaticSecret::random_from_rng(OsRng);
            let pk = PublicKey::from(&sk);
            let sk = STANDARD.encode(sk.as_bytes());
            if !wg_conf {
                eprintln!("Created private key");
                println!("{}", sk);
            }
            let wg_pubkey = WgPubkey(pk.to_bytes());
            let tunnel = client
                .run(CreateTunnel::UdpPort {
                    id: None,
                    wg_pubkey,
                    relay,
                    exit,
                })
                .await?;
            eprintln!("Created tunnel {}", &tunnel.id);
            if wg_conf {
                eprintln!("{}", serde_json::to_string_pretty(&tunnel)?);
                let TunnelConfig::UdpPort { client, server } = tunnel.config else {
                    bail!("unexpected tunnel variant")
                };
                println!("{}", build_wg_conf(Some(tunnel.id), sk, client, server))
            } else {
                println!("{}", serde_json::to_string_pretty(&tunnel)?);
            }
        }
        Commands::DeleteAllTunnels => {
            eprintln!("Get all existing tunnels");
            let tunnels = client.run(ListTunnels {}).await?;
            let tunnel_ids = tunnels.into_iter().map(|t| t.id).collect::<Vec<String>>();
            eprintln!("IDs: {:?}", tunnel_ids);

            eprintln!("Deleting all existing tunnels");
            for id in tunnel_ids {
                eprintln!("Deleting tunnel {}", &id);
                client.run(DeleteTunnel { id }).await?;
            }
        }
        Commands::TopUp { months } => {
            eprintln!("Creating top up invoice");
            let LightningTopUpInfo { invoice } = client.run(CreateLightningTopUp { months }).await?;
            println!("{}", &invoice);
            let qr_code = QrCode::new(invoice)?.render::<qrcode::render::unicode::Dense1x2>().build();
            eprintln!("{}", qr_code)
        }
    };

    Ok(())
}

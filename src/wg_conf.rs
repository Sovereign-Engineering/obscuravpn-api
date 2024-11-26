use crate::types::{WgClientConfig, WgServerConfig};
use itertools::Itertools;

pub fn build_wg_conf(tunnel_id: Option<String>, secret_key_base64: String, client: WgClientConfig, server: WgServerConfig) -> String {
    let mut conf = String::new();

    conf += "[Interface]\n";
    if let Some(tunnel_id) = tunnel_id {
        conf += &format!("# Obscura tunnel ID: {tunnel_id}\n");
    }
    conf += &format!("PrivateKey = {}\n", secret_key_base64);
    conf += &format!("Address = {}\n", client.addresses.iter().format(","));
    conf += &format!("DNS = {}\n", server.dnses.iter().format(","));

    conf += "\n[Peer]\n";
    conf += &format!("PublicKey = {}\n", server.wg_pubkey);
    conf += "AllowedIPs = 0.0.0.0/0,::0/0\n";
    conf += &format!("Endpoint = {}\n", server.endpoints.first().unwrap());

    conf
}

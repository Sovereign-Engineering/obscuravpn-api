use crate::cmd::Cmd;
use crate::types::OneTunnel;
use crate::types::WgPubkey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateTunnel {
    UdpPort {
        id: Option<Uuid>,
        label: Option<String>,
        wg_pubkey: WgPubkey,
        relay: Option<String>,
        exit: Option<String>,
    },
    Obfuscated {
        id: Option<Uuid>,
        label: Option<String>,
        wg_pubkey: WgPubkey,
        relay: Option<String>,
        exit: Option<String>,
    },
}

impl Cmd for CreateTunnel {
    type Output = OneTunnel;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = super::PATH;
}

#[test]
fn test_json() {
    let cmd_json = r#"
    {
        "type": "obfuscated",
        "id": "bd309cd5-e6e7-40b0-82d8-dbacdc827cb6",
        "label": null,
        "wg_pubkey": "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=",
        "relay": "NYC-001",
        "exit": "NYC-001"
    }
    "#;
    let output_json = r#"
    {
      "id": "dc799918-7738-446f-b1fc-ae3ba98103c7",
      "status": {
        "type": "created",
        "when": 1725050273
      },
      "config": {
        "type": "obfuscated",
        "client_pubkey": "wjaiHUEOJ8k3X+U3b6H6yTcipqFipIbFQSB0CwZDNlQ=",
        "client_ips_v4": ["10.150.177.7/32"],
        "client_ips_v6": ["fc00:bbbb:bbbb:bb01:d:0:16:b107/128"],
        "dns": ["10.64.0.1"],
        "gateway_ip_v4": "10.64.0.1",
        "relay_addr_v4": "8.8.31.3:443",
        "relay_addr_v6": "[2001:db8:1234:ffff:ffff:ffff:ffff:ffff]:443",
        "relay_cert": "asdf",
        "exit_pubkey": "4s9JIhxC/D02tosXYYcgrD+pHI+C7oTAFsXzVisKjRs="
      },
      "relay": {
        "id": "NYC-001",
        "ip_v4": "8.8.31.3",
        "ip_v6": "2001:db8:1234:ffff:ffff:ffff:ffff:ffff",
        "preferred_exits": [{ "id": "nyc-wg-30" }],
        "tls_cert": "MIIBWjCCAQGgAwIBAgIVAK3WuHUPFg+mmBGiDhW9VNjDmudKMAoGCCqGSM49BAMCMCExHzAdBgNVBAMMFnJjZ2VuIHNlbGYgc2lnbmVkIGNlcnQwIBcNNzUwMTAxMDAwMDAwWhgPNDA5NjAxMDEwMDAwMDBaMCExHzAdBgNVBAMMFnJjZ2VuIHNlbGYgc2lnbmVkIGNlcnQwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASLPcQhOgBGP8HRlGBy6rTO7SWnGgSuCEW6hi+Q/wnUr8H9FkJC7UGD9738XGKEq35ZKybrekr0DbK8YsVZ8SX+oxQwEjAQBgNVHREECTAHggVyZWxheTAKBggqhkjOPQQDAgNHADBEAiAYnVn/bozbp6u0iShFfUgbdGvAvf/hXwLhgonj+Fh+xAIgUgfQqvWFPbh3BNu98LhNFrESngTHtYYAOWt3wZdQOm4=",
        "ports": [53, 443]
      },
      "exit": {
        "id": "NYC-001",
        "country_code": "US",
        "city_code": "nyc",
        "city_name": "New York",
        "provider_id": "us-nyc-wg-301",
        "provider_url": "https://mullvad.net/servers",
        "provider_name": "Mullvad VPN",
        "provider_homepage_url": "https://mullvad.net"
      }
    }
    "#;
    crate::cmd::check_cmd_json::<CreateTunnel>(Some(cmd_json), Some(output_json));
}

#[test]
fn test_json_named() {
    let cmd_json = r#"
    {
        "type": "obfuscated",
        "label": "Banking",
        "id": "bd309cd5-e6e7-40b0-82d8-dbacdc827cb6",
        "wg_pubkey": "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=",
        "relay": "NYC-001",
        "exit": "NYC-001"
    }
    "#;
    let output_json = r#"
    {
      "id": "3f221f13-7002-458d-a447-595aba1a372d",
      "label": "Banking",
      "status": {
        "type": "disconnected",
        "when": 1725050273
      },
      "config": {
        "type": "obfuscated",
        "client_pubkey": "wjaiHUEOJ8k3X+U3b6H6yTcipqFipIbFQSB0CwZDNlQ=",
        "client_ips_v4": ["10.150.177.7/32"],
        "client_ips_v6": ["fc00:bbbb:bbbb:bb01:d:0:16:b107/128"],
        "dns": ["10.64.0.1"],
        "gateway_ip_v4": "10.64.0.1",
        "relay_addr_v4": "8.8.31.3:443",
        "relay_addr_v6": "[2001:db8:1234:ffff:ffff:ffff:ffff:ffff]:443",
        "relay_cert": "asdf",
        "exit_pubkey": "4s9JIhxC/D02tosXYYcgrD+pHI+C7oTAFsXzVisKjRs="
      },
      "relay": {
        "id": "NYC-001",
        "ip_v4": "8.8.31.3",
        "ip_v6": "2001:db8:1234:ffff:ffff:ffff:ffff:ffff",
        "preferred_exits": [{ "id": "nyc-wg-30" }],
        "tls_cert": "MIIBWjCCAQGgAwIBAgIVAK3WuHUPFg+mmBGiDhW9VNjDmudKMAoGCCqGSM49BAMCMCExHzAdBgNVBAMMFnJjZ2VuIHNlbGYgc2lnbmVkIGNlcnQwIBcNNzUwMTAxMDAwMDAwWhgPNDA5NjAxMDEwMDAwMDBaMCExHzAdBgNVBAMMFnJjZ2VuIHNlbGYgc2lnbmVkIGNlcnQwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAASLPcQhOgBGP8HRlGBy6rTO7SWnGgSuCEW6hi+Q/wnUr8H9FkJC7UGD9738XGKEq35ZKybrekr0DbK8YsVZ8SX+oxQwEjAQBgNVHREECTAHggVyZWxheTAKBggqhkjOPQQDAgNHADBEAiAYnVn/bozbp6u0iShFfUgbdGvAvf/hXwLhgonj+Fh+xAIgUgfQqvWFPbh3BNu98LhNFrESngTHtYYAOWt3wZdQOm4=",
        "ports": [53, 443]
      },
      "exit": {
        "id": "NYC-001",
        "country_code": "US",
        "city_code": "nyc",
        "city_name": "New York",
        "provider_id": "us-nyc-wg-301",
        "provider_url": "https://mullvad.net/servers",
        "provider_name": "Mullvad VPN",
        "provider_homepage_url": "https://mullvad.net"
      }
    }
    "#;
    crate::cmd::check_cmd_json::<CreateTunnel>(Some(cmd_json), Some(output_json));
}

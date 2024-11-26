use crate::cmd::Cmd;
use crate::types::OneTunnel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListTunnels {}
impl Cmd for ListTunnels {
    type Output = Vec<OneTunnel>;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = super::PATH;
}

#[test]
fn test_json() {
    let output_json = r#"
    [{
      "id": "dc799918-7738-446f-b1fc-ae3ba98103c7",
      "status": {
        "type": "created",
        "when": 1725050273
      },
      "config": {
        "type": "udp_port",
        "client": {
          "wg_pubkey": "wjaiHUEOJ8k3X+U3b6H6yTcipqFipIbFQSB0CwZDNlQ=",
          "addresses": [
            "10.150.177.7/32",
            "fc00:bbbb:bbbb:bb01:d:0:16:b107/128"
          ]
        },
        "server": {
          "wg_pubkey": "4s9JIhxC/D02tosXYYcgrD+pHI+C7oTAFsXzVisKjRs=",
          "endpoints": [
            "121.127.40.52:23527",
            "[::1]:23527"
          ],
          "dnses": [
            "10.64.0.1"
          ]
        }
      },
      "relay": {
        "id": "NYC-001",
        "ip_v4": "8.8.31.3",
        "ip_v6": "2001:db8:1234:ffff:ffff:ffff:ffff:ffff"
      },
      "exit": {
        "id": "NYC-001",
        "country_code": "US",
        "city_name": "New York"
      }
    }]
    "#;
    crate::cmd::check_cmd_json::<ListTunnels>(None, Some(output_json));
}

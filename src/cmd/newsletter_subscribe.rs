use crate::types::SubscriptionTarget;
use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsletterSubscribe {
    pub target: SubscriptionTarget,

    /// Set of subscriptions to subscribe to, they are identified by strings.
    ///
    /// This is required, for legacy reasons if not provided or empty it defaults to the general newsletter.
    #[serde(default)]
    pub subscriptions: Vec<String>,
}

impl Cmd for NewsletterSubscribe {
    type Output = ();
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "newsletter/subscribe";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<NewsletterSubscribe>(
        Some(
            r#"
            {
                "subscriptions": [
                    "blog"
                ],
                "target": {
                    "type": "email",
                    "addr": "me@example"
                }
            }
            "#,
        ),
        None,
    );

    crate::cmd::check_cmd_json::<NewsletterSubscribe>(
        Some(
            r#"
            {
                "subscriptions": [
                    "platform-windows",
                    "platform-android"
                ],
                "target": {
                    "type": "nostr",
                    "addr": "me@example"
                }
            }
            "#,
        ),
        None,
    );
}

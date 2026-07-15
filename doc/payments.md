# Payments

This is a guide to how to use the Obscura API to create and fund accounts.

<div class="warning">

This API is only officially supported for our partners. If you are interested in partnering with Obscura please contact <support@obscura.net>.

</div>

## Background Information

This documentation will be critical to understanding how to manage funding.

- [Account Number](crate::types::AccountId)
- [Account Creation](crate::cmd::CreateAccount)
- [Authentication](crate::doc::auth)
- [Errors](crate::doc::error)
- [Making Requests](crate::doc::requests)

## Example Flows

These are some example flows that highlight how the API can be used for common integration patterns. The links go to reference documentation.

### One-Click Top-Up

If you are a crypto wallet and would like to let your user easily subscribe to Obscura the flow would look something like this:

1. Request the user's Account Number.
1. [Fetch the account info](crate::cmd::GetAccountInfo) to validate the current status. If the account is already [`active`](crate::types::AccountInfo::active) check the [`current_expiry`](crate::types::AccountInfo::current_expiry) field. If it is `None` or far in the future then inform the user about the time left and confirm they do want to top-up now.
1. [Fetch the prices](crate::cmd::ListPrices) and ask the user to select the desired package.
1. Generate an invoice ([Monero](crate::cmd::CreateMoneroTopUp) or [Lightning](crate::cmd::CreateLightningTopUp)) and confirm the crypto price with the user.
1. Record the ID ([Monero](crate::cmd::MoneroTopUpInfo::id) or [Lightning](crate::cmd::LightningTopUpInfo::id)) for the user. This is their payment reference if they ever need to contact support.
1. Pay the invoice.
1. Poll the check status API ([Monero](crate::cmd::CheckMoneroTopUp) or [Lightning](crate::cmd::CheckLightningTopUp)).
1. Let the user know when their account is funded.

### Auto Top-up

While none of the supported currencies support native subscriptions a wallet could help the user avoid forgetting to fund their account.

1. Request the user's Account Number.
1. [Fetch the account info](crate::cmd::GetAccountInfo) to validate the current status.
	- If the account is not [`active`](crate::types::AccountInfo::active) then the user will be making the first payment to fund the account immediately.
1. Poll the account info. Daily is a reasonable rate.
1. Wait until [`current_expiry`](crate::types::AccountInfo::current_expiry) is soon (how soon is user preference).
1. Notify the user that their account is expiring, or with user authorization automatically perform a top-up.
1. [Fetch the prices](crate::cmd::ListPrices) and determine the appropriate package.
1. Generate an invoice ([Monero](crate::cmd::CreateMoneroTopUp) or [Lightning](crate::cmd::CreateLightningTopUp)) and confirm the price is within the user's authorized parameters.
1. Pay the invoice.
1. Poll the check status API ([Monero](crate::cmd::CheckMoneroTopUp) or [Lightning](crate::cmd::CheckLightningTopUp)) and notify the user if the payment isn't processed in an appropriate time period.

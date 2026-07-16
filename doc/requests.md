# Making Obscura API Requests

The Obscura API is served at <https://v1.api.prod.obscura.net/api/>.

## Rust Definitions

The API is described via Rust types in the [obscuravpn-api](https://github.com/Sovereign-Engineering/obscuravpn-api) crate.

These docs contain cross-referenced links to all API calls including the request and response types.

Each API endpoint (other than [the authentication endpoint](crate::doc::auth)) implements the [`Cmd`](crate::cmd::Cmd) trait. You can find a list of all API endpoints by checking [Implementors](crate::cmd::Cmd#implementors) on that page.

This trait specifies the key information of the API call.

- The type `Cmd` is implemented on is the request body.
	- For GET queries it is serialized as URL parameters.
		- For example: [`ListPrices::code`](crate::cmd::ListPrices::code) becomes `?code=xxx`.
	- Otherwise it is serialized as JSON and sent in the request body.
		- For example: [`CreateLightningTopUp`](crate::cmd::CreateLightningTopUp) is sent as the request body `{"months":1}`.
- The [`PATH`](crate::cmd::Cmd::PATH) constant indicates the URL path.
	- For example [`GetAccountInfo`](crate::cmd::GetAccountInfo) has a `PATH` of `"account"`. Which means that the request should be made to <https://v1.api.prod.obscura.net/api/account>.
- The [`METHOD`](crate::cmd::Cmd::METHOD) constant indicates the HTTP method to use.
	- For example [`GetAccountInfo`](crate::cmd::GetAccountInfo) has a `METHOD` of `GET`.

## Other Key Docs

- [Authentication](crate::doc::auth)
- [Errors](crate::doc::error)
- [User-Agent Requirements](crate::doc::user_agent)

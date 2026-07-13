# Authentication

Authentication is a simple process.

1. [Create an account](crate::doc::account_creation).
2. [Exchange Account Number for an auth token](crate::token::AcquireToken).
3. [Make requests with the auth token.](crate::token::AcquireToken2Output::auth_token)

While some endpoints don't currently require auth it is recommended to provide an auth token on every request (other than the auth request itself). This may enable more favourable rate limiting, higher quality responses and avoid blocks.

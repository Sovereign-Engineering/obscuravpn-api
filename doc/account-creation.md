# Account Creation

Account creation is implicit.

Follow these steps:

## Generate an Account Number

About account numbers: [Account Number](crate::types::AccountId)

1. Generate a securely random 19-digit integer string (with leading zeros as appropriate).
1. Calculate and append the [Verhoeff check digit](https://en.wikipedia.org/wiki/Verhoeff_algorithm).

## Create the Account

Use the [`AcquireToken`](crate::token::AcquireToken) call to validate the Account Number and availability of new accounts.

# Obscura VPN API Client

## Unstable

This crate and the API behind it are unstable in a number of ways:

1. The API of this crate may be broken at any time.
2. This crate may expose experimental interfaces and services which are completely unsupported. There is no indication of which interfaces these are. Both the Rust APIs and the backing APIs and implementations may change at any time with no notice.

## Support

No support is provided for this code or the API directly. However, if you are experiencing issues with your Obscura VPN service please contact <support@obscura.net>.

## Contributions

At this time we are unable to accept external contributions. This is something that we plan to resolve soon. However until we finish the paperwork we are unable to look at any patches and will close all PRs without looking at them.

## Development

To enter a setup environment run:

```sh
nix develop
```

To run single commands in the environment run:

```sh
nix develop -c just lint
```

## Static WireGuard UDP relay tunnel

Warning: This is not an officially supported service.

To create a UDP relay tunnel and write the matching WireGuard configuration to `wg0.conf`.

```bash
cargo run --example api_cli -- --account-no $OBS_ACCOUNT_ID create-static-tunnel --wg-conf > wg0.conf
```

The resulting configuration file should be compatible with any WireGuard client.
```bash
wg-quick up ./wg0.conf
```

### Tunnel deletion

Idle static tunnels are not removed automatically by clients if no tunnel slots are left.
You may use this command to delete all tunnels if you run out of tunnel slots:
```bash
cargo run --example api_cli -- --account-no $OBS_ACCOUNT_ID delete-all-tunnels
```

### Common issues

- The API or your internet stopped working unexpectedly? You probably deleted the tunnel while your WireGuard client was connected.
- Can't connect to a newly created tunnel? You are probably connected to another tunnel already.

In any case, the solution is almost always:
```bash
wg-quick down ./wg0.conf
```

## Utilities

### Generating a Valid User ID

```bash
cargo run --example gen_id
```

Note that this generates a user id _with_ the checksum (20 characters).

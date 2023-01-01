jacanaoesta
===========

Simple tool to list the people you follow on Mastodon that no longer are active.

## Current usage:

```
Find people that no longer are active in your Mastodon follow list.

Usage: jacanaoesta [OPTIONS] <instance>

Arguments:
  <instance>

Options:
  -k, --api-key      Ask for API key
  -d, --days <days>  Days since last status to consider inactive [default: 180]
  -h, --help         Print help information
  -V, --version      Print version information
```

Expected output:

```
Paste API Key here:
Found 171 users. Checking...
veracrypt (https://mastodon.social/@veracrypt) seems to be inactive
...
fsf (https://status.fsf.org/fsf) seems to be inactive
gnome (https://quitter.no/gnome) seems to be inactive
38 of them seem to be inactive for at least 180 days
```

The `api-key` needs `read:accounts` scope.

## Build from source

1. Clone repository
2. Run `cargo build --release`
3. Find the binary in `./targets/release/jacanaoesta`

For development purposes `cargo run` should be enough.

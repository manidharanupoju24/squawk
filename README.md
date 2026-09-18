# squawk 

A tiny `dig`-style DNS lookup CLI written in Rust — query any resolver directly to see how a name **actually** resolves.

In aviation, a transponder *squawks* a code back when it's interrogated. That's exactly what a DNS server does when you ask it a question — so that's the name.

```console
$ squawk cloudflare.com -s 1.1.1.1
;; server: 1.1.1.1:53
;; question: cloudflare.com A

cloudflare.com.    300  A 104.16.132.229
cloudflare.com.    300  A 104.16.133.229

;; 2 answer(s) in 19.6ms
```

## Why

`dig` is great, but sometimes you just want a small, focused tool that:

- points at a **specific** resolver (`-s`) so you can compare what different servers answer for the same name, and
- prints a clean, quiet, dig-like summary with the query latency.

It's handy for checking whether a DNS record is live at its authoritative source (e.g. an Infoblox grid member) versus what clients see through the normal resolution path, and for spot-checking GTM / edge answers from Akamai.

## Install

Requires **Rust 1.85+** (the project uses edition 2024).

```console
$ git clone https://github.com/svaughtAA/squawk.git
$ cd squawk
$ cargo install --path .
```

This builds in release mode and installs the `squawk` binary to `~/.cargo/bin/`, which is already on your `PATH` if you installed Rust with rustup.

Prefer not to install? Run it straight from the project with `cargo run -- <args>`.

## Usage

```console
$ squawk <name> [OPTIONS]
```

| Option              | Short | Default          | Description                                        |
| ------------------- | :---: | ---------------- | -------------------------------------------------- |
| `<name>`            |       | *(required)*     | The domain name to look up.                        |
| `--type`            | `-t`  | `A`              | Record type: `A`, `AAAA`, `CNAME`, `TXT`, `MX`, `NS`, `SOA`, `SRV`, `CAA`, … |
| `--server`          | `-s`  | system resolver  | DNS server IP to query directly.                   |
| `--port`            | `-p`  | `53`             | Port for the DNS server.                           |
| `--help`            | `-h`  |                  | Print help.                                        |

## Examples

```console
# A record via the system resolver
$ squawk example.com

# IPv6
$ squawk example.com -t AAAA

# Follow a CNAME
$ squawk www.github.com -t CNAME

# Mail servers
$ squawk google.com -t MX

# Ask a specific resolver directly
$ squawk cloudflare.com -s 1.1.1.1

# Same question, different server — do they agree?
$ squawk cloudflare.com -s 8.8.8.8

# Hit an internal resolver on a non-standard port
$ squawk myapp.internal.example.com -s 10.0.0.53 -p 53
```

A name with no records of the requested type is reported cleanly rather than treated as a crash:

```console
$ squawk example.com -t TXT
;; server: system resolver
;; question: example.com TXT

;; no TXT records for example.com (12.4ms)
```

## Sanity-checking against `dig`

The quickest way to trust the output is to diff it against the reference tool, pointing both at the same server:

```console
$ squawk google.com -s 8.8.8.8
$ dig @8.8.8.8 google.com +short
```

The answers should line up. (Exact IPs may still rotate within a provider's pool, and TTLs count down between queries — both are normal DNS behavior, not discrepancies.)

## Built with

- [`clap`](https://crates.io/crates/clap) — argument parsing via derive macros
- [`tokio`](https://crates.io/crates/tokio) — async runtime
- [`hickory-resolver`](https://crates.io/crates/hickory-resolver) — the DNS resolver
- [`anyhow`](https://crates.io/crates/anyhow) — ergonomic error handling

## License

MIT — see [`LICENSE`](LICENSE).

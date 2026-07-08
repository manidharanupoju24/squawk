// squawk - a tiny `dig` like DNS lookup tool 
//
// Usage examples: 
//      squawk example.com                              # A record via system resolver
//      squawk example.com -t AAAA                      # IPv6
//      squawk example.com -t TXT                       # TXT records
//      squawk myapp.example.com -s 1.1.1.1             # query a specific server
//      squawk myapp.example.com -s 10.0.0.53 -p 53     # e.g. your infoblox grid member 

use std::net::IpAddr;
use std::str::FromStr;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use hickory_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
use hickory_resolver::error::ResolveErrorKind;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioAsyncResolver;

// clap turns this struct into a full CLI parser via the 'derive' macro
// Each field becomes a flag/argument 
// doc comments become --help text


#[derive(Parser)]
#[command(name = "squawk", about= "A tiny dig-like DNS lookup tool")]
struct Cli {
    /// domain name to look up (e.g. example.com)
    name: String,

    /// Record type: A, AAAA, CNAME, TXT, MX, NS, SOA, SRV, CAA..
    #[arg(short = 't', long = "type", default_value = "A")]
    record_type: String,

    /// DNS server IP to query directly. Omit to use the system resolver.
    /// Option<String> means "maybe a value, maybe not" - Rust makes the 
    /// absence explicit instead of null. 
    #[arg(short = 's', long = "server")]
    server: Option<IpAddr>,

    ///Port for the DNS server (default 53).
    #[arg(short = 'p', long = "port", default_value_t = 53)]
    port: u16,

}

// `#[tokio::main]` sets up the async runtime and lets `main` be `async`.
// Returning `Result<()>` means any `?` that fails will bubble up and print 
// a clean error instead of panic
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse the type string ("a", "AAAA", ...) into hickory's RecordType enum.
    // `.to_uppercase()` normalizes user input; `?` returns early on bad type.
    let rtype = RecordType::from_str(&cli.record_type.to_uppercase())
        .with_context(|| format!("unknown record type: {}", cli.record_type))?;

    // Build the resolver. Two branches:  a specific server, or the system config.
    // `match` on the Option is Rust's null-safe fork in the road.
    let resolver = match cli.server {
        Some(ip) => {
            // from_ips_clear = plain UDP/TCP (no DNS-over-TLS) 
            let group = NameServerConfigGroup::from_ips_clear(&[ip], cli.port, true);
            let config = ResolverConfig::from_parts(None, vec![], group);
            TokioAsyncResolver::tokio(config, ResolverOpts::default())
        }
        None => {
            // fall back to /etc/resolv.conf (or platform equivalent).
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())
        }
    };

    // Print a small header, dig-style
    match cli.server {
        Some(ip) => println!(";; server: {ip}:{}", cli.port),
        None => println!(";; server: system resolver"), 
    }

    println!(";; question: {} {}\n", cli.name, rtype);

    // Time the query so you can see latency (useful for edge comparisions).
    let started = Instant::now();
    let result = resolver.lookup(cli.name.as_str(), rtype).await;
    let elapsed = started.elapsed();

    // Not every failure is a real error. "No records" is a normal DNS answer,
    // so we match on the specific error variant and treat it gently. Anything 
    // else (timeout, connection refused, SERVFAIL) is a genuine error we return.
    let lookup = match result {
        Ok(lookup) => lookup, 
        Err(err) => match err.kind() {
            ResolveErrorKind::NoRecordsFound { .. } => {
                println!(";; no {rtype} records for {} ({:.1?})", cli.name, elapsed);
                return Ok(());
            }
            _ => {
                return Err(anyhow::Error::new(err)
                .context(format!("lookup failed for {} {}", cli.name, rtype)));
            }
        },
    };

    // `records()` gives a slice of the answer records. We iterate and print each
    let mut count = 0;
    for record in lookup.records() {
        // `if let Some(..)` unpacks the Option only when data is present. 
        if let Some(data) = record.data() {
            println!(
            "{name:<40} {ttl:>6} {rtype:<6} {data}",
            name = record.name(),
            ttl = record.ttl(),
            rtype = record.record_type(),
            data = data,
        );
            count += 1;
        }
    }

    println!("\n;; {count} answer(s) in {:.1?}", elapsed);
    Ok(())
}


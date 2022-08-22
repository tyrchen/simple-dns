# Simple DNS

Sometimes I need a simple DNS server which could be configured easily (without touching zone file format) and be integrated to a Rust program to serve DNS queries to specific domains. So I wrote this simple DNS server. It utilize awesome [trust-dns](https://github.com/bluejekyll/trust-dns) to do DNS server and DNS forwarding. The server will automatically create SOA records for your domains (you don't need to define them). If the server can't find a match in its configuration, it will forward the query to google DNS server (8.8.8.8:53) automatically.

## Usage

To build a DNS server, you just need to load configuration, and run server.

```Rust
use anyhow::Result;
use simple_dns::{Config, SimpleDns};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config: Config = serde_yaml::from_str(include_str!("../fixtures/config.yaml"))?;

    let server = SimpleDns::try_load(config).await?;

    // this will block. You can use tokio::spawn to run it in your program
    server.run().await?;

    Ok(())
}
```

The configuration looks like this:

```yaml
---
bind: 0.0.0.0:53
domains:
  tyr.test:
    - name: '@'
      records: [127.0.0.1]
    - name: www
      records: [127.0.0.2]
    - name: '*'
      records: ['www']
      type: CNAME
  tyr1.test:
    - name: '@'
      records: [127.0.0.1]
    - name: 'abc'
      records: [127.0.0.1]
```

Don't forget to create a `/etc/resolver/local` file and put `nameserver 127.0.0.1` in it. Like this:

```bash
$ cat /etc/resolver/local
nameserver 127.0.0.1
```

Once your server started, you can use dig to verify the result.

```bash
➜ dig @127.0.0.1 tyr.test

; <<>> DiG 9.10.6 <<>> @127.0.0.1 tyr.test
; (1 server found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 27566
;; flags: qr aa rd; QUERY: 1, ANSWER: 1, AUTHORITY: 0, ADDITIONAL: 0
;; WARNING: recursion requested but not available

;; QUESTION SECTION:
;tyr.test.			IN	A

;; ANSWER SECTION:
tyr.test.		3600	IN	A	127.0.0.1

;; Query time: 2 msec
;; SERVER: 127.0.0.1#53(127.0.0.1)
;; WHEN: Sun Aug 21 23:11:01 PDT 2022
;; MSG SIZE  rcvd: 50
```

Below is the server output for the query:

```bash
➜ RUST_LOG=info cargo run --example server
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `~/.target/debug/examples/server`
2022-08-22T06:10:59.474271Z  INFO trust_dns_server::store::forwarder::authority: loading forwarder config: .
2022-08-22T06:10:59.475073Z  INFO trust_dns_server::store::forwarder::authority: forward resolver configured: .:
2022-08-22T06:11:01.746236Z  INFO trust_dns_server::server::server_future: request:27566 src:UDP://127.0.0.1#52589 QUERY:tyr.test.:A:IN qflags:RD,AD response:NoError rr:1/0/0 rflags:RD,AA
```

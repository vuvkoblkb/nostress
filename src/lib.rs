use rand::{seq::SliceRandom, Rng};
use std::fs;
use std::sync::Arc;

/// Membaca file eksternal dan return Vec<String>
pub fn load_file(filename: &str) -> Vec<String> {
    fs::read_to_string(filename)
        .unwrap_or_else(|_| String::new())
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect()
}

/// Mendapatkan item random dari Vec. Fallback ke default jika kosong.
pub fn pick_random<'a>(list: &'a [String], default: &'a str) -> &'a str {
    list.choose(&mut rand::thread_rng())
        .map(|s| s.as_str())
        .unwrap_or(default)
}

/// Membuat IP random v4
pub fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "{}.{}.{}.{}",
        rng.gen_range(1..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(1..255)
    )
}

/// Payload random untuk POST
pub fn random_payload() -> String {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(20..200);
    (0..len)
        .map(|_| (0x20u8 + (rng.gen::<u8>() % 95)) as char)
        .collect()
}

/// Semua config eksternal dalam satu struct
pub struct ConfigLists {
    pub proxies: Arc<Vec<String>>,
    pub referers: Arc<Vec<String>>,
    pub useragents: Arc<Vec<String>>,
    pub ciphers: Arc<Vec<String>>,
    pub sigalgs: Arc<Vec<String>>,
    pub paths: Arc<Vec<String>>,
    pub langs: Arc<Vec<String>>,
    pub accepts: Arc<Vec<String>>,
    pub encodings: Arc<Vec<String>>,
    pub controles: Arc<Vec<String>>,
}

impl ConfigLists {
    pub fn load(proxyfile: &str) -> Self {
        Self {
            proxies: Arc::new(load_file(proxyfile)),
            referers: Arc::new(load_file("referers.txt")),
            useragents: Arc::new(load_file("useragents.txt")),
            ciphers: Arc::new(load_file("ciphers.txt")),
            sigalgs: Arc::new(load_file("sigalgs.txt")),
            paths: Arc::new(load_file("paths.txt")),
            langs: Arc::new(load_file("lang_header.txt")),
            accepts: Arc::new(load_file("accept_header.txt")),
            encodings: Arc::new(load_file("encoding_header.txt")),
            controles: Arc::new(load_file("controle_header.txt")),
        }
    }
}
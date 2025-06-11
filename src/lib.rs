use rand::{seq::SliceRandom, Rng};
use std::{fs, sync::Arc};
use reqwest::Method;

/// Load file, Vec<String>
pub fn load_file(filename: &str) -> Vec<String> {
    fs::read_to_string(filename)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect()
}
pub fn pick_random<'a>(list: &'a [String], default: &'a str) -> &'a str {
    list.choose(&mut rand::thread_rng()).map(|s| s.as_str()).unwrap_or(default)
}
pub fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!("{}.{}.{}.{}", rng.gen_range(1..255), rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(1..255))
}
pub fn random_str(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| {
        let c = rng.gen_range(32u8..127u8) as char;
        if rng.gen_bool(0.1) { std::char::from_u32(rng.gen_range(0x1F600..0x1F64F)).unwrap_or(c) } else { c }
    }).collect()
}
pub fn random_payload() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(0..4096);
    (0..len).map(|_| rng.gen::<u8>()).collect()
}
pub fn random_method() -> String {
    let normal = ["GET","POST","PUT","PATCH","DELETE","HEAD","OPTIONS","TRACE","CONNECT"];
    let weird = ["BREACH","FOOBAR","HACK","BYPASS","GIBBERISH","XD","🐞","攻撃"];
    let mut all = normal.to_vec();
    all.extend_from_slice(weird);
    all.choose(&mut rand::thread_rng()).unwrap().to_string()
}
pub fn random_header_case(s: &str) -> String {
    let mut rng = rand::thread_rng();
    s.chars().map(|c| if rng.gen_bool(0.5) { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }).collect()
}
pub fn random_path(paths: &[String]) -> String {
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
pub fn random_content_type() -> String {
    let types = [
        "application/json", "application/x-www-form-urlencoded", "multipart/form-data",
        "text/plain", "application/xml", "image/png", "application/octet-stream",
        "text/🐞", "x-blackhole", "application/x-unknown"
    ];
    types.choose(&mut rand::thread_rng()).unwrap().to_string()
}
pub fn random_cookies() -> String {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(1..6);
    (0..count)
        .map(|_| format!("{}={}", random_str(rng.gen_range(3..7)), random_str(rng.gen_range(5..12))))
        .collect::<Vec<_>>().join("; ")
}
pub fn random_headers(base: &[(&str, String)]) -> Vec<(String, String)> {
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
pub fn random_http_version() -> &'static str {
    let versions = ["HTTP/1.0", "HTTP/1.1", "HTTP/2", "HTTP/0.9", "HTTP/3.14"];
    versions.choose(&mut rand::thread_rng()).unwrap()
}
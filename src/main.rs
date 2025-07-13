use futures_util::StreamExt;
use itertools::Itertools;
use reqwest::header::HeaderMap;
use reqwest::{header, Client, Proxy};
use scraper::Selector;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use url::Url;

const DICT: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
const COMB_LEN: u8 = 6;
const DELAY: u8 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // удалённые - //st.prntscr.com/2023/07/24/0635/img/0_173a7b_211be8ff.png
    // id тега - screenshot-image

    let mut headers = HeaderMap::new();
    // headers.insert(header::HOST, "prnt.sc".parse().unwrap());
    headers.insert(
        header::USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0"
            .parse()
            .unwrap(),
    );
    // headers.insert(header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap());
    headers.insert(header::ACCEPT_LANGUAGE, "en-US,en;q=0.5".parse().unwrap());
    // headers.insert(header::ACCEPT_ENCODING, "gzip, deflate, br, zstd".parse().unwrap());
    headers.insert(header::DNT, "1".parse().unwrap());
    headers.insert("Sec-GPC", "1".parse().unwrap());
    headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(header::COOKIE, "language=ru".parse().unwrap());
    headers.insert(header::UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    headers.insert("Priority", "u=0, i".parse().unwrap());

    let client = reqwest::ClientBuilder::new()
        // .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0")
        .default_headers(headers)
        .proxy(Proxy::all("socks5://127.0.0.1:2080").unwrap())
        .build()?;

    let comb_iter = (0..COMB_LEN).map(|_| DICT).multi_cartesian_product();

    // let mut futures = Vec::new();
    for comb_vec in comb_iter.take(100) {
        let client = client.clone();
        // futures.push(async move {
        let comb: String = comb_vec.into_iter().collect();
        let url = format!("https://prnt.sc/{comb}");
        println!("Getting {url}");

        let text = client
            .get(url)
            .send()
            .await?
            // .expect("Error sending request")
            .text()
            .await?;
        // .expect("Error getting text");
        // println!("{text}");

        let html = scraper::Html::parse_document(&text);
        let sel = Selector::parse(r#"img#screenshot-image"#).unwrap();
        if let Some(element) = html.select(&sel).next() {
            if let Some(src) = element.value().attr("src") {
                println!("src: {src}");
                if src != "//st.prntscr.com/2023/07/24/0635/img/0_173a7b_211be8ff.png" {
                    if let Ok(url) = Url::parse(src) {
                        let path = Path::new(url.path());
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap();
                        let path = format!("img/{comb}.{ext}");

                        println!("{comb} found");
                        download(client, src, &path).await?;
                        // .expect("Download failed");
                    }
                } else {
                    println!("{comb} not found")
                }
            } else {
                println!("{comb} src not found")
            }
        } else {
            println!("img#screenshot-image not found.");
        }

        sleep(Duration::from_secs(DELAY as u64)).await;
        // });
    }
    // join_all(futures).await;

    Ok(())
}

async fn download(client: Client, src: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.get(src).send().await?;
    if !resp.status().is_success() {
        return Err(format!("Failed with status {}", resp.status()).into());
    }

    let mut file = File::create(path).await?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }

    Ok(())
}

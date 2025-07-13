use futures_util::StreamExt;
use itertools::Itertools;
use reqwest::header::HeaderMap;
use reqwest::{Client, Proxy, header};
use scraper::Selector;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use tokio::{fs, io};
use url::Url;

const DICT: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
const COMB_LEN: u8 = 6;
const FILE_BASE: &str = "img";
const DELAY: u8 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client().await?;

    let comb_iter = (0..COMB_LEN).map(|_| DICT).multi_cartesian_product();

    for comb_vec in comb_iter {
        // .take(1000000)
        let comb: String = comb_vec.into_iter().collect();
        if is_founded_file(&comb).await? {
            continue;
        }

        let client = client.clone();
        let url = format!("https://prnt.sc/{comb}");
        println!("Getting {url}");

        let text = client.get(url).send().await?.text().await?;

        let html = scraper::Html::parse_document(&text);
        let sel = Selector::parse(r#"img#screenshot-image"#).unwrap();
        if let Some(src) = html.select(&sel).next().and_then(|e| e.value().attr("src")) {
            println!("src: {src}");

            if src == "//st.prntscr.com/2023/07/24/0635/img/0_173a7b_211be8ff.png" {
                println!("{comb} not found\n");
                continue;
            }

            if let Ok(url) = Url::parse(src) {
                let path = Path::new(url.path());
                let ext = path.extension().and_then(|s| s.to_str()).unwrap();
                let path = format!("{FILE_BASE}/{comb}.{ext}");

                println!("{comb} downloading\n");
                download(client, src, &path).await?;
            }
        } else {
            if text == "error code: 1006" {
                panic!("Your IP banned")
            }

            panic!("img#screenshot-image or src not found.");
        }

        sleep(Duration::from_secs(DELAY as u64)).await;
    }

    Ok(())
}

async fn get_client() -> reqwest::Result<Client> {
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

    Ok(client)
}

async fn download(client: Client, src: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.get(src).send().await?;
    if !resp.status().is_success() {
        println!("Failed with status {}", resp.status());
        return Ok(());
    }

    let mut file = File::create(path).await?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }

    Ok(())
}

async fn is_founded_file(comb: &str) -> io::Result<bool> {
    let mut entries = fs::read_dir(FILE_BASE).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem == comb {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

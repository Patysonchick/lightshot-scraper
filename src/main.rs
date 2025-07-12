use futures_util::StreamExt;
use itertools::Itertools;
use reqwest::{header, Proxy};
use reqwest::header::HeaderMap;
use scraper::Selector;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const DICT: [char; 36] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const COMB_LEN: u8 = 6;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // удалённые - //st.prntscr.com/2023/07/24/0635/img/0_173a7b_211be8ff.png
    // id тега - screenshot-image

    let mut headers = HeaderMap::new();
    // headers.insert(header::HOST, "prnt.sc".parse().unwrap());
    headers.insert(header::USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0".parse().unwrap());
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

    let comb_iter = (0..COMB_LEN)
        .map(|_| DICT)
        .multi_cartesian_product();

    for comb_vec in comb_iter.take(100) {
        let comb: String = comb_vec.into_iter().collect();
        let url = format!("https://prnt.sc/{comb}");
        println!("Getting {url}");

        let text = client.get(url).send().await?.text().await?;
        // println!("{text}");

        let html = scraper::Html::parse_document(&text);
        let sel = Selector::parse(r#"img#screenshot-image"#).unwrap();
        if let Some(element) = html.select(&sel).next() {
            if let Some(src) = element.value().attr("src") {
                let path = format!("img/{comb}.png");
                println!("src: {}", src);

                // let full_url = if src.starts_with("//") {
                //     format!("https:{}", src)
                // } else {
                //     src.to_string()
                // };
                // println!("Полный URL: {}", full_url);

                if src != "//st.prntscr.com/2023/07/24/0635/img/0_173a7b_211be8ff.png" {
                    println!("{comb} found");

                    let resp = client.get(src).send().await?;
                    if !resp.status().is_success() {
                        let error_message = format!("Ошибка: сервер ответил со статусом {}", resp.status());
                        return Err(error_message.into());
                    }

                    let mut file = File::create(path).await?;
                    let mut stream = resp.bytes_stream();

                    while let Some(chunk_result) = stream.next().await {
                        let chunk = chunk_result?;
                        file.write_all(&chunk).await?;
                    }
                } else {
                    println!("{comb} not found")
                }

            } else {
                panic!("src not found")
            }
        } else {
            panic!("img#screenshot-image not found.");
        }
    }

    Ok(())
}

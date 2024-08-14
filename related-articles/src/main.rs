use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{Read, Write},
    path, time,
};

#[derive(Debug, Deserialize, Serialize)]
struct Crate {
    id: String,
    name: String,
    homepage: String,
    repository: String,
    #[serde(default)]
    crates_io: String,
    #[serde(default)]
    docs_rs: String,
    #[serde(default)]
    links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Link {
    #[serde(default)]
    date: String,
    title: String,
    link: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinkCrate {
    crate_name: String,
    date: String,
    title: String,
    link: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HtmlPage {
    date: String,
    title: String,
    link: String,
    content_links: Vec<String>,
}

fn gen_sitemap() -> Result<(), Box<dyn Error>> {
    use sitemap_rs::sitemap::Sitemap;
    use sitemap_rs::sitemap_index::SitemapIndex;
    use sitemap_rs::url::{ChangeFrequency, Url};
    use sitemap_rs::url_set::UrlSet;

    let file = fs::File::open("crates.csv")?;
    let rdr = csv::Reader::from_reader(file);
    let mut iter = rdr.into_deserialize();

    let mut crates = Vec::new();

    while let Some(line) = iter.next() {
        let row: Crate = line?;
        crates.push(row);
    }

    // dbg!(&crates);

    let mut page_num = 1;
    let mut sitemap_crates = Vec::new();
    let mut sitemap_articles = Vec::new();
    for rows in &crates.into_iter().chunks(10_000) {
        dbg!(&page_num);

        sitemap_crates.push(Sitemap::new(
            format!("https://rustacean.info/sitemap-crates-{page_num:03}.xml"),
            None,
        ));
        sitemap_articles.push(Sitemap::new(
            format!("https://rustacean.info/sitemap-articles-{page_num:03}.xml"),
            None,
        ));

        let mut sitemap_crates_urls = Vec::new();
        let mut sitemap_articles_urls = Vec::new();

        for row in rows {
            let name = row.name;
            sitemap_crates_urls.push(
                Url::builder(format!("https://rustacean.info/crates/{}", name))
                    .change_frequency(ChangeFrequency::Daily)
                    .priority(0.8)
                    .build()?,
            );
            sitemap_articles_urls.push(
                Url::builder(format!("https://rustacean.info/crates/{}/articles", name))
                    .change_frequency(ChangeFrequency::Daily)
                    .priority(0.8)
                    .build()?,
            );
        }

        // dbg!(&sitemap_crates_urls);
        // dbg!(&sitemap_articles_urls);

        let sitemap_crates_urls = UrlSet::new(sitemap_crates_urls)?;
        let mut sitemap_crates_urls_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("../public/sitemap-crates-{page_num:03}.xml"))?;
        sitemap_crates_urls
            .write(&mut sitemap_crates_urls_file)
            .unwrap();

        let sitemap_articles_urls = UrlSet::new(sitemap_articles_urls)?;
        let mut sitemap_articles_urls_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("../public/sitemap-articles-{page_num:03}.xml"))?;
        sitemap_articles_urls
            .write(&mut sitemap_articles_urls_file)
            .unwrap();

        page_num += 1;
    }

    // dbg!(&sitemap_crates);
    // dbg!(&sitemap_articles);

    let sitemap_crates = SitemapIndex::new(sitemap_crates)?;
    let mut sitemap_crates_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../public/sitemap-crates-000.xml")?;
    sitemap_crates.write(&mut sitemap_crates_file).unwrap();

    let sitemap_articles = SitemapIndex::new(sitemap_articles)?;
    let mut sitemap_articles_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../public/sitemap-articles-000.xml")?;
    sitemap_articles.write(&mut sitemap_articles_file).unwrap();

    Ok(())
}

fn get_crates() -> Result<Vec<Crate>, Box<dyn Error>> {
    let file = fs::File::open("crates.csv")?;
    let rdr = csv::Reader::from_reader(file);
    let mut iter = rdr.into_deserialize();

    let mut crates = Vec::new();

    while let Some(line) = iter.next() {
        let mut row: Crate = line?;

        row.crates_io = format!("https://crates.io/crates/{}", row.name);
        row.docs_rs = format!("https://docs.rs/{}", row.name);

        row.repository = row.repository.trim_start_matches("https://").into();
        row.repository = row.repository.trim_start_matches("http://").into();
        row.crates_io = row.crates_io.trim_start_matches("https://").into();
        row.crates_io = row.crates_io.trim_start_matches("http://").into();
        row.homepage = row.homepage.trim_start_matches("https://").into();
        row.homepage = row.homepage.trim_start_matches("http://").into();
        row.docs_rs = row.docs_rs.trim_start_matches("https://").into();
        row.docs_rs = row.docs_rs.trim_start_matches("http://").into();

        row.repository = format!("{}/", row.repository.trim().trim_end_matches("/"));
        row.crates_io = format!("{}/", row.crates_io.trim().trim_end_matches("/"));
        row.homepage = format!("{}/", row.homepage.trim().trim_end_matches("/"));
        row.docs_rs = format!("{}/", row.docs_rs.trim().trim_end_matches("/"));

        let exact_match_filters = [
            "github.com/",
            "github.com",
            "www.google.com/",
            "www.google.com",
            "google.com/",
            "google.com",
            "facebook.com/",
            "facebook.com",
            "crates.io/",
            "crates.io",
            "example.com/",
            "example.com",
        ];
        for filter in exact_match_filters {
            if row.repository == filter {
                row.repository = "".into();
            }
            if row.crates_io == filter {
                row.crates_io = "".into();
            }
            if row.docs_rs == filter {
                row.docs_rs = "".into();
            }
            if row.homepage == filter {
                row.homepage = "".into();
            }
        }

        let prefix_match_filters = ["github.com/rust-lang/rust"];
        for filter in prefix_match_filters {
            if row.repository.starts_with(filter) {
                row.repository = "".into();
            }
            if row.crates_io.starts_with(filter) {
                row.crates_io = "".into();
            }
            if row.docs_rs.starts_with(filter) {
                row.docs_rs = "".into();
            }
            if row.homepage.starts_with(filter) {
                row.homepage = "".into();
            }
        }

        crates.push(row);
    }

    Ok(crates)
}

fn write_crates_to_json(crates: &[Crate]) -> Result<(), Box<dyn Error>> {
    let json_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("crates.json")?;
    serde_json::to_writer_pretty(json_file, crates)?;
    Ok(())
}

async fn curl_twir_links() -> Result<(), Box<dyn Error>> {
    let mut paths: Vec<_> = fs::read_dir("../../this-week-in-rust/content")?
        .map(|path| path.unwrap())
        .filter(|path| {
            !path
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .ends_with(".rst")
        })
        .collect();
    paths.sort_by_key(|dir| dir.file_name());
    paths.reverse();

    let mut all_links = Vec::new();

    for path in paths {
        if path.metadata()?.is_dir() {
            continue;
        }
        dbg!(&path);
        let date = path.file_name().as_os_str().to_str().unwrap()[0..10].to_string();
        // dbg!(&date);
        let mut file = fs::File::open(path.path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        // dbg!(&contents);
        let finder = linkify::LinkFinder::new();
        let mut links: Vec<_> = finder
            .spans(&contents)
            .map(|span| {
                let link = span.as_str().to_string();
                (date.clone(), link)
            })
            .filter(|(_, link)| link.starts_with("http"))
            .filter(|(_, link)| {
                let filters = [
                    "//github.com",
                    "this-week-in-rust",
                    "rust-lang.org",
                    "crates.io",
                    "x.com",
                    "twitter.com",
                    "reddit.com",
                    "meetup.com",
                    "lu.ma",
                    "google.com",
                    "youtu.be",
                    "youtube.com",
                    "example.com",
                ];
                for filter in filters {
                    if link.contains(filter) {
                        return false;
                    }
                }
                true
            })
            .collect();
        links = links
            .into_iter()
            .sorted_by_key(|(_, link)| link.clone())
            .dedup_by(|a, b| a.1 == b.1)
            .collect();
        // dbg!(&links);
        all_links.extend(links);
    }

    all_links = all_links
        .into_iter()
        .sorted_by_key(|(_, link)| link.clone())
        .dedup_by(|a, b| a.1 == b.1)
        .collect();

    dbg!(&all_links);
    dbg!(&all_links.len());

    all_links = all_links
        .into_iter()
        .sorted_by_key(|(date, _)| date.clone())
        .rev()
        .collect();

    let client = reqwest::Client::new();
    let total = all_links.len();

    for (i, (date, link)) in all_links.into_iter().enumerate() {
        let n = i + 1;
        println!("[{n} / {total}] {date} - {link}");

        let sanitise_link = sanitise_file_name::sanitise_with_options(
            &link,
            &sanitise_file_name::Options {
                length_limit: 200,
                ..Default::default()
            },
        );

        let file_name = format!("links/{date}_{sanitise_link}.html");

        if path::Path::new(&file_name).exists() {
            continue;
        }

        let response = client
            .get(&link)
            .timeout(time::Duration::from_secs(10))
            .send()
            .await;
        let html = match response {
            Ok(res) => {
                let status = res.status();
                println!("\t-> Status: {:?}", status);
                if status.as_u16() == 404 {
                    println!("\t-> 404 Not Found");
                    continue;
                }
                match res.text().await {
                    Ok(text) => text,
                    Err(err) => {
                        dbg!(&err);
                        continue;
                    }
                }
            }
            Err(err) => {
                dbg!(&err);
                continue;
            }
        };

        let mut html_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_name)?;

        html_file.write_all(format!("<!-- {date} -->\n").as_bytes())?;
        html_file.write_all(format!("<!-- {link} -->\n\n").as_bytes())?;
        html_file.write_all(html.as_bytes())?;
    }
    Ok(())
}

fn get_html_title(contents: &str) -> Result<Option<String>, Box<dyn Error>> {
    let dom = scraper::Html::parse_document(contents);
    let selector = scraper::Selector::parse("title")?;
    let title = dom
        .select(&selector)
        .next()
        .map(|node| node.inner_html().trim().to_string());
    Ok(title)
}

fn consolidate_crates_json() -> Result<(), Box<dyn Error>> {
    let mut crates = get_crates()?;
    // dbg!(&crates);

    let mut paths: Vec<_> = fs::read_dir("links")?
        .map(|path| path.unwrap())
        .filter(|path| {
            path.file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .ends_with(".html")
        })
        .collect();
    paths.sort_by_key(|dir| dir.file_name());
    paths.reverse();

    let num_paths = paths.len();

    let html_pages: Vec<_> = paths
        .par_iter()
        .enumerate()
        .map(|(i, path)| {
            let n = i + 1;
            println!("[{n:4} / {num_paths}] {:?}", path);

            let mut file = fs::File::open(path.path()).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let title = get_html_title(&contents).unwrap().unwrap_or_default();

            let s = contents
                .match_indices("<!-- ")
                .nth(0)
                .map(|(idx, _)| idx)
                .unwrap();
            let e = contents
                .match_indices(" -->")
                .nth(0)
                .map(|(idx, _)| idx)
                .unwrap();
            let date = contents[(s + 5)..e].to_string();

            let s = contents
                .match_indices("<!-- ")
                .nth(1)
                .map(|(idx, _)| idx)
                .unwrap();
            let e = contents
                .match_indices(" -->")
                .nth(1)
                .map(|(idx, _)| idx)
                .unwrap();
            let link = contents[(s + 5)..e].to_string();

            let finder = linkify::LinkFinder::new();
            let content_links: Vec<_> = finder
                .spans(&contents)
                .map(|span| span.as_str().to_string())
                .filter(|link| link.starts_with("http") && !link.starts_with("/"))
                .filter(|link| {
                    let link = link.to_lowercase();
                    !(link.ends_with(".css")
                        || link.ends_with(".js")
                        || link.ends_with(".pdf")
                        || link.ends_with(".png")
                        || link.ends_with(".jpg")
                        || link.ends_with(".jpeg")
                        || link.ends_with(".ico"))
                })
                .map(|link| {
                    let link = link
                        .trim()
                        .trim_start_matches("https://")
                        .trim_start_matches("http://");
                    format!("{}/", link.trim_end_matches("/"))
                })
                .collect();

            HtmlPage {
                date,
                title,
                link,
                content_links,
            }
        })
        .filter(|html_page| !html_page.title.starts_with("404: Not Found | Solana"))
        .collect();

    let num_crates = crates.len();

    println!("Starts matching");

    // let mut crates: Vec<_> = crates
    //     .into_iter()
    //     .filter(|crate_row| {
    //         // crate_row.name.starts_with("sea-")
    //         crate_row.name.starts_with("rustc-")
    //         // crate_row.name == "r"
    //     })
    //     .collect();

    crates
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, crate_row)| {
            let n = i + 1;
            println!("[{n:6} / {num_crates}] {}", crate_row.name);

            for html_page in html_pages.iter() {
                for content_link in html_page.content_links.iter() {
                    // if !crate_row.crates_io.is_empty() && content_link.starts_with(&crate_row.crates_io)
                    // {
                    //     dbg!((&crate_row.crates_io, &content_link));
                    // }
                    // if !crate_row.docs_rs.is_empty() && content_link.starts_with(&crate_row.docs_rs)
                    // {
                    //     dbg!((&crate_row.docs_rs, &content_link));
                    // }
                    // if !crate_row.repository.is_empty() && content_link.starts_with(&crate_row.repository)
                    // {
                    //     dbg!((&crate_row.repository, &content_link));
                    // }
                    // if !crate_row.homepage.is_empty() && content_link.starts_with(&crate_row.homepage)
                    // {
                    //     dbg!((&crate_row.homepage, &content_link));
                    // }
                    if (!crate_row.crates_io.is_empty()
                        && content_link.starts_with(&crate_row.crates_io))
                        || (!crate_row.docs_rs.is_empty()
                            && content_link.starts_with(&crate_row.docs_rs))
                        || (!crate_row.repository.is_empty()
                            && content_link.starts_with(&crate_row.repository))
                        || (!crate_row.homepage.is_empty()
                            && content_link.starts_with(&crate_row.homepage))
                    {
                        let link_row = Link {
                            date: html_page.date.to_string(),
                            title: html_page.title.to_string(),
                            link: html_page.link.to_string(),
                        };
                        crate_row.links.push(link_row);
                        break;
                    }
                }
            }
        });

    println!("Finished matching");

    println!("Starts writing");

    write_crates_to_json(&crates)?;

    println!("Finished writing");

    Ok(())
}

fn output_related_articles() -> Result<(), Box<dyn Error>> {
    let file = fs::File::open("crates.json")?;
    let mut crates: Vec<Crate> = serde_json::from_reader(file)?;
    let mut num_crates_with_links = 0;
    let num_crates = crates.len();

    for (i, crate_row) in crates.iter_mut().enumerate() {
        let n = i + 1;
        println!("Stage.1 [{n} / {num_crates}] {}", crate_row.name);

        for link in crate_row.links.iter_mut() {
            if link.title.is_empty() {
                link.title = link.link.to_string();
            }
        }
    }

    let mut all_links = Vec::new();

    for (i, crate_row) in crates.iter().enumerate() {
        let n = i + 1;
        println!("Stage.2 [{n} / {num_crates}] {}", crate_row.name);
        let file_name = format!(
            "../../rustacean.info/related-articles/{}.json",
            crate_row.name
        );

        let mut links: Vec<Link> = if let Ok(file) = fs::File::open(&file_name) {
            serde_json::from_reader(file)?
        } else {
            Vec::new()
        };

        links.extend(crate_row.links.clone());
        links = links
            .into_iter()
            .sorted_by_key(|link| link.link.clone())
            .dedup_by(|a, b| a.link == b.link)
            .sorted_by_key(|link| link.date.clone())
            .rev()
            .collect();

        if !links.is_empty() {
            let json_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_name)?;

            serde_json::to_writer_pretty(json_file, &links)?;
            num_crates_with_links += 1;
        } else {
            let _ = fs::remove_file(file_name);
        }

        all_links.extend(links.into_iter().map(|link| LinkCrate {
            crate_name: crate_row.name.to_string(),
            date: link.date,
            title: link.title,
            link: link.link,
        }));
    }

    crates.sort_by_key(|crate_row| crate_row.links.len());

    for crate_row in crates.iter() {
        if !crate_row.links.is_empty() {
            println!(
                "Found {:3} Related Articles for https://rustacean.info/crates/{}",
                crate_row.links.len(),
                crate_row.name
            );
        }
    }

    dbg!(&num_crates);
    dbg!(&num_crates_with_links);
    dbg!(&all_links.len());

    all_links = all_links
        .into_iter()
        .sorted_by_key(|link| link.link.clone())
        .dedup_by(|a, b| a.link == b.link)
        .sorted_by_key(|link| link.date.clone())
        .rev()
        .collect();

    let json_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../../rustacean.info/related-articles/_links.json")?;
    serde_json::to_writer_pretty(json_file, &all_links)?;

    let json_file_minify = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("../../rustacean.info/related-articles/_links.min.json")?;
    serde_json::to_writer(json_file_minify, &all_links)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // gen_sitemap()?;
    // get_crates()?;
    // curl_twir_links().await?;
    consolidate_crates_json()?;
    output_related_articles()?;

    Ok(())
}

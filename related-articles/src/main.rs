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
    links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Link {
    title: String,
    link: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HtmlPage {
    title: String,
    link: String,
    content_links: Vec<String>,
}

fn get_crates() -> Result<Vec<Crate>, Box<dyn Error>> {
    let file = fs::File::open("crates.csv")?;
    let rdr = csv::Reader::from_reader(file);
    let mut iter = rdr.into_deserialize();

    let mut crates = Vec::new();

    while let Some(line) = iter.next() {
        let mut row: Crate = line?;
        row.crates_io = format!("https://crates.io/crates/{}", row.name);
        row.repository = row.repository.trim_start_matches("https://").into();
        row.repository = row.repository.trim_start_matches("http://").into();
        row.crates_io = row.crates_io.trim_start_matches("https://").into();
        row.crates_io = row.crates_io.trim_start_matches("http://").into();
        row.homepage = row.homepage.trim_start_matches("https://").into();
        row.homepage = row.homepage.trim_start_matches("http://").into();
        row.repository = row.repository.trim().into();
        row.crates_io = row.crates_io.trim().into();
        row.homepage = row.homepage.trim().into();
        let filters = [
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
        ];
        for filter in filters {
            if row.repository == filter {
                row.repository = "".into();
            }
            if row.crates_io == filter {
                row.crates_io = "".into();
            }
            if row.homepage == filter {
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
            .timeout(time::Duration::from_secs(30))
            .send()
            .await;
        let html = match response {
            Ok(res) => match res.text().await {
                Ok(text) => text,
                Err(err) => {
                    dbg!(&err);
                    continue;
                }
            },
            Err(err) => {
                dbg!(&err);
                continue;
            }
        };

        let mut html_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
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
            println!("[{n} / {num_paths}] {:?}", path);

            let mut file = fs::File::open(path.path()).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let title = get_html_title(&contents).unwrap().unwrap_or_default();

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
                .filter(|link| link.starts_with("http"))
                .collect();

            HtmlPage {
                title,
                link,
                content_links,
            }
        })
        .collect();

    let num_crates = crates.len();

    crates
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, crate_row)| {
            let n = i + 1;
            println!("[{n} / {num_crates}] {}", crate_row.name);

            for html_page in html_pages.iter() {
                for content_link in html_page.content_links.iter() {
                    if (!crate_row.crates_io.is_empty()
                        && content_link.ends_with(&crate_row.crates_io))
                        || (!crate_row.repository.is_empty()
                            && content_link.ends_with(&crate_row.repository))
                        || (!crate_row.homepage.is_empty()
                            && content_link.ends_with(&crate_row.homepage))
                    {
                        let link_row = Link {
                            title: html_page.title.to_string(),
                            link: html_page.link.to_string(),
                        };
                        crate_row.links.push(link_row);
                        break;
                    }
                }
            }
        });

    write_crates_to_json(&crates)?;
    Ok(())
}

fn output_related_articles() -> Result<(), Box<dyn Error>> {
    let file = fs::File::open("crates.json")?;
    let mut crates: Vec<Crate> = serde_json::from_reader(file)?;
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

    for (i, crate_row) in crates.iter().enumerate() {
        let n = i + 1;
        println!("Stage.2 [{n} / {num_crates}] {}", crate_row.name);

        if !crate_row.links.is_empty() {
            let json_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(format!(
                    "../public/related-articles/{}.json",
                    crate_row.name
                ))?;
            serde_json::to_writer_pretty(json_file, &crate_row.links)?;
        }
    }

    let mut all_links = Vec::new();
    for (i, crate_row) in crates.into_iter().enumerate() {
        let n = i + 1;
        println!("Stage.3 [{n} / {num_crates}] {}", crate_row.name);

        all_links.extend(crate_row.links);
    }
    let json_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("../public/related-articles.json")?;
    serde_json::to_writer_pretty(json_file, &all_links)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // get_crates()?;
    // curl_twir_links()?;
    // consolidate_crates_json()?;
    // output_related_articles()?;

    Ok(())
}

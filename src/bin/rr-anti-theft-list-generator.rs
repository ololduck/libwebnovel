use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use libwebnovel::{Backend, Backends, Chapter};
use log::{info, LevelFilter};

static KNOWN_PHRASES_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(env!["CARGO_MANIFEST_DIR"])
        .join("ressources")
        .join("royalroad")
        .join("known_anti-theft_sentences.txt")
});

const URL: &str = "https://www.royalroad.com/fiction/21220/mother-of-learning/";

fn load_known_sentences() -> Vec<String> {
    let mut f = File::open(KNOWN_PHRASES_PATH.as_path()).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
fn save_known_sentences(sentences: &[String]) {
    let mut file = File::create(KNOWN_PHRASES_PATH.as_path()).unwrap();
    file.write_all(sentences.join("\n").as_bytes()).unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::formatted_timed_builder()
        .filter(None, LevelFilter::Info)
        .init();
    let backend = Backends::new(URL)?;
    let mut diffs = load_known_sentences();
    let chapters = (0..100)
        .map(|i| {
            info!("Iteration {i}");
            backend.get_chapter(1).unwrap()
        })
        .collect::<Vec<Chapter>>();
    let reference_chapter = &chapters[0];
    for chapter in &chapters[1..] {
        for diff in diff::lines(&reference_chapter.to_string(), &chapter.to_string()) {
            if let diff::Result::Right(r) = diff {
                if !r.starts_with("<p>") {
                    continue;
                }
                let text = r
                    .strip_prefix("<p>")
                    .unwrap()
                    .strip_suffix("</p>")
                    .unwrap()
                    .to_string();
                info!("found line \"{text}\"");
                diffs.push(text)
            }
        }
    }
    diffs.sort();
    diffs.dedup();
    save_known_sentences(&diffs);
    info!("Found {} unique warnings", diffs.len());
    println!(
        "const ROYALROAD_ANTI_THEFT_TEXT: &[&str] = &[\n{}\n];",
        diffs
            .iter()
            .map(|s| format!(r#"    "{}""#, s))
            .collect::<Vec<String>>()
            .join(",\n")
    );
    Ok(())
}

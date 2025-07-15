use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    url: String,

    #[arg(short, long)]
    output: Option<String>,
}

fn get_filename_from_url(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let path_segments = url.path_segments()?;
    let filename = path_segments.last()?;
    if filename.is_empty() {
        None
    } else {
        Some(filename.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let output: String;
    let url = args.url;
    let filename: String;
    if let Some(some_filename) = get_filename_from_url(&url) {
        filename = some_filename
    } else {
        filename = String::from("downloaded")
    }
    if let Some(out) = args.output {
        output = out
    } else {
        output = filename
    }
    let response = match Client::new().get(&url).send().await {
        Ok(resp) => {
            info!("Success responce: {}", resp.status());
            resp
        }
        Err(e) => {
            error!("Error reqwest: {}", e);
            return Err(e.into());
        }
    };
    let file_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"),
    );

    let mut file = tokio::fs::File::create(&output).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    pb.finish();
    println!("{}: {}", "Succeful downloaded to".green().bold(), &output);
    info!("Succeful downloaded to: {}", &output);

    Ok(())
}

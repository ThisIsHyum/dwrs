use std::path::PathBuf;
// use std::collections::HashMap;
use std::sync::Arc;

use clap::Parser;
use colored::*;
//use futures::io::BufReader;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{MultiProgress,ProgressBar, ProgressStyle};
use log::{error, info, warn};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::sync::Semaphore;
use tokio::task;
// use url::Url;

#[derive(Parser)]
#[command(name = "dwrs",author, version, about = "Parallel downloader with progress bar")]
#[command(group(clap::ArgGroup::new("input").required(true).args(&["url","file"])))]
struct Args {

    #[arg(required = false)]
    url: Vec<String>,

    #[arg(short, long)]
    output: Vec<String>,

    #[arg(short, long, default_value = "1")]
    jobs: usize,
    #[arg(short,long)]
    file: Option<PathBuf>,
}

// fn get_filename_from_url(url_str: &str) -> Option<String> {
//     let url = Url::parse(url_str).ok()?;
//     let path_segments = url.path_segments()?;
//     let filename = path_segments.last()?;
//     if filename.is_empty() {
//         None
//     } else {
//         Some(filename.to_string())
//     }
// }

async fn parse_file(path: &PathBuf) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut pairs = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() == 2 {
            pairs.push((parts[0].to_string(),parts[1].to_string()));
        } else if parts.len() == 1 {
            let filename = parts[0].split('/').last().unwrap_or("file.bin").to_string();
            pairs.push((parts[0].to_string(), filename));
        } else {
            eprintln!("Wrong format of string: {}", line)
        }
    }
    Ok(pairs)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("logger init");

    let args = Args::parse();

    let mut url_output_pairs = Vec::new();

    if let Some(file_path) = args.file {
        url_output_pairs = parse_file(&file_path).await.unwrap_or_else(|e| {
            eprintln!("Error in reading file: {}", e);
            panic!();
        });
    } else {

        for (i,url) in args.url.iter().enumerate() {
            let output = if let Some(path) = args.output.get(i) {
                path.clone()
            } else {
                url.split('/').last().unwrap_or("file.bin").to_string()
            };
            url_output_pairs.push((url.clone(), output));
        }

        if !args.output.is_empty() && args.output.len() != args.url.len() {
            println!("{}",args.output.len());
            error!("Error count of output files dont equal of urls");
            eprintln!("{}","Error count of output files dont equal of urls".bold().red());
            panic!()
        }


    }




    let client = Client::new();
    let mp = Arc::new(MultiProgress::new());
    let semaphore = Arc::new(Semaphore::new(args.jobs));

    let mut tasks = FuturesUnordered::new();

    for (url, output) in url_output_pairs.into_iter() {
        let output = PathBuf::from(output);
        // let output = if let Some(path) = args.output.get(i) {
        //     PathBuf::from(path)
        // } else {
        //     PathBuf::from(get_filename_from_url(&url).unwrap())
        // };

        let client = client.clone();
        let mp = mp.clone();
        let sem = semaphore.clone();
        let url = url.clone();

        tasks.push(task::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({percent}%) {msg}").unwrap()
            );

            pb.set_message(format!("Download {} -> {:?}",url, output));

            match download_file(&client, &url , &output, &pb).await {
                Ok(_) => pb.finish_with_message(format!("{}: {:?}","Success dowloaded".green().bold(),output)),
                Err(e) => pb.finish_with_message(format!("{}: {:?}: {}","Error download".red().bold(),output,e)),
            }
        }));
    }

    while let Some(_) = tasks.next().await {}

}



async fn download_file(
    client: &Client,
    url: &str,
    output: &PathBuf,
    pb: &ProgressBar,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    let file_size = response.content_length().unwrap_or(0);


    pb.set_length(file_size);
    let mut file = tokio::fs::File::create(&output).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }


    Ok(())

}





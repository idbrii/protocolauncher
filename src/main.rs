use clap::Parser;
use std::collections::HashMap;
use std::process::Command;
use url::Url;

/// Establish a protocol for viewing svn logs, parse the url containing the revision and path, and
/// launch Tortoise to view logs at that revision.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Url to parse
    #[arg(short, long)]
    url: String,
}

fn query_as_map(u: Url) -> HashMap<String, String> {
    u.query_pairs().into_owned().collect()
}

fn main() {
    let args = Args::parse();

    let url = match Url::parse(args.url.as_str()) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Error parsing URL: {}", e);
            return;
        }
    };

    match url.scheme() {
        "viewsvn" => {}
        _ => {
            eprintln!("Unsupported protocol: {}", url.scheme());
            return;
        }
    };

    let query = query_as_map(url);

    let server_url = query
        .get("server_url")
        .expect("No server_url specified in url.");

    // Validate the url, but we'll just use the string.
    Url::parse(server_url).expect("Invalid server_url");

    let revision_str = query
        .get("revision")
        .expect("No revision specified in url.");
    let revision = revision_str
        .parse::<u32>()
        .expect("Url revision was not a number");

    println!("/startrev:{revision}");

    // TortoiseProc /command:log /startrev:$revision /path:https://svn.corp.ca/svn/corp_repository
    let output = Command::new("TortoiseProc.exe")
        .arg("/command:log")
        .arg(format!("/startrev:{revision}"))
        .arg(format!("/path:{server_url}"))
        .output()
        .expect("Failed to execute command.");
    println!(
        "Tortoise output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

use clap::Parser;
use std::collections::HashMap;
use std::process::Command;
use url::Url;

const HANDLED_PROTOCOL: &str = "viewsvn";

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

pub trait UnwrapExt<T> {
    fn unwrap_or_error(self, msg: &str) -> T;
}

impl<T, E: std::fmt::Display> UnwrapExt<T> for Result<T, E> {
    fn unwrap_or_error(self, msg: &str) -> T {
        self.unwrap_or_else(|err| {
            eprintln!("{msg}: {}", { err });
            std::process::exit(1);
        })
    }
}

impl<T> UnwrapExt<T> for Option<T> {
    fn unwrap_or_error(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("{msg}");
            std::process::exit(1);
        })
    }
}

fn view_log(args: Args) {
    let url = Url::parse(args.url.as_str()).unwrap_or_error("Error parsing URL");

    match url.scheme() {
        HANDLED_PROTOCOL => {}
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
    Url::parse(server_url).unwrap_or_error("Invalid server_url");

    let revision_str = query
        .get("revision")
        .unwrap_or_error("No revision specified in url.");
    let revision = revision_str
        .parse::<u32>()
        .unwrap_or_error("Url revision was not a number");

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

fn main() {
    view_log(Args::parse());
    // TODO:
    //https://stackoverflow.com/questions/389204/how-do-i-create-my-own-url-protocol-e-g-so
}

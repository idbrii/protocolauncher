use clap::Parser;
use directories::ProjectDirs;
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::{error, info, trace};
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use url::Url;
use winreg::enums::*;
use winreg::RegKey;

const HANDLED_PROTOCOL: &str = "viewsvn";

/// Establish a protocol for viewing svn logs, parse the url containing the revision and path, and
/// launch Tortoise to view logs at that revision.
///
/// When launched with no arguments, registers this program as a handler for our protocol.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Url to parse
    #[arg(short, long)]
    url: Option<String>,
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
            error!("{msg}: {}", { err });
            std::process::exit(1);
        })
    }
}

impl<T> UnwrapExt<T> for Option<T> {
    fn unwrap_or_error(self, msg: &str) -> T {
        self.unwrap_or_else(|| {
            error!("{msg}");
            std::process::exit(1);
        })
    }
}

pub fn log_key(_key: &RegKey, disp: &RegDisposition) {
    match disp {
        REG_CREATED_NEW_KEY => info!("A new key has been created"),
        REG_OPENED_EXISTING_KEY => info!("An existing key has been opened"),
    }
}

fn register_handler() -> io::Result<()> {
    let exe_path = std::env::current_exe()?;

    // Open or create the registry key for URI handling
    let hkcr = RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT);

    let path = Path::new(HANDLED_PROTOCOL);
    let (root_key, disp) = hkcr.create_subkey(&path)?;
    log_key(&root_key, &disp);

    // Set the default value of the URI scheme key to the name of your application
    root_key.set_value("", &format!("URL:{HANDLED_PROTOCOL} Protocol"))?;
    root_key.set_value("URL Protocol", &"")?;

    let (shell_key, disp) = root_key.create_subkey("shell")?;
    log_key(&shell_key, &disp);

    // Create the shell verb.
    let (open_key, disp) = shell_key.create_subkey("open")?;
    log_key(&open_key, &disp);

    // What to do when that verb is executed.
    let (command_key, disp) = open_key.create_subkey("command")?;
    log_key(&command_key, &disp);

    // Set the default value of the "command" key to the command-line template for your application
    let command_template = format!("\"{}\" --url \"%1\"", exe_path.display());
    command_key.set_value("", &command_template)?;

    info!(
        "Successfully registered URI handler for '{}'",
        HANDLED_PROTOCOL
    );

    Ok(())
}

fn view_log(args: Args) {
    let url = Url::parse(
        args.url
            .unwrap_or_error("Didn't receive url argument.")
            .as_str(),
    )
    .unwrap_or_error("Error parsing URL");

    match url.scheme() {
        HANDLED_PROTOCOL => {}
        _ => {
            error!("Unsupported protocol: {}", url.scheme());
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
    trace!(
        "Tortoise output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    info!("Launched TortoiseProc.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(proj_dirs) = ProjectDirs::from("ca", "idbrii", "protolaunch") {
        let path = proj_dirs.config_dir().join("test.log");
        println!("Log path: {}", path.display());
        Logger::try_with_str("info")
            .unwrap_or_error("Failed to init log.")
            .log_to_file(FileSpec::try_from(path).unwrap())
            .duplicate_to_stderr(Duplicate::Trace)
            .start()
            .unwrap_or_error("Failed to open log file.");
    }

    info!("Argument count: {}", args.len());

    match args.len() {
        1 => register_handler().expect("Failed to register protocol."),
        _ => view_log(Args::parse()),
    }
}

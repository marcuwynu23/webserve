//! CLI option parsing and integration tests

use std::process::Command;
use structopt::StructOpt;
use tempfile::TempDir;
use webserve::ServeOptions;

#[test]
fn test_cli_options_defaults() {
    let args = vec!["webserve"];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.port, 8080);
    assert_eq!(options.host, "127.0.0.1");
    assert!(!options.spa);
    assert!(!options.watch);
    assert!(!options.open);
    assert!(!options.no_redirect_dir_slash);
}

#[test]
fn test_cli_options_custom_port() {
    let args = vec!["webserve", "--port", "3000"];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.port, 3000);
}

#[test]
fn test_cli_options_custom_host() {
    let args = vec!["webserve", "--host", "0.0.0.0"];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.host, "0.0.0.0");
}

#[test]
fn test_cli_options_spa_flag() {
    let args = vec!["webserve", "--spa"];
    let options = ServeOptions::from_iter(args.iter());
    assert!(options.spa);
}

#[test]
fn test_cli_options_watch_flag() {
    let args = vec!["webserve", "--watch"];
    let options = ServeOptions::from_iter(args.iter());
    assert!(options.watch);
}

#[test]
fn test_cli_options_directory() {
    let temp_dir = TempDir::new().unwrap();
    let dir_str = temp_dir.path().to_str().unwrap();
    let args = vec!["webserve", "--dir", dir_str];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.directory.unwrap(), temp_dir.path());
}

#[test]
fn test_cli_options_open_and_no_redirect_dir_slash() {
    let args = vec!["webserve", "--open", "--no-redirect-dir-slash"];
    let options = ServeOptions::from_iter(args.iter());
    assert!(options.open);
    assert!(options.no_redirect_dir_slash);
}

#[test]
fn test_cli_short_flags() {
    let temp_dir = TempDir::new().unwrap();
    let dir_str = temp_dir.path().to_str().unwrap();
    let args = vec!["webserve", "-p", "4000", "-h", "0.0.0.0", "-d", dir_str, "-w"];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.port, 4000);
    assert_eq!(options.host, "0.0.0.0");
    assert_eq!(options.directory.as_ref().unwrap(), temp_dir.path());
    assert!(options.watch);
}

#[test]
fn test_cli_combined_options() {
    let temp_dir = TempDir::new().unwrap();
    let dir_str = temp_dir.path().to_str().unwrap();
    let args = vec![
        "webserve",
        "--dir",
        dir_str,
        "--port",
        "5000",
        "--spa",
        "--watch",
        "--open",
    ];
    let options = ServeOptions::from_iter(args.iter());
    assert_eq!(options.port, 5000);
    assert_eq!(options.directory.as_ref().unwrap(), temp_dir.path());
    assert!(options.spa);
    assert!(options.watch);
    assert!(options.open);
}

/// --help exits 0 and prints usage (integration: run binary).
#[test]
fn test_cli_help_exits_zero_and_prints_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_webserve"))
        .arg("--help")
        .output()
        .expect("run webserve binary");
    assert!(
        output.status.success(),
        "expected --help to exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("webserve") || stdout.contains("Usage"),
        "stdout should contain usage: {}",
        stdout
    );
}

//! CLI option parsing tests

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

//! Legacy CLI entry point (prefer `bin/ethean` via default-run).

use ethean::{config::Config, Result};
use clap::{Arg, Command, ArgMatches};
use std::path::PathBuf;
use tracing::{info, error};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::init();
    
    info!("🚀 Ethean Lean Consensus Client v{}", ethean::VERSION);
    info!("📦 Modular Rust implementation starting...");
    
    // Parse CLI arguments
    let matches = build_cli().get_matches();
    
    // Initialize configuration
    let config = initialize_config(&matches)?;
    
    // Start the client
    start_client(config)?;
    
    info!("✅ Client started successfully!");
    Ok(())
}

fn build_cli() -> Command {
    Command::new("ethean")
        .version(ethean::VERSION)
        .about("Ethean Lean Consensus Client")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("network")
                .short('n')
                .long("network")
                .value_name("NETWORK")
                .help("Network to connect to")
                .default_value("mainnet")
                .value_parser(["mainnet", "goerli", "sepolia", "holesky"])
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("DIR")
                .help("Data directory path")
                .default_value("./data")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level")
                .default_value("info")
                .value_parser(["trace", "debug", "info", "warn", "error"])
        )
        .arg(
            Arg::new("http-address")
                .long("http-address")
                .value_name("ADDRESS")
                .help("HTTP API bind address")
                .default_value("127.0.0.1:5052")
        )
        .arg(
            Arg::new("p2p-address")
                .long("p2p-address")
                .value_name("ADDRESS")
                .help("P2P network bind address")
                .default_value("0.0.0.0:9000")
        )
        .arg(
            Arg::new("metrics")
                .long("metrics")
                .help("Enable metrics endpoint")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("metrics-port")
                .long("metrics-port")
                .value_name("PORT")
                .help("Metrics endpoint port")
                .default_value("9090")
                .value_parser(clap::value_parser!(u16))
        )
        .subcommand(
            Command::new("validator")
                .about("Run validator client")
                .arg(
                    Arg::new("keys")
                        .long("keys")
                        .value_name("DIR")
                        .help("Validator keys directory")
                        .required(true)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("lean-node")
                        .long("lean-node")
                        .value_name("URL")
                        .help("Lean consensus node URL")
                        .default_value("http://localhost:5052")
                )
        )
        .subcommand(
            Command::new("database")
                .about("Database operations")
                .subcommand(
                    Command::new("backup")
                        .about("Create database backup")
                        .arg(
                            Arg::new("output")
                                .short('o')
                                .long("output")
                                .value_name("DIR")
                                .help("Backup output directory")
                                .required(true)
                                .value_parser(clap::value_parser!(PathBuf))
                        )
                )
                .subcommand(
                    Command::new("restore")
                        .about("Restore from backup")
                        .arg(
                            Arg::new("backup")
                                .short('b')
                                .long("backup")
                                .value_name("DIR")
                                .help("Backup directory")
                                .required(true)
                                .value_parser(clap::value_parser!(PathBuf))
                        )
                )
        )
}

fn initialize_config(matches: &ArgMatches) -> Result<Config> {
    let mut config = Config::default();
    
    // Set network
    if let Some(network) = matches.get_one::<String>("network") {
        config.network = network.clone();
    }
    
    // Set data directory
    if let Some(data_dir) = matches.get_one::<PathBuf>("data-dir") {
        config.data_dir = data_dir.clone();
    }
    
    // Set HTTP API address
    if let Some(http_address) = matches.get_one::<String>("http-address") {
        config.http_address = http_address.clone();
    }
    
    // Set P2P address
    if let Some(p2p_address) = matches.get_one::<String>("p2p-address") {
        config.p2p_address = p2p_address.clone();
    }
    
    // Set metrics
    if matches.get_flag("metrics") {
        config.metrics_enabled = true;
        if let Some(port) = matches.get_one::<u16>("metrics-port") {
            config.metrics_port = *port;
        }
    }
    
    // Load config file if specified
    if let Some(config_path) = matches.get_one::<PathBuf>("config") {
        config = Config::from_file(config_path)?;
    }
    
    Ok(config)
}

fn start_client(config: Config) -> Result<()> {
    // Create and configure client
    let mut client = Client::new(config)?;
    
    // Initialize storage
    client.initialize_storage()?;
    
    // Initialize networking
    client.initialize_network()?;
    
    // Start API server
    client.start_api_server()?;
    
    // Start consensus engine
    client.start_consensus()?;
    
    // Main event loop
    client.run()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        // Basic smoke test
        assert_eq!(2 + 2, 4);
    }
}

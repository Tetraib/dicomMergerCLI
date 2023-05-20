use reqwest::Error;
use std::fs;
use std::io::{self, Write};
use toml::Value;

struct Config {
    server_address: String,
    username: String,
    password: String,
}

impl Config {
    fn from_file(path: &str) -> Result<Config, String> {
        let config_data = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let config_value = config_data
            .parse::<Value>()
            .map_err(|err| err.to_string())?;
        let config_table = config_value
            .as_table()
            .ok_or_else(|| "Invalid TOML configuration".to_string())?;

        let server_address = config_table
            .get("server_address")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing or invalid 'server_address' in config".to_string())?
            .to_string();

        let username = config_table
            .get("username")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing or invalid 'username' in config".to_string())?
            .to_string();

        let password = config_table
            .get("password")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing or invalid 'password' in config".to_string())?
            .to_string();

        Ok(Config {
            server_address,
            username,
            password,
        })
    }
}

async fn merge_series_into_study(
    config: &Config,
    study_id: &str,
    series_id: &str,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/studies/{}/merge", config.server_address, study_id);
    let body = format!("{{\"Resources\":[\"{}\"]}}", series_id);

    let response = client
        .post(&url)
        .basic_auth(&config.username, Some(&config.password))
        .body(body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Series merged successfully!");
    } else {
        println!("Failed to merge series. Status: {}", response.status());
    }

    Ok(())
}

fn prompt_user(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    let config_path = "config.toml";
    let config = Config::from_file(config_path).expect("Failed to read config file");

    let study_id = prompt_user("Enter the study ID where the series will be merged into: ");
    let series_id = prompt_user("Enter the series ID to be merged: ");

    if let Err(err) = merge_series_into_study(&config, &study_id, &series_id).await {
        eprintln!("Error: {}", err);
    }
}

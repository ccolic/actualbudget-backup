use std::fs;

use clap::Parser;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActualBudgetError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Budget file not found with sync ID: {0}")]
    BudgetFileNotFound(String),
    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },
}

type Result<T> = std::result::Result<T, ActualBudgetError>;

#[derive(Serialize)]
struct LoginRequest<'a> {
    #[serde(rename = "loginMethod")]
    login_method: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct LoginResponse {
    data: LoginData,
}

#[derive(Deserialize)]
struct LoginData {
    token: String,
}

#[derive(Deserialize)]
struct FileListResponse {
    data: Vec<FileInfo>,
}

#[derive(Deserialize)]
struct FileInfo {
    #[serde(rename = "groupId")]
    group_id: String,
    #[serde(rename = "fileId")]
    file_id: String,
}

pub struct ActualBudgetAPIClient {
    http_client: Client,
    server_url: String,
    budget_sync_id: String,
    password: String,
    token: Option<String>,
    budget_file_id: Option<String>,
}

impl ActualBudgetAPIClient {
    /// Creates a new instance of `ActualBudgetAPIClient`.
    pub fn new(server_url: String, budget_sync_id: String, password: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(ActualBudgetAPIClient {
            http_client,
            server_url,
            budget_sync_id,
            password,
            token: None,
            budget_file_id: None,
        })
    }

    /// Checks if the given server URL is reachable.
    pub fn check_connectivity(&self) -> Result<()> {
        let response = self.http_client.head(&self.server_url).send()?;

        if response.status().is_success() {
            println!("✓ Server '{}' is reachable", &self.server_url);
            Ok(())
        } else {
            let status = response.status();
            let message = format!("Server returned status: {}", status);
            Err(ActualBudgetError::ServerError {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Authenticates the user with the given password and retrieves the authentication token.
    pub fn authenticate(&mut self) -> Result<()> {
        let login_request = LoginRequest {
            login_method: "password",
            password: &self.password,
        };

        let login_url = format!("{}/account/login", &self.server_url);
        let response = self
            .http_client
            .post(&login_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&login_request)
            .send()?;

        if !response.status().is_success() {
            return Err(ActualBudgetError::AuthenticationFailed);
        }

        let login_response: LoginResponse = response.json()?;
        self.token = Some(login_response.data.token);

        println!("✓ Authentication successful!");
        Ok(())
    }

    /// Retrieves the file ID of the budget file.
    pub fn get_file_id(&mut self) -> Result<()> {
        let token = self
            .token
            .as_ref()
            .ok_or(ActualBudgetError::AuthenticationFailed)?;

        let list_file_url = format!("{}/sync/list-user-files", &self.server_url);

        let mut headers = HeaderMap::new();
        headers.insert("X-ACTUAL-TOKEN", HeaderValue::from_str(token)?);

        let response = self
            .http_client
            .get(&list_file_url)
            .headers(headers)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let message = format!("Failed to fetch file list: {}", status);
            return Err(ActualBudgetError::ServerError {
                status: status.as_u16(),
                message,
            });
        }

        let file_list: FileListResponse = response.json()?;

        for file_info in file_list.data {
            if file_info.group_id == self.budget_sync_id {
                self.budget_file_id = Some(file_info.file_id.clone());
                println!("✓ Found budget file ID: {}", &file_info.file_id);
                return Ok(());
            }
        }

        Err(ActualBudgetError::BudgetFileNotFound(
            self.budget_sync_id.clone(),
        ))
    }

    /// Downloads the budget file and writes it to the current directory.
    /// Returns the filename of the downloaded file.
    pub fn download_file(&self) -> Result<String> {
        let token = self
            .token
            .as_ref()
            .ok_or(ActualBudgetError::AuthenticationFailed)?;
        let file_id = self
            .budget_file_id
            .as_ref()
            .ok_or(ActualBudgetError::BudgetFileNotFound(
                self.budget_sync_id.clone(),
            ))?;

        let filename = format!(
            "actualbudget_backup_{}.zip",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        );

        let download_url = format!("{}/sync/download-user-file", &self.server_url);

        let mut headers = HeaderMap::new();
        headers.insert("X-ACTUAL-TOKEN", HeaderValue::from_str(token)?);
        headers.insert("X-ACTUAL-FILE-ID", HeaderValue::from_str(file_id)?);

        println!("📥 Downloading budget file...");
        let response = self
            .http_client
            .get(&download_url)
            .headers(headers)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let message = format!("Failed to download file: {}", status);
            return Err(ActualBudgetError::ServerError {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.bytes()?;
        fs::write(&filename, &body)?;

        println!("✓ File downloaded successfully! Saved as: {}", &filename);
        Ok(filename)
    }

    /// Convenience method to perform the complete backup process.
    pub fn backup(&mut self) -> Result<String> {
        self.check_connectivity()?;
        self.authenticate()?;
        self.get_file_id()?;
        self.download_file()
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Actual Budget API Client - Download budget backups", long_about = None)]
struct Args {
    /// Server URL for the Actual Budget instance
    #[arg(short, long, env = "SERVER_URL")]
    server_url: String,

    /// Budget sync ID from your Actual Budget settings
    #[arg(short, long, env = "BUDGET_SYNC_ID")]
    budget_sync_id: String,

    /// Password for authentication (recommended to use PASSWORD env var)
    #[arg(env = "PASSWORD", hide = true)]
    password: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🚀 Starting Actual Budget backup...");

    let mut client =
        ActualBudgetAPIClient::new(args.server_url, args.budget_sync_id, args.password)?;

    match client.backup() {
        Ok(filename) => {
            println!("🎉 Backup completed successfully!");
            println!("📁 File saved as: {}", filename);
        }
        Err(e) => {
            eprintln!("❌ Backup failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

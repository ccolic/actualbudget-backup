use std::collections::HashMap;

use clap::Parser;
use reqwest::{
    self,
    header::{HeaderMap, CONTENT_TYPE},
};

struct ActualBudgetAPIClient {
    http_client: reqwest::blocking::Client,
    server_url: String,
    budget_sync_id: String,
    budget_file_id: String,
    password: String,
    token: String,
}

impl ActualBudgetAPIClient {
    /// Creates a new instance of `ActualBudgetAPIClient`.
    ///
    /// # Arguments
    ///
    /// * `server_url` - The URL of the Actual Budget server.
    /// * `budget_sync_id` - Your budget Sync ID.
    /// * `password` - The password for authentication.
    ///
    /// # Returns
    ///
    /// A new instance of `ActualBudgetAPIClient`.
    fn new(server_url: String, budget_sync_id: String, password: String) -> ActualBudgetAPIClient {
        ActualBudgetAPIClient {
            http_client: reqwest::blocking::Client::new(),
            token: String::new(),
            budget_file_id: String::new(),
            server_url,
            budget_sync_id,
            password,
        }
    }

    /// Checks if the given server URL is reachable.
    ///
    /// # Panics
    ///
    /// Panics if the server is not reachable.
    fn check_connectivity(&self) {
        let response = self.http_client.head(&self.server_url).send();

        match response {
            Ok(_) => println!("Server '{}' is reachable", &self.server_url),
            Err(e) => panic!("Cannot reach server. Error: {}", e),
        }
    }

    /// Authenticates the user with the given password. Retrieves the authentication token.
    ///
    /// # Panics
    ///
    /// Panics if the authentication fails.
    fn authenticate(&mut self) {
        let post_data: HashMap<&str, &str> =
            HashMap::from([("loginMethod", "password"), ("password", &self.password)]);

        let login_url: String = format!("{}/account/login", &self.server_url);
        let request = self
            .http_client
            .post(login_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&post_data);
        let response = request
            .send()
            .unwrap_or_else(|e| panic!("Error while sending authentication request: {}", e))
            .error_for_status()
            .unwrap_or_else(|e| panic!("Authentication error: {}", e));

        let json: serde_json::Value = response.json().unwrap();
        self.token = json["data"]["token"].as_str().unwrap().to_string();

        println!("Authn successful!");
    }

    /// Retrieves the file ID of the budget file.
    ///
    /// # Panics
    ///
    /// Panics if the file ID cannot be retrieved.
    fn get_file_id(&mut self) {
        let list_file_url: String = format!("{}/sync/list-user-files", &self.server_url);

        let mut headers = HeaderMap::new();
        headers.insert("X-ACTUAL-TOKEN", self.token.parse().unwrap());

        let request = self.http_client.get(list_file_url).headers(headers);

        let response = request
            .send()
            .unwrap_or_else(|e| panic!("Error fetching file ID: {}", e))
            .error_for_status()
            .unwrap_or_else(|e| panic!("Error fetching file ID: {}", e));

        let json: serde_json::Value = response.json().unwrap();

        let data_array = json["data"].as_array().unwrap();

        for item in data_array {
            let group_id = item["groupId"].as_str().unwrap();
            if group_id == self.budget_sync_id {
                self.budget_file_id = item["fileId"].as_str().unwrap().to_string();
                println!("File ID: {}", self.budget_file_id);
                break;
            }
        }
    }

    /// Downloads the budget file and writes it to the current directory with the name `actualbudget_backup_<YYYY-MM-DD_hh-mm-ss>.zip`.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be downloaded or written.
    fn download_file(&self) {
        let filename: String = format!(
            "actualbudget_backup_{}.zip",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        );

        let download_url: String = format!("{}/sync/download-user-file", &self.server_url);

        let mut headers = HeaderMap::new();
        headers.insert("X-ACTUAL-TOKEN", self.token.parse().unwrap());
        headers.insert("X-ACTUAL-FILE-ID", self.budget_file_id.parse().unwrap());

        let request = self.http_client.get(download_url).headers(headers);

        let response = request
            .send()
            .unwrap_or_else(|e| panic!("Error downloading file: {}", e))
            .error_for_status()
            .unwrap_or_else(|e| panic!("Error downloading file: {}", e));

        let body = response.bytes().unwrap();

        // Get the content of the response
        std::fs::write(&filename, &body).unwrap_or_else(|e| panic!("Error writing file: {}", e));
        println!("File downloaded successfully! Written to {}", &filename);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // server URL
    #[arg(short, long, env)]
    server_url: String,

    // budget sync ID
    #[arg(short, long, env)]
    budget_sync_id: String,

    // password from env only
    #[arg(env)]
    password: String, // PASSWORD env var
}

fn main() {
    // parse command line arguments
    let args = Args::parse();

    let mut c = ActualBudgetAPIClient::new(args.server_url, args.budget_sync_id, args.password);
    c.check_connectivity();
    c.authenticate();
    c.get_file_id();
    c.download_file();
}

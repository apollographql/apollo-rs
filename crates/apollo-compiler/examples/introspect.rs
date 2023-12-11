use apollo_compiler::introspection_client::request;
use apollo_compiler::introspection_client::Response;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

fn main() {
    let url = std::env::args().nth(1).unwrap();
    let child = Command::new("curl")
        .args(["--header", "Content-Type: application/json"])
        .args(["--data-binary", "@-"])
        .arg(url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child_stdin = child.stdin.unwrap();
    let mut child_stdout = child.stdout.unwrap();
    let mut response_body = String::new();
    child_stdin.write_all(request().as_bytes()).unwrap();
    child_stdin.flush().unwrap();
    drop(child_stdin);
    child_stdout.read_to_string(&mut response_body).unwrap();
    // println!("{}", &response_body[1140..][..100]);
    let response: Response = serde_json::from_str(&response_body).unwrap();
    for error in response.errors {
        println!("# {:?}", error)
    }
    if let Some(schema) = response.schema {
        println!("{}", schema)
    }
}

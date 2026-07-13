use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
};

use serde_json::{json, Value};

use super::{
    messages::LspMessage,
    protocol::path_to_uri,
};

#[derive(Debug)]
pub struct LspClient {
    stdin: ChildStdin,
    rx: Receiver<LspMessage>,
    next_id: u32,
    version: i32,
}

impl LspClient {
    pub fn new() -> std::io::Result<Self> {
        let mut child = Command::new("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);

            loop {
                let mut content_length = 0usize;

                loop {
                    let mut line = String::new();

                    if reader.read_line(&mut line).is_err() {
                        return;
                    }

                    if line == "\r\n" {
                        break;
                    }

                    if let Some(value) = line.strip_prefix("Content-Length:") {
                        content_length = value.trim().parse().unwrap_or(0);
                    }
                }

                let mut body = vec![0; content_length];

                if reader.read_exact(&mut body).is_err() {
                    return;
                }

                match serde_json::from_slice::<Value>(&body) {
                    Ok(value) => {
                        let _ = tx.send(LspMessage::Json(value));
                    }
                    Err(err) => {
                        crate::debug::log(format!("JSON parse error: {err}"));
                    }
                }
            }
        });

        Ok(Self {
            stdin,
            rx,
            next_id: 1,
            version: 1,
        })
    }

    fn send(&mut self, value: Value) -> std::io::Result<()> {
        let text = value.to_string();

        crate::debug::log(format!("{}", text));

        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            text.len(),
            text
        )?;

        self.stdin.flush()
    }

    pub fn initialize(&mut self) -> std::io::Result<()> {
        self.send(json!({
            "jsonrpc":"2.0",
            "id":self.next_id,
            "method":"initialize",
            "params":{
                "processId":std::process::id(),
                "rootUri":null,
                "workspaceFolders":null,
                "capabilities":{},
                "clientInfo":{
                    "name":"ziro"
                }
            }
        }))?;

        self.next_id += 1;

        Ok(())
    }

    pub fn initialized(&mut self) -> std::io::Result<()> {
        self.send(json!({
            "jsonrpc":"2.0",
            "method":"initialized",
            "params":{}
        }))
    }

    pub fn did_open(
        &mut self,
        path: &Path,
        text: &str,
    ) -> std::io::Result<()> {
        self.send(json!({
            "jsonrpc":"2.0",
            "method":"textDocument/didOpen",
            "params":{
                "textDocument":{
                    "uri":path_to_uri(path),
                    "languageId":"rust",
                    "version":self.version,
                    "text":text,
                }
            }
        }))
    }

    pub fn did_change(
        &mut self,
        path: &Path,
        text: &str,
    ) -> std::io::Result<()> {
        self.version += 1;

        self.send(json!({
            "jsonrpc":"2.0",
            "method":"textDocument/didChange",
            "params":{
                "textDocument":{
                    "uri":path_to_uri(path),
                    "version":self.version
                },
                "contentChanges":[
                    {
                        "text":text
                    }
                ]
            }
        }))
    }

    pub fn completion(
        &mut self,
        path: &std::path::Path,
        line: usize,
        character: usize,
    ) -> std::io::Result<()> {
        let uri = format!(
            "file://{}",
            path.canonicalize()?.display()
        );

        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            }
        });

        self.next_id += 1;

        self.send(request)
    }

    pub fn try_recv(&self) -> Option<LspMessage> {
        self.rx.try_recv().ok()
    }
}

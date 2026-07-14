use super::{
    messages::LspMessage,
    protocol::{CompletionItem, Diagnostic, Hover, path_to_uri},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

#[derive(Debug, Clone, Copy)]
enum Request {
    Initialize,
    Completion,
    Hover(usize, usize),
    SemanticTokens,
}
#[derive(Debug)]
pub struct LspClient {
    stdin: ChildStdin,
    rx: Receiver<LspMessage>,
    next_id: u32,
    version: i32,
    requests: Arc<Mutex<HashMap<u32, Request>>>,
    pub language_id: String,
}

impl LspClient {
    pub fn spawn(binary: &str, args: &[&str], language_id: &str) -> std::io::Result<Self> {
        let mut child = Command::new(binary)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let requests = Arc::new(Mutex::new(HashMap::new()));
        let pending = requests.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.strip_prefix("Content-Length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0; length];
                if reader.read_exact(&mut body).is_err() {
                    return;
                }
                let Ok(value) = serde_json::from_slice::<Value>(&body) else {
                    continue;
                };
                if value.get("method").and_then(Value::as_str)
                    == Some("textDocument/publishDiagnostics")
                {
                    let uri = value
                        .pointer("/params/uri")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let diagnostics = value
                        .pointer("/params/diagnostics")
                        .cloned()
                        .and_then(|v| serde_json::from_value::<Vec<Diagnostic>>(v).ok())
                        .unwrap_or_default();
                    let _ = tx.send(LspMessage::Diagnostics(uri, diagnostics));
                    continue;
                }
                let request = value
                    .get("id")
                    .and_then(Value::as_u64)
                    .and_then(|id| pending.lock().ok()?.remove(&(id as u32)));
                match (request, value.get("result").cloned()) {
                    (Some(Request::Initialize), result) => {
                        let legend = result
                            .as_ref()
                            .and_then(|value| {
                                value.pointer(
                                    "/capabilities/semanticTokensProvider/legend/tokenTypes",
                                )
                            })
                            .and_then(Value::as_array)
                            .map(|items| {
                                items
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .map(str::to_string)
                                    .collect()
                            })
                            .unwrap_or_default();
                        let _ = tx.send(LspMessage::Initialized(legend));
                    }
                    (Some(Request::Completion), Some(result)) => {
                        #[derive(Deserialize)]
                        struct List {
                            items: Vec<CompletionItem>,
                        }
                        let items = serde_json::from_value::<List>(result.clone())
                            .map(|v| v.items)
                            .or_else(|_| serde_json::from_value(result))
                            .unwrap_or_default();
                        let _ = tx.send(LspMessage::Completion(items));
                    }
                    (Some(Request::Hover(line, character)), result) => {
                        let _ = tx.send(LspMessage::Hover(
                            line,
                            character,
                            result.and_then(|v| serde_json::from_value::<Hover>(v).ok()),
                        ));
                    }
                    (Some(Request::SemanticTokens), result) => {
                        let data = result
                            .and_then(|value| value.get("data").cloned())
                            .and_then(|value| serde_json::from_value::<Vec<u32>>(value).ok())
                            .unwrap_or_default();
                        let _ = tx.send(LspMessage::SemanticTokens(data));
                    }
                    _ => {}
                }
            }
        });
        Ok(Self {
            stdin,
            rx,
            next_id: 1,
            version: 1,
            requests,
            language_id: language_id.to_string(),
        })
    }
    fn send(&mut self, value: Value) -> std::io::Result<()> {
        let text = value.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", text.len(), text)?;
        self.stdin.flush()
    }
    pub fn initialize(&mut self, root: &Path) -> std::io::Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        self.requests
            .lock()
            .unwrap()
            .insert(id, Request::Initialize);
        let uri = path_to_uri(root);
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"initialize","params":{"processId":std::process::id(),"rootUri":uri,"workspaceFolders":[{"uri":path_to_uri(root),"name":"workspace"}],"capabilities":{"textDocument":{"completion":{"completionItem":{"snippetSupport":true}},"semanticTokens":{"requests":{"full":true},"tokenTypes":[],"tokenModifiers":[],"formats":["relative"]}}}}}))
    }
    pub fn initialized(&mut self) -> std::io::Result<()> {
        self.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}))
    }
    pub fn did_open(&mut self, path: &Path, text: &str) -> std::io::Result<()> {
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":path_to_uri(path),"languageId":self.language_id,"version":self.version,"text":text}}}))
    }
    pub fn did_change(&mut self, path: &Path, text: &str) -> std::io::Result<()> {
        self.version += 1;
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":path_to_uri(path),"version":self.version},"contentChanges":[{"text":text}]}}))
    }
    pub fn completion(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
    ) -> std::io::Result<()> {
        self.request(
            Request::Completion,
            "textDocument/completion",
            path,
            line,
            character,
        )
    }
    pub fn hover(&mut self, path: &Path, line: usize, character: usize) -> std::io::Result<()> {
        self.request(
            Request::Hover(line, character),
            "textDocument/hover",
            path,
            line,
            character,
        )
    }
    pub fn semantic_tokens(&mut self, path: &Path) -> std::io::Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        self.requests
            .lock()
            .unwrap()
            .insert(id, Request::SemanticTokens);
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":path_to_uri(path)}}}))
    }
    fn request(
        &mut self,
        request: Request,
        method: &str,
        path: &Path,
        line: usize,
        character: usize,
    ) -> std::io::Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        self.requests.lock().unwrap().insert(id, request);
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":{"textDocument":{"uri":path_to_uri(path)},"position":{"line":line,"character":character}}}))
    }
    pub fn try_recv(&self) -> Option<LspMessage> {
        self.rx.try_recv().ok()
    }
    pub fn shutdown(&mut self) -> std::io::Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown","params":null}))?;
        self.send(json!({"jsonrpc":"2.0","method":"exit","params":null}))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LspStatus {
    Initializing,
    Alive,
    Offline,
}

#[derive(Debug)]
pub struct LspSession {
    pub client: LspClient,
    pub status: LspStatus,
    pub semantic_legend: Vec<String>,
}

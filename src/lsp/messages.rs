use serde_json::Value;

use super::protocol::{CompletionItem, Diagnostic, Hover, Location};

#[derive(Debug)]
pub enum LspMessage {
    /// initialize response
    Initialized,

    /// textDocument/publishDiagnostics
    Diagnostics(Vec<Diagnostic>),

    /// textDocument/completion response
    Completion(Vec<CompletionItem>),

    /// textDocument/hover response
    Hover(Hover),

    /// textDocument/definition response
    Definition(Vec<Location>),

    /// Anything we don't explicitly understand yet.
    Json(Value),
}
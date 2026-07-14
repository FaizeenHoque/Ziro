use super::protocol::{CompletionItem, Diagnostic, Hover};
#[derive(Debug)]
pub enum LspMessage {
    Initialized(Vec<String>),
    Diagnostics(String, Vec<Diagnostic>),
    Completion(Vec<CompletionItem>),
    Hover(usize, usize, Option<Hover>),
    SemanticTokens(Vec<u32>),
}

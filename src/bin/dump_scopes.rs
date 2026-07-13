// src/bin/dump_scopes.rs
// Standalone debug binary. Run with: cargo run --bin dump_scopes -- path/to/file.rs
//
// Purpose: print the ACTUAL scope name syntect assigns to every token,
// so you stop guessing from screenshots and can see exactly which scopes
// your theme file has no rule for.

use syntect::easy::ScopeRegionIterator;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use syntect::util::LinesWithEndings;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: dump_scopes <file>");
    let content = std::fs::read_to_string(path).expect("failed to read file");

    let syntax_set = SyntaxSet::load_defaults_newlines();

    let syntax = std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(|extension| syntax_set.find_syntax_by_extension(extension))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    println!("Using syntax: {}", syntax.name);

    let mut parse_state = ParseState::new(syntax);
    let mut scope_stack = ScopeStack::new();

    for (line_no, line) in LinesWithEndings::from(&content).enumerate() {
        let ops = parse_state
            .parse_line(line, &syntax_set)
            .expect("parse_line failed");

        for (text, op) in ScopeRegionIterator::new(&ops, line) {
            scope_stack
                .apply(op)
                .expect("failed to apply scope op");

            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            // This is the full scope stack, most specific scope last.
            // The LAST scope in this list is usually what a theme rule
            // needs to match against.
            let scopes: Vec<String> = scope_stack
                .as_slice()
                .iter()
                .map(|scope| scope.build_string())
                .collect();

            println!("line {:>3} | {:>20?} -> {}", line_no + 1, trimmed, scopes.join(" "));
        }
    }
}
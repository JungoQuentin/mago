use std::path::PathBuf;

use mago_span::Span;
use tower_lsp::lsp_types::Position as LspPosition;
use tower_lsp::lsp_types::Range as LspRange;

/// Convert an [`LspPosition`] into an offset
pub fn position_to_offset(file_path: &PathBuf, position: LspPosition) -> usize {
    // TODO: optimise with source-manager
    let content = std::fs::read_to_string(file_path).unwrap();
    let mut offset: usize = 0;

    for (line, content) in content.lines().enumerate() {
        if line == position.line as usize {
            offset += position.character as usize;
            break;
        }
        offset += content.len() + 1;
    }

    offset
}

/// Convert an offset into an [`LspPosition`]. Returns `None` when eof is reached
pub fn offset_to_position(file_path: &PathBuf, mut offset: usize) -> Option<LspPosition> {
    // TODO: optimise with source-manager
    let content = std::fs::read_to_string(file_path).unwrap();

    for (line, content) in content.lines().enumerate() {
        if offset <= content.len() {
            return Some(LspPosition { line: line as u32, character: offset as u32 });
        }
        offset -= content.len() + 1;
    }

    // TODO: unreachable ?
    None
}

pub fn span_to_range(file_path: &PathBuf, span: &Span) -> Option<LspRange> {
    let start = offset_to_position(file_path, span.start.offset)?;
    let end = offset_to_position(file_path, span.end.offset)?;

    Some(LspRange::new(start, end))
}

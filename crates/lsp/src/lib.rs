use std::str::FromStr;

use definition::DefinitionFinder;
use helpers::{offset_to_position, parse_file, position_to_offset};
use mago_span::HasSpan;
use tower_lsp::jsonrpc::Result as ServerResult;
use tower_lsp::lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, LocationLink, Position, Range, TextDocumentIdentifier,
    TextDocumentPositionParams, Url,
};

pub mod definition;
pub mod helpers;

pub fn goto_definition(params: GotoDefinitionParams) -> ServerResult<Option<GotoDefinitionResponse>> {
    let GotoDefinitionParams {
        text_document_position_params:
            TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri }, position },
        ..
    } = params;

    let file_path = uri.to_file_path().expect("only support file:// scheme");
    let offset: usize = position_to_offset(&file_path, position);

    let program = parse_file(&file_path);

    let idents = DefinitionFinder.find(&program, offset);

    let n = idents.len();

    if n != 0 {
        eprint!("\n\n{n} found : {:?}\n\n", idents);
    } else {
        eprintln!("-")
    };
    let Some(last_identifier) = idents.last() else {
        return Ok(None);
    };

    Ok(Some(GotoDefinitionResponse::Link(vec![LocationLink {
        origin_selection_range: Some(Range::new(
            offset_to_position(&file_path, last_identifier.span().start.offset).unwrap(),
            offset_to_position(&file_path, last_identifier.span().end.offset).unwrap(),
        )),
        target_uri: Url::from_str("file:///Users/quentin/perso/php/test-php/src/new/classic.php").unwrap(),
        target_range: Range::new(Position { line: 4, character: 6 }, Position { line: 4, character: 14 }),
        target_selection_range: Range::new(Position { line: 4, character: 6 }, Position { line: 4, character: 14 }),
    }])))
}

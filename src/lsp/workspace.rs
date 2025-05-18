use std::path::PathBuf;
use std::str::FromStr;

use ahash::HashMap;
use mago_lsp::definition::DefinitionFinder;
use mago_lsp::helpers::offset_to_position;
use mago_lsp::helpers::position_to_offset;
use mago_reference::query::Query;
use mago_reference::ReferenceFinder;
use mago_reference::ReferenceKind;
use mago_span::HasSpan;
use tower_lsp::lsp_types::*;

use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;
use mago_reflector::reflect;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_semantics::Semantics;
use mago_source::SourceCategory;
use mago_source::SourceManager;
use mago_span::Span;

use crate::config::Configuration;
use crate::error::Error;
use crate::reflection::reflect_all_non_user_defined_sources;
use crate::source;

#[derive(Debug)]
pub(super) struct MagoWorkspace {
    configuration: Configuration,
    source_manager: SourceManager,
    semantics: Vec<Semantics>,
    codebase: CodebaseReflection,
}

impl MagoWorkspace {
    pub async fn initialize(interner: &ThreadedInterner, root: PathBuf) -> Result<Self, Error> {
        let configuration = Configuration::load_from(root)?;
        let source_manager = source::load(interner, &configuration.source, true, true).await?;
        let sources: Vec<_> = source_manager.source_ids_for_category(SourceCategory::UserDefined).collect();
        let length = sources.len();

        let mut codebase = reflect_all_non_user_defined_sources(interner, &source_manager).await?;
        let mut handles = Vec::with_capacity(length);
        for source_id in sources {
            handles.push(tokio::spawn({
                let interner = interner.clone();
                let manager = source_manager.clone();

                async move {
                    let source = manager.load(&source_id)?;
                    let semantics = Semantics::build(&interner, configuration.php_version, source);
                    let reflections = reflect(&interner, &semantics.source, &semantics.program, &semantics.names);

                    Result::<_, Error>::Ok((semantics, reflections))
                }
            }));
        }

        let mut semantics = Vec::with_capacity(length);
        for handle in handles {
            let (semantic, reflections) = handle.await??;

            codebase = mago_reflector::merge(interner, codebase, reflections);
            semantics.push(semantic);
        }

        mago_reflector::populate(interner, &mut codebase);

        Ok(MagoWorkspace { configuration, source_manager, semantics, codebase })
    }

    pub async fn get_workspace_diagnostic_report(&self) -> Result<WorkspaceDiagnosticReport, Error> {
        let mut hashmap = HashMap::default();
        for semantics in self.semantics.iter() {
            for issue in semantics.issues.iter() {
                tracing::error!("issue: {:?}", issue);

                let (uri, diagnostic) = issue_to_diagnostic("semantics", &self.source_manager, issue)?;
                hashmap.entry(uri).or_insert_with(Vec::new).push(diagnostic);
            }
        }

        let mut reports = vec![];
        for (uri, diagnostics) in hashmap {
            let report = WorkspaceFullDocumentDiagnosticReport {
                uri,
                version: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport { result_id: None, items: diagnostics },
            };

            reports.push(WorkspaceDocumentDiagnosticReport::Full(report));
        }

        Ok(WorkspaceDiagnosticReport { items: reports })
    }

    pub async fn get_document_diagnostic(
        &self,
        document_url: &Url,
        path: PathBuf,
    ) -> Result<RelatedFullDocumentDiagnosticReport, Error> {
        let semantics = self.semantics.iter().find(|semantics| {
            let semantics_path = semantics.source.path.as_ref().expect("source must have a path");

            semantics_path == &path
        });

        let semantics = match semantics {
            Some(semantics) => semantics,
            None => {
                return Ok(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport { result_id: None, items: vec![] },
                })
            }
        };

        let mut diagnostics = Vec::new();
        let mut related_documents = HashMap::default();
        for issue in semantics.issues.iter() {
            tracing::error!("issue: {:?}", issue);

            let (url, diagnostic) = issue_to_diagnostic("semantics", &self.source_manager, issue)?;

            if url == *document_url {
                diagnostics.push(diagnostic);
            } else {
                related_documents
                    .entry(url)
                    .or_insert_with(|| FullDocumentDiagnosticReport { result_id: None, items: Vec::new() })
                    .items
                    .push(diagnostic);
            }
        }

        Ok(RelatedFullDocumentDiagnosticReport {
            related_documents: Some(
                related_documents
                    .into_iter()
                    .map(|(uri, report)| (uri, DocumentDiagnosticReportKind::Full(report)))
                    .collect(),
            ),
            full_document_diagnostic_report: FullDocumentDiagnosticReport { result_id: None, items: diagnostics },
        })
    }

    /// Simplest implementation of `goto_definition`, only for functions.
    pub async fn goto_function_definition(
        &self,
        interner: &ThreadedInterner,
        file: &PathBuf,
        cursor_position: Position,
    ) -> Option<LocationLink> {
        let semantics = self
            .semantics
            .iter()
            .find(|semantics| {
                let semantics_path = semantics.source.path.as_ref().expect("source must have a path");

                semantics_path == file
            })
            .unwrap();

        let offset: usize = position_to_offset(&file, cursor_position);
        let idents = DefinitionFinder.find(&semantics.program, offset);

        if idents.len() > 1 {
            eprintln!("y'en a trop");
            return None;
        }

        let Some(last_identifier) = idents.last() else {
            return None;
        };

        let string_identifier = last_identifier.value();
        let r = interner.lookup(&string_identifier);
        let references = ReferenceFinder::new(&interner).find(&semantics, Query::EndsWith(r.to_string(), true));
        let mut references = references.iter().filter(|reference| reference.kind == ReferenceKind::Definition);
        let target_uri = Url::from_file_path(semantics.source.path.as_ref().unwrap()).unwrap();
        let Some(first) = references.next() else {
            eprintln!("pas de ref");
            return None;
        };
        if references.next().is_some() {
            eprintln!("warn: plusieurs !");
        }

        let range = Range::new(
            offset_to_position(&file, first.span.start.offset).unwrap(),
            offset_to_position(&file, first.span.end.offset).unwrap(),
        );
        Some(LocationLink {
            origin_selection_range: Some(Range::new(
                offset_to_position(&file, last_identifier.span().start.offset).unwrap(),
                offset_to_position(&file, last_identifier.span().end.offset).unwrap(),
            )),
            target_uri,
            target_range: range.clone(),
            target_selection_range: range,
        })
    }
}

fn issue_to_diagnostic(
    issue_source: &str,
    source_manager: &SourceManager,
    issue: &Issue,
) -> Result<(Url, Diagnostic), Error> {
    let primary_annotation = issue
        .annotations
        .iter()
        .find(|p| matches!(p.kind, AnnotationKind::Primary))
        .expect("issue should have at least one annotation");

    let span = &primary_annotation.span;
    let location = span_to_location(source_manager, span)?;

    // Convert Level to DiagnosticSeverity
    let severity = match issue.level {
        Level::Note => Some(DiagnosticSeverity::HINT),
        Level::Help => Some(DiagnosticSeverity::INFORMATION),
        Level::Warning => Some(DiagnosticSeverity::WARNING),
        Level::Error => Some(DiagnosticSeverity::ERROR),
    };

    let code: Option<NumberOrString> = issue.code.clone().map(NumberOrString::String);

    let mut related_information = Vec::new();
    for annotation in issue.annotations.iter() {
        let location = span_to_location(source_manager, &annotation.span)?;
        related_information
            .push(DiagnosticRelatedInformation { location, message: annotation.message.clone().unwrap_or_default() });
    }

    let diagnostic = Diagnostic {
        range: location.range,
        severity,
        code,
        code_description: None,
        message: issue.message.clone(),
        related_information: Some(related_information),
        tags: None,
        data: None,
        source: Some(issue_source.to_string()),
    };

    Ok((location.uri, diagnostic))
}

fn span_to_location(source_manager: &SourceManager, span: &Span) -> Result<Location, Error> {
    let source_id = &span.start.source;
    let source = source_manager.load(source_id)?;

    let range = Range {
        start: Position {
            line: (source.line_number(span.start.offset)) as u32,
            character: (source.column_number(span.start.offset)) as u32,
        },
        end: Position {
            line: (source.line_number(span.end.offset)) as u32,
            character: (source.column_number(span.end.offset)) as u32,
        },
    };

    let file_path = PathBuf::from(&source.path.expect("source must have a path"));
    let url = Url::from_file_path(file_path).expect("file path must be valid");

    Ok(Location { uri: url, range })
}

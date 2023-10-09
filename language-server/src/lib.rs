use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use lsp_server::{Connection, ExtractError, IoThreads, Message, Notification, Request, RequestId};
use lsp_types::notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument};
use lsp_types::{
    DiagnosticOptions, DiagnosticServerCapabilities, OneOf, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

use crate::dialect_resolver::{DialectResolver, LanguageBasedDialectResolver};
use crate::workspace::Workspace;

mod dialect_resolver;
mod workspace;

pub fn start_lsp_server() -> Result<IoThreads, Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![
                "{".to_string(),
                "}".to_string(),
                "(".to_string(),
                ")".to_string(),
                "[".to_string(),
                "]".to_string(),
                "\"".to_string(),
                "\'".to_string(),
            ]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            all_commit_characters: None,
            completion_item: None,
        }),
        code_lens_provider: Some(lsp_types::CodeLensOptions {
            resolve_provider: Some(true),
        }),
        inline_value_provider: Some(OneOf::Left(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: None,
            inter_file_dependencies: true,
            workspace_diagnostics: true,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        ..Default::default()
    })
    .unwrap();

    let workspace = Workspace::new();
    let resolver = LanguageBasedDialectResolver::new();

    connection
        .initialize(server_capabilities)
        .expect("Initialization failed due to wrong capabilities.");
    main_connection_loop(connection, workspace, resolver)?;
    return Ok(io_threads);
}

fn main_connection_loop(
    connection: Connection,
    workspace: RefCell<Workspace>,
    resolver: Rc<dyn DialectResolver>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
            }

            Message::Notification(notification) => {
                match cast_notification::<DidOpenTextDocument>(&notification) {
                    Ok(params) => {
                        let Some(_file) =
                            workspace.borrow_mut().open(&params, Rc::clone(&resolver))
                        else {
                            continue;
                        };
                    }
                    _ => {}
                }

                match cast_notification::<DidChangeTextDocument>(&notification) {
                    Ok(params) => {
                        let Some(_file) = workspace.borrow_mut().update(&params) else {
                            continue;
                        };
                    }
                    _ => {}
                }

                match cast_notification::<DidCloseTextDocument>(&notification) {
                    Ok(params) => {
                        workspace.borrow_mut().close(&params);
                    }
                    _ => {}
                }
            }

            Message::Response(_response) => {}
        }
    }

    return Ok(());
}

fn cast_request<R>(req: &Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    return req.clone().extract(R::METHOD);
}

fn cast_notification<R>(not: &Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    return not.clone().extract(R::METHOD);
}

//! Repository management routes.

use crate::api::types::{IdResponse, MessageResponse, RegisterRepoRequest};
use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use codemem_core::Repository;
use std::sync::Arc;

pub async fn list_repos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Repository>>, StatusCode> {
    match state.server.engine.list_repos() {
        Ok(repos) => Ok(Json(repos)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn register_repo(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRepoRequest>,
) -> Result<(StatusCode, Json<IdResponse>), (StatusCode, Json<MessageResponse>)> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Derive namespace from path basename (not full path)
    let namespace = Some(
        std::path::Path::new(&req.path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&req.path)
            .to_string(),
    );

    let repo = Repository {
        id: id.clone(),
        path: req.path,
        name: req.name,
        namespace,
        created_at: now,
        last_indexed_at: None,
        status: "idle".to_string(),
    };

    match state.server.engine.add_repo(&repo) {
        Ok(()) => Ok((StatusCode::CREATED, Json(IdResponse { id }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse {
                message: e.to_string(),
            }),
        )),
    }
}

pub async fn get_repo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Repository>, StatusCode> {
    match state.server.engine.get_repo(&id) {
        Ok(Some(repo)) => Ok(Json(repo)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_repo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse>, StatusCode> {
    match state.server.engine.remove_repo(&id) {
        Ok(true) => Ok(Json(MessageResponse {
            message: "Deleted".to_string(),
        })),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn index_repo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<MessageResponse>)> {
    let repo = match state.server.engine.get_repo(&id) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(MessageResponse {
                    message: "Repository not found".to_string(),
                }),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse {
                    message: e.to_string(),
                }),
            ))
        }
    };

    // Update status to indexing
    let _ = state
        .server
        .engine
        .update_repo_status(&id, "indexing", None);

    // Trigger indexing in background
    let path = repo.path.clone();
    let repo_id = id.clone();
    let indexing_tx = state.indexing_events.clone();
    let server = Arc::clone(&state.server);

    tokio::spawn(async move {
        let mut indexer = codemem_engine::Indexer::new();
        let root = std::path::Path::new(&path);

        match indexer.index_directory_with_progress(root, Some(&indexing_tx)) {
            Ok(_result) => {
                let now = chrono::Utc::now().to_rfc3339();
                let _ = server
                    .engine
                    .update_repo_status(&repo_id, "idle", Some(&now));
            }
            Err(_) => {
                let _ = server.engine.update_repo_status(&repo_id, "error", None);
            }
        }
    });

    Ok(Json(MessageResponse {
        message: "Indexing started".to_string(),
    }))
}

/// POST /api/repos/:id/analyze — full pipeline: index → enrich → PageRank → clusters
pub async fn analyze_repo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<MessageResponse>)> {
    let repo = match state.server.engine.get_repo(&id) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(MessageResponse {
                    message: "Repository not found".to_string(),
                }),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse {
                    message: e.to_string(),
                }),
            ))
        }
    };

    let _ = state.server.engine.update_repo_status(&id, "indexing", None);

    let path = repo.path.clone();
    let namespace = repo.namespace.clone().unwrap_or_else(|| {
        std::path::Path::new(&repo.path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&repo.path)
            .to_string()
    });
    let repo_id = id.clone();
    let server = Arc::clone(&state.server);

    tokio::task::spawn_blocking(move || {
        let root = std::path::Path::new(&path);
        let cd = codemem_engine::index::incremental::ChangeDetector::new();
        let options = codemem_engine::AnalyzeOptions {
            path: root,
            namespace: &namespace,
            git_days: 90,
            change_detector: Some(cd),
            progress: None,
            skip_scip: false,
            skip_embed: false,
            skip_enrich: false,
            force: false,
        };
        match server.engine.analyze(options) {
            Ok(result) => {
                let now = chrono::Utc::now().to_rfc3339();
                let _ = server.engine.update_repo_status(&repo_id, "idle", Some(&now));
                tracing::info!(
                    "Full analyze complete: {} files, {} symbols, {} communities",
                    result.files_parsed,
                    result.symbols_found,
                    result.community_count
                );
            }
            Err(e) => {
                tracing::error!("Full analyze failed: {e}");
                let _ = server.engine.update_repo_status(&repo_id, "error", None);
            }
        }
    });

    Ok(Json(MessageResponse {
        message: "Full analysis started (index → enrich → PageRank → clusters)".to_string(),
    }))
}

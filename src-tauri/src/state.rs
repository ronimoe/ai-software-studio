use crate::{
    core::worktree_context::WorktreeContextService,
    db::Db,
    dispatch::{get_autorun, DispatchHandle},
    engines::EngineService,
    error::AppError,
    git::GitService,
    process::ProcessRunner,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};
use std::sync::Arc;

pub struct AppState {
    pub db: Db,
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub engines: EngineService,
    pub verification: VerificationService,
    pub git: GitService,
    pub worktree_context: WorktreeContextService,
    pub process: Arc<ProcessRunner>,
    pub dispatch: DispatchHandle,
}

impl AppState {
    pub async fn init() -> Result<Self, AppError> {
        let db = Db::init().await?;
        let autorun = get_autorun(&db).await?;
        let dispatch = DispatchHandle::new(!autorun);
        Ok(Self {
            tasks: TaskService::new(db.clone()),
            projects: ProjectService::new(db.clone()),
            engines: EngineService::new(),
            verification: VerificationService::new(db.clone()),
            git: GitService::new(),
            worktree_context: WorktreeContextService::new(),
            process: Arc::new(ProcessRunner::new()),
            dispatch,
            db,
        })
    }
}

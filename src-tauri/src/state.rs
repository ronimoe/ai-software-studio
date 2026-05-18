use crate::{
    core::worktree_context::WorktreeContextService,
    db::Db,
    engines::EngineService,
    error::AppError,
    git::GitService,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};

pub struct AppState {
    pub db: Db,
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub engines: EngineService,
    pub verification: VerificationService,
    pub git: GitService,
    pub worktree_context: WorktreeContextService,
}

impl AppState {
    pub async fn init() -> Result<Self, AppError> {
        let db = Db::init().await?;
        Ok(Self {
            tasks: TaskService::new(db.clone()),
            projects: ProjectService::new(db.clone()),
            engines: EngineService::new(),
            verification: VerificationService::new(),
            git: GitService::new(),
            worktree_context: WorktreeContextService::new(),
            db,
        })
    }
}

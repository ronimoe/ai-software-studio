use crate::{
    db::Db,
    engines::EngineService,
    error::AppError,
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
}

impl AppState {
    pub async fn init() -> Result<Self, AppError> {
        let db = Db::init().await?;
        Ok(Self {
            tasks: TaskService::new(db.clone()),
            projects: ProjectService::new(db.clone()),
            engines: EngineService::new(),
            verification: VerificationService::new(),
            db,
        })
    }
}

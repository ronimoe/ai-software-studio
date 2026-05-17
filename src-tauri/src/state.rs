use crate::{
    engines::EngineService,
    projects::ProjectService,
    tasks::TaskService,
    verification::VerificationService,
};

pub struct AppState {
    pub tasks: TaskService,
    pub projects: ProjectService,
    pub engines: EngineService,
    pub verification: VerificationService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: TaskService::new(),
            projects: ProjectService::new(),
            engines: EngineService::new(),
            verification: VerificationService::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

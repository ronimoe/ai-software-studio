use super::settings::SettingsRepository;
use crate::db::Db;
use crate::models::VerificationSettings;

#[tokio::test]
async fn get_returns_defaults_when_unset() {
    let db = Db::test_pool().await.expect("db");
    let repo = SettingsRepository::new(db);
    let s = repo.get_for_project("proj-x").await.expect("get");
    assert_eq!(s.install.as_deref(), Some("pnpm install"));
    assert_eq!(s.test.as_deref(), Some("pnpm test"));
}

#[tokio::test]
async fn set_then_get_round_trips() {
    let db = Db::test_pool().await.expect("db");
    let repo = SettingsRepository::new(db);
    let custom = VerificationSettings {
        install: None,
        typecheck: Some("tsc --noEmit".into()),
        lint: None,
        test: Some("cargo test".into()),
        build: None,
    };
    repo.set_for_project("proj-y", &custom).await.expect("set");
    let got = repo.get_for_project("proj-y").await.expect("get");
    assert!(got.install.is_none());
    assert_eq!(got.typecheck.as_deref(), Some("tsc --noEmit"));
    assert!(got.lint.is_none());
    assert_eq!(got.test.as_deref(), Some("cargo test"));
}

#[tokio::test]
async fn set_overwrites_previous_values() {
    let db = Db::test_pool().await.expect("db");
    let repo = SettingsRepository::new(db);
    let first = VerificationSettings { install: Some("first".into()), ..Default::default() };
    let second = VerificationSettings { install: Some("second".into()), ..Default::default() };
    repo.set_for_project("p", &first).await.unwrap();
    repo.set_for_project("p", &second).await.unwrap();
    let got = repo.get_for_project("p").await.unwrap();
    assert_eq!(got.install.as_deref(), Some("second"));
}

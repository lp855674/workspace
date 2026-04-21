use paths::AppPaths;

#[test]
fn app_paths_for_root_sets_expected_file_names() {
    let app_paths = AppPaths::for_root("C:/FrameworkRoot");

    assert!(app_paths.settings_file.ends_with("settings.json"));
    assert!(app_paths.database_file.ends_with("framework.db"));
}

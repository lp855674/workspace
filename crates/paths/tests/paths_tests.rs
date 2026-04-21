use paths::AppPaths;
use std::path::Path;

#[test]
fn app_paths_for_root_sets_root_and_expected_files_under_root() {
    let app_paths = AppPaths::for_root("C:/FrameworkRoot");
    let root = Path::new("C:/FrameworkRoot");

    assert_eq!(app_paths.root(), root);
    assert_eq!(app_paths.keymap_file(), root.join("keymap.json"));

    let settings_file = app_paths.settings_file();
    let keymap_file = app_paths.keymap_file();
    let database_file = app_paths.database_file();

    assert!(settings_file.starts_with(root));
    assert!(keymap_file.starts_with(root));
    assert!(database_file.starts_with(root));

    assert_eq!(settings_file, root.join("settings.json"));
    assert_eq!(database_file, root.join("framework.db"));
}

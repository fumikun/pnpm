//! The `storeUmask` setting.
//!
//! Unix-only: it sets POSIX permission bits, which Windows does not have.
#![cfg(unix)]

use crate::_utils::append_workspace_yaml_key;
use pnpm_testing_utils::{
    bin::{AddMockedRegistry, CommandTempCwd},
    command_env::CommandTestExt,
};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

/// Installing under `umask 077` would otherwise leave the store file, and
/// so every hardlink to it, readable by its owner only.
#[test]
fn store_umask_decides_the_mode_of_hardlinked_files_whatever_the_process_umask() {
    let CommandTempCwd {
        pacquet,
        root,
        workspace,
        npmrc_info,
        ..
    } = CommandTempCwd::init().add_mocked_registry();
    let AddMockedRegistry { mock_instance, .. } = npmrc_info;
    append_workspace_yaml_key(&workspace, "packageImportMethod", "hardlink");
    fs::write(
        workspace.join("package.json"),
        serde_json::json!({ "dependencies": { "@pnpm.e2e/dep-of-pkg-with-1-dep": "100.0.0" } })
            .to_string(),
    )
    .expect("write package.json");
    let config_home = root.path().join("xdg-config");
    fs::create_dir_all(config_home.join("pnpm")).expect("create global config directory");
    fs::write(config_home.join("pnpm/config.yaml"), "storeUmask: 002\n")
        .expect("write global config");

    let status = Command::new("sh")
        .without_ambient_pnpm_config()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("-c")
        .arg(r#"umask 077 && exec "$0" install"#)
        .arg(pacquet.get_program())
        .current_dir(&workspace)
        .status()
        .expect("run install under umask 077");
    assert!(status.success());

    let installed = workspace.join("node_modules/@pnpm.e2e/dep-of-pkg-with-1-dep/package.json");
    assert_eq!(mode(&installed), 0o664);

    drop((root, mock_instance));
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path)
        .expect("stat installed file")
        .permissions()
        .mode()
        & 0o7777
}

use super::{
    Config, EnvVar, EnvVarOs, GetCurrentDir, GetHomeDir, HostNoHome, LinkProbe, OsString, Path,
    PathBuf, WorkspaceSettings, assert_eq, fs, io, safe_host_var, tempdir,
};
use pnpm_store_dir::StoreUmask;

fn umask(value: &str) -> Option<StoreUmask> {
    Some(value.parse().unwrap())
}

#[test]
fn global_config_reads_unquoted_digits_as_octal_in_any_order_with_the_store_dir() {
    for text in ["storeUmask: 002\nstoreDir: /store\n", "storeDir: /store\nstoreUmask: 002\n"] {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("config.yaml"), text).expect("write global config.yaml");
        let settings = WorkspaceSettings::load_global(tmp.path())
            .expect("loads global config")
            .expect("global config exists");
        let mut config = Config::default();

        settings.apply_to(&mut config, tmp.path());

        assert_eq!(config.store_dir.umask(), umask("002"), "{text}");
        assert_eq!(config.store_dir.root(), Path::new("/store/v11"), "{text}");
    }
}

#[test]
fn a_project_cannot_set_the_store_umask() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("pnpm-workspace.yaml"), "storeUmask: \"000\"\n")
        .expect("write to pnpm-workspace.yaml");

    let config = Config::new().current::<HostNoHome>(tmp.path()).expect("loads");

    assert_eq!(config.store_dir.umask(), None);
    assert_eq!(config.workspace_key_issues.refused, ["storeUmask"]);
}

#[test]
fn the_environment_sets_the_store_umask() {
    let tmp = tempdir().unwrap();

    struct HostWithStoreUmask;
    impl EnvVar for HostWithStoreUmask {
        fn var(name: &str) -> Option<String> {
            match name {
                "PNPM_CONFIG_STORE_UMASK" => Some("0o027".to_owned()),
                _ => safe_host_var(name),
            }
        }
    }
    impl EnvVarOs for HostWithStoreUmask {
        fn var_os(_: &str) -> Option<OsString> {
            None
        }
    }
    impl GetHomeDir for HostWithStoreUmask {
        fn home_dir() -> Option<PathBuf> {
            None
        }
    }
    inert_link_probe!(HostWithStoreUmask);
    host_current_dir!(HostWithStoreUmask);

    let config = Config::new().current::<HostWithStoreUmask>(tmp.path()).expect("loads");

    assert_eq!(config.store_dir.umask(), umask("027"));
}

#[test]
fn rejects_a_value_that_is_not_an_octal_umask() {
    for value in ["8", "1000", "true", "-1"] {
        let result = serde_saphyr::from_str::<WorkspaceSettings>(&format!("storeUmask: {value}"));
        assert!(result.is_err(), "must reject {value}: {result:?}");
    }
}

#[test]
fn resolves_and_resets_the_store_umask() {
    let mut config = Config::default();
    config.store_dir.set_umask(umask("002"));

    let resolved = WorkspaceSettings::from_resolved(&config);
    assert_eq!(resolved.store_umask, umask("002"));
    let mut reapplied = Config::default();
    resolved.apply_to(&mut reapplied, Path::new("/workspace"));
    assert_eq!(reapplied.store_dir.umask(), umask("002"));

    assert!(WorkspaceSettings::reset_setting_to_default::<HostNoHome>(
        &mut config,
        &Config::default(),
        "storeUmask",
        Path::new("/workspace"),
    ));
    assert_eq!(config.store_dir.umask(), None);
}

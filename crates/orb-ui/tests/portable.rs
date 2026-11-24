//! The settings file's name, its copy from `ui.toml`, the version keys and portable mode
//! (task-289): the rules run on every system through `Settings::load_resolved`, which takes
//! the portable folder and the per-user file as arguments.

use std::fs;
use std::path::{Path, PathBuf};

use orb_ui::settings::{Role, Settings};

const NAME: &str = "OpenRB-338.toml";

fn table(file: &Path) -> toml::Table {
    fs::read_to_string(file).unwrap().parse().unwrap()
}

/// A per-user folder holding `ui.toml` with a songs folder.
fn user_with_legacy(d: &Path) -> PathBuf {
    let user = d.join("user");
    fs::create_dir_all(&user).unwrap();
    fs::write(
        user.join("ui.toml"),
        "songs_dir = \"/legacy\"\nstartup_sound = false\n",
    )
    .unwrap();
    user.join(NAME)
}

#[test]
fn ui_toml_is_copied_once_and_left_alone() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let legacy = user.with_file_name("ui.toml");
    let before = fs::read(&legacy).unwrap();
    let mut s = Settings::load_resolved(Role::Standalone, None, Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")));
    assert!(!s.startup_sound);
    // the copy is made at the load, with the version key and the old values
    let t = table(&user);
    assert_eq!(
        t["standalone_version"].as_str(),
        Some(orb_ui::about::VERSION)
    );
    assert_eq!(t["songs_dir"].as_str(), Some("/legacy"));
    assert_eq!(t["startup_sound"].as_bool(), Some(false));
    s.set_songs_dir(Some(PathBuf::from("/new")));
    s.save().unwrap();
    assert_eq!(table(&user)["songs_dir"].as_str(), Some("/new"));
    // ui.toml is never changed or deleted
    assert_eq!(fs::read(&legacy).unwrap(), before);
    // a later load reads the new file, not ui.toml again
    let r = Settings::load_resolved(Role::Standalone, None, Some(user));
    assert_eq!(r.songs_dir, Some(PathBuf::from("/new")));
}

#[test]
fn the_plugin_copy_of_ui_toml_has_the_plugin_version() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let s = Settings::load_resolved(Role::Plugin, None, Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")));
    let t = table(&user);
    assert_eq!(t["plugin_version"].as_str(), Some(orb_ui::about::VERSION));
    assert!(!t.contains_key("standalone_version"));
}

#[test]
fn an_existing_new_file_wins_over_ui_toml() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    fs::write(&user, "songs_dir = \"/current\"\n").unwrap();
    let s = Settings::load_resolved(Role::Standalone, None, Some(user));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/current")));
}

#[test]
fn every_save_writes_its_version_and_keeps_the_others() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join(NAME);
    // never blank: even the defaults write a key
    let mut s = Settings::load_resolved(Role::Standalone, None, Some(file.clone()));
    s.save().unwrap();
    let t = table(&file);
    assert_eq!(
        t["standalone_version"].as_str(),
        Some(orb_ui::about::VERSION)
    );
    assert!(!t.contains_key("plugin_version"));
    // the plugin adds its own and keeps the app's as found
    fs::write(
        &file,
        "standalone_version = \"old app\"\nsongs_dir = \"/s\"\n",
    )
    .unwrap();
    let mut p = Settings::load_resolved(Role::Plugin, None, Some(file.clone()));
    p.set_startup_sound(false);
    p.save().unwrap();
    let t = table(&file);
    assert_eq!(t["standalone_version"].as_str(), Some("old app"));
    assert_eq!(t["plugin_version"].as_str(), Some(orb_ui::about::VERSION));
    assert_eq!(t["songs_dir"].as_str(), Some("/s"));
    // and the app keeps the plugin's
    fs::write(&file, "plugin_version = \"old plugin\"\n").unwrap();
    let mut a = Settings::load_resolved(Role::Standalone, None, Some(file.clone()));
    a.set_theme(orb_ui::settings::Theme::Classic);
    a.save().unwrap();
    let t = table(&file);
    assert_eq!(t["plugin_version"].as_str(), Some("old plugin"));
    assert_eq!(
        t["standalone_version"].as_str(),
        Some(orb_ui::about::VERSION)
    );
    // no temporary file stays
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1);
}

#[test]
fn an_absent_local_file_means_the_per_user_one() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = d.path().join("app");
    fs::create_dir_all(&local).unwrap();
    let mut s = Settings::load_resolved(Role::Standalone, Some(&local), Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")));
    s.set_songs_dir(Some(PathBuf::from("/x")));
    s.save().unwrap();
    assert!(!local.join(NAME).exists());
    assert_eq!(table(&user)["songs_dir"].as_str(), Some("/x"));
}

#[test]
fn a_local_file_with_settings_is_used_alone() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = d.path().join("app");
    fs::create_dir_all(&local).unwrap();
    fs::write(local.join(NAME), "songs_dir = \"/local\"\n").unwrap();
    let user_before = fs::read(&user).ok();
    let legacy_before = fs::read(user.with_file_name("ui.toml")).unwrap();
    let mut s = Settings::load_resolved(Role::Standalone, Some(&local), Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/local")));
    assert!(s.startup_sound, "nothing comes from the per-user settings");
    s.set_mods_dir(Some(PathBuf::from("/m")));
    s.save().unwrap();
    let t = table(&local.join(NAME));
    assert_eq!(t["mods_dir"].as_str(), Some("/m"));
    assert_eq!(t["songs_dir"].as_str(), Some("/local"));
    assert!(t.contains_key("standalone_version"));
    // the per-user file was not read, created or written
    assert_eq!(fs::read(&user).ok(), user_before);
    assert!(!user.exists());
    assert_eq!(
        fs::read(user.with_file_name("ui.toml")).unwrap(),
        legacy_before
    );
}

#[test]
fn an_empty_local_file_takes_the_per_user_settings_in() {
    let empties = [
        "",
        "  \n\t\r\n",
        "# the portable settings\n# nothing yet\n",
        "\n   # a comment after blank lines\n",
    ];
    for empty in empties {
        let d = tempfile::tempdir().unwrap();
        let user = user_with_legacy(d.path());
        let local = d.path().join("app");
        fs::create_dir_all(&local).unwrap();
        fs::write(local.join(NAME), empty).unwrap();
        let mut s = Settings::load_resolved(Role::Standalone, Some(&local), Some(user.clone()));
        assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")), "{empty:?}");
        assert!(!s.startup_sound);
        // written into the local file at once, which is no longer empty
        let t = table(&local.join(NAME));
        assert_eq!(t["songs_dir"].as_str(), Some("/legacy"), "{empty:?}");
        assert_eq!(
            t["standalone_version"].as_str(),
            Some(orb_ui::about::VERSION)
        );
        // from now on the local file only
        s.set_songs_dir(Some(PathBuf::from("/later")));
        s.save().unwrap();
        assert_eq!(
            table(&local.join(NAME))["songs_dir"].as_str(),
            Some("/later")
        );
        // the per-user folder holds ui.toml only, as it was: no copy, no new file
        assert!(!user.exists(), "{empty:?}");
        assert_eq!(fs::read_dir(user.parent().unwrap()).unwrap().count(), 1);
        let again = Settings::load_resolved(Role::Standalone, Some(&local), Some(user));
        assert_eq!(again.songs_dir, Some(PathBuf::from("/later")));
    }
}

#[test]
fn an_empty_local_file_with_no_per_user_file_gets_the_defaults() {
    let d = tempfile::tempdir().unwrap();
    let local = d.path().join("app");
    fs::create_dir_all(&local).unwrap();
    fs::write(local.join(NAME), "").unwrap();
    let s = Settings::load_resolved(
        Role::Plugin,
        Some(&local),
        Some(d.path().join("none/x.toml")),
    );
    assert!(s.startup_sound && s.notice.is_none());
    let t = table(&local.join(NAME));
    assert_eq!(t["plugin_version"].as_str(), Some(orb_ui::about::VERSION));
    assert!(!d.path().join("none").exists());
}

#[test]
fn a_damaged_local_file_is_an_error_and_never_falls_back() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = d.path().join("app");
    fs::create_dir_all(&local).unwrap();
    fs::write(local.join(NAME), "songs_dir = = broken\n").unwrap();
    let s = Settings::load_resolved(Role::Standalone, Some(&local), Some(user.clone()));
    let e = s.load_error.expect("an error to show");
    assert!(e.contains(NAME) && e.contains("damaged"), "{e}");
    assert_eq!(s.songs_dir, None);
    assert_eq!(
        fs::read_to_string(local.join(NAME)).unwrap(),
        "songs_dir = = broken\n"
    );
    assert!(!user.exists());
}

#[cfg(unix)]
#[test]
fn an_unwritable_local_folder_warns_once_and_writes_nothing_per_user() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = d.path().join("app");
    fs::create_dir_all(&local).unwrap();
    fs::write(local.join(NAME), "songs_dir = \"/local\"\n").unwrap();
    fs::set_permissions(&local, fs::Permissions::from_mode(0o555)).unwrap();
    // root writes anywhere: nothing to check
    if fs::write(local.join("probe"), "").is_ok() {
        fs::set_permissions(&local, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let mut s = Settings::load_resolved(Role::Standalone, Some(&local), Some(user.clone()));
    // read as it is
    assert_eq!(s.songs_dir, Some(PathBuf::from("/local")));
    s.set_mods_dir(Some(PathBuf::from("/m")));
    // not an error, one notice
    s.save().unwrap();
    let n = s.notice.take().expect("a notice");
    assert!(
        n.starts_with(&format!("Settings cannot be saved in {}:", local.display())),
        "{n}"
    );
    s.set_mods_dir(Some(PathBuf::from("/m2")));
    s.save().unwrap();
    assert!(s.notice.is_none(), "once only");
    // the value stays in memory, the file as it was, no per-user file
    assert_eq!(s.mods_dir, Some(PathBuf::from("/m2")));
    assert_eq!(
        fs::read_to_string(local.join(NAME)).unwrap(),
        "songs_dir = \"/local\"\n"
    );
    assert!(!user.exists());
    fs::set_permissions(&local, fs::Permissions::from_mode(0o755)).unwrap();
}

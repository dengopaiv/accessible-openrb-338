//! The settings file's name, its copy from `ui.toml`, the atomic save and portable mode
//! (task-289; the marker file, task-295): the rules run on every system through
//! `Settings::load_resolved`, which takes the portable folder and the per-user file as arguments.

use std::fs;
use std::path::{Path, PathBuf};

use orb_ui::settings::Settings;

const NAME: &str = "OpenRB-338.toml";
const MARKER: &str = "OpenRB-338.portable";

/// A local folder (the program's) with the marker file.
fn local_with_marker(d: &Path) -> PathBuf {
    let local = d.join("app");
    fs::create_dir_all(&local).unwrap();
    fs::write(local.join(MARKER), "").unwrap();
    local
}

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
    let mut s = Settings::load_resolved(None, Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")));
    assert!(!s.startup_sound);
    // the copy is made at the load, with the old values
    let t = table(&user);
    assert_eq!(t["songs_dir"].as_str(), Some("/legacy"));
    assert_eq!(t["startup_sound"].as_bool(), Some(false));
    s.set_songs_dir(Some(PathBuf::from("/new")));
    s.save().unwrap();
    assert_eq!(table(&user)["songs_dir"].as_str(), Some("/new"));
    // ui.toml is never changed or deleted
    assert_eq!(fs::read(&legacy).unwrap(), before);
    // a later load reads the new file, not ui.toml again
    let r = Settings::load_resolved(None, Some(user));
    assert_eq!(r.songs_dir, Some(PathBuf::from("/new")));
}

#[test]
fn an_existing_new_file_wins_over_ui_toml() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    fs::write(&user, "songs_dir = \"/current\"\n").unwrap();
    let s = Settings::load_resolved(None, Some(user));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/current")));
}

#[test]
fn a_save_is_atomic_and_leaves_no_temporary_file() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join(NAME);
    let mut s = Settings::load_resolved(None, Some(file.clone()));
    s.save().unwrap();
    assert!(!table(&file).is_empty());
    // an unknown key (such as the removed version keys of a dev build) loads and is dropped at the
    // save
    fs::write(&file, "removed_key = \"old\"\nsongs_dir = \"/s\"\n").unwrap();
    let mut a = Settings::load_resolved(None, Some(file.clone()));
    assert_eq!(a.songs_dir, Some(PathBuf::from("/s")));
    a.set_theme(orb_ui::settings::Theme::Classic);
    a.save().unwrap();
    let t = table(&file);
    assert_eq!(t["songs_dir"].as_str(), Some("/s"));
    assert!(!t.contains_key("removed_key"));
    // no temporary file stays
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1);
}

#[test]
fn without_the_marker_the_per_user_file_is_used_even_if_a_local_one_exists() {
    for local_file in [None, Some(""), Some("songs_dir = \"/local\"\n")] {
        let d = tempfile::tempdir().unwrap();
        let user = user_with_legacy(d.path());
        let local = d.path().join("app");
        fs::create_dir_all(&local).unwrap();
        if let Some(t) = local_file {
            fs::write(local.join(NAME), t).unwrap();
        }
        let mut s = Settings::load_resolved(Some(&local), Some(user.clone()));
        assert_eq!(
            s.songs_dir,
            Some(PathBuf::from("/legacy")),
            "{local_file:?}"
        );
        assert!(!s.portable);
        s.set_songs_dir(Some(PathBuf::from("/x")));
        s.save().unwrap();
        assert_eq!(table(&user)["songs_dir"].as_str(), Some("/x"));
        // the local file is ignored, not changed or deleted
        match local_file {
            None => assert!(!local.join(NAME).exists()),
            Some(t) => assert_eq!(fs::read_to_string(local.join(NAME)).unwrap(), t),
        }
    }
}

#[test]
fn a_local_file_with_settings_is_used_alone() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = local_with_marker(d.path());
    fs::write(local.join(NAME), "songs_dir = \"/local\"\n").unwrap();
    let user_before = fs::read(&user).ok();
    let legacy_before = fs::read(user.with_file_name("ui.toml")).unwrap();
    let mut s = Settings::load_resolved(Some(&local), Some(user.clone()));
    assert_eq!(s.songs_dir, Some(PathBuf::from("/local")));
    assert!(s.startup_sound, "nothing comes from the per-user settings");
    s.set_mods_dir(Some(PathBuf::from("/m")));
    s.save().unwrap();
    let t = table(&local.join(NAME));
    assert_eq!(t["mods_dir"].as_str(), Some("/m"));
    assert_eq!(t["songs_dir"].as_str(), Some("/local"));
    // the per-user file was not read, created or written
    assert_eq!(fs::read(&user).ok(), user_before);
    assert!(!user.exists());
    assert_eq!(
        fs::read(user.with_file_name("ui.toml")).unwrap(),
        legacy_before
    );
}

#[test]
fn an_absent_local_file_with_the_marker_takes_the_per_user_settings_in() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = local_with_marker(d.path());
    let mut s = Settings::load_resolved(Some(&local), Some(user.clone()));
    assert!(s.portable);
    assert_eq!(s.songs_dir, Some(PathBuf::from("/legacy")));
    assert!(!s.startup_sound);
    // written into the local file at once
    let t = table(&local.join(NAME));
    assert_eq!(t["songs_dir"].as_str(), Some("/legacy"));
    // from now on the local file only
    s.set_songs_dir(Some(PathBuf::from("/later")));
    s.save().unwrap();
    assert_eq!(
        table(&local.join(NAME))["songs_dir"].as_str(),
        Some("/later")
    );
    // the per-user folder holds ui.toml only, as it was: no copy, no new file
    assert!(!user.exists());
    assert_eq!(fs::read_dir(user.parent().unwrap()).unwrap().count(), 1);
    let again = Settings::load_resolved(Some(&local), Some(user));
    assert_eq!(again.songs_dir, Some(PathBuf::from("/later")));
}

#[test]
fn a_marker_with_content_counts_and_a_present_local_file_is_kept_as_it_is() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = local_with_marker(d.path());
    fs::write(local.join(MARKER), "portable mode is on\n").unwrap();
    // an empty local file is used as it is: no per-user settings come in
    fs::write(local.join(NAME), "").unwrap();
    let s = Settings::load_resolved(Some(&local), Some(user.clone()));
    assert!(s.portable && s.load_error.is_none());
    assert_eq!(s.songs_dir, None);
    assert!(s.startup_sound);
    assert_eq!(fs::read_to_string(local.join(NAME)).unwrap(), "");
    assert!(!user.exists());
}

#[test]
fn an_absent_local_file_with_no_per_user_file_gets_the_defaults() {
    let d = tempfile::tempdir().unwrap();
    let local = local_with_marker(d.path());
    let s = Settings::load_resolved(Some(&local), Some(d.path().join("none/x.toml")));
    assert!(s.startup_sound && s.notice.is_none());
    let t = table(&local.join(NAME));
    assert!(!t.is_empty());
    assert!(!d.path().join("none").exists());
}

#[test]
fn a_damaged_local_file_is_an_error_and_never_falls_back() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = local_with_marker(d.path());
    fs::write(local.join(NAME), "songs_dir = = broken\n").unwrap();
    let s = Settings::load_resolved(Some(&local), Some(user.clone()));
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
    let local = local_with_marker(d.path());
    fs::write(local.join(NAME), "songs_dir = \"/local\"\n").unwrap();
    fs::set_permissions(&local, fs::Permissions::from_mode(0o555)).unwrap();
    // root writes anywhere: nothing to check
    if fs::write(local.join("probe"), "").is_ok() {
        fs::set_permissions(&local, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    let mut s = Settings::load_resolved(Some(&local), Some(user.clone()));
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

/// The fork's older portable copy (its task-223, kept by task-233): `portable.txt` and no
/// `OpenRB-338.portable` beside the program, the settings in `data/ui.toml` with a songs folder
/// stored relative to the program's folder. The first start is portable, takes those settings
/// (not the per-user ones) with the folder read inside the program's folder, writes them to
/// `OpenRB-338.toml` with the folder relative again, and leaves the old file as it was.
#[test]
fn an_older_fork_copy_keeps_its_settings_and_relative_folders() {
    let d = tempfile::tempdir().unwrap();
    let user = user_with_legacy(d.path());
    let local = d.path().join("stick").join("OpenRB");
    fs::create_dir_all(local.join("data")).unwrap();
    fs::write(
        local.join("portable.txt"),
        "This file makes this copy portable.\n",
    )
    .unwrap();
    let old = local.join("data").join("ui.toml");
    fs::write(&old, "songs_dir = \"songs\"\nstartup_sound = false\n").unwrap();
    let before = fs::read(&old).unwrap();

    let mut s = Settings::load_resolved(Some(&local), Some(user.clone()));
    assert!(s.portable);
    assert_eq!(s.portable_root.as_deref(), Some(local.as_path()));
    assert_eq!(
        s.songs_dir,
        Some(local.join("songs")),
        "read inside the folder"
    );
    assert!(
        !s.startup_sound,
        "the old portable settings, not the defaults"
    );
    let t = table(&local.join(NAME));
    assert_eq!(
        t["songs_dir"].as_str(),
        Some("songs"),
        "stored relative again"
    );
    assert_eq!(
        fs::read(&old).unwrap(),
        before,
        "data/ui.toml is not changed"
    );
    assert!(!user.exists(), "nothing is written per-user");

    // later saves and loads use OpenRB-338.toml only
    s.set_startup_sound(true);
    s.save().unwrap();
    let again = Settings::load_resolved(Some(&local), Some(user));
    assert!(again.startup_sound);
    assert_eq!(again.songs_dir, Some(local.join("songs")));
    assert_eq!(fs::read(&old).unwrap(), before);
}

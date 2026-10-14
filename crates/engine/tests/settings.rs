use evkey_engine::settings::{load, stamp, CodeTable, TonePlacement};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("evkey-settings-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn missing_files_give_defaults() {
    let d = temp_dir("missing");
    let l = load(&d).unwrap();
    assert!(l.settings.check_spelling && l.macros.is_empty());
    assert_eq!(stamp(&d), [None, None]);
}

#[test]
fn files_are_parsed_and_mapped_to_engine_config() {
    let d = temp_dir("parse");
    std::fs::write(
        d.join("config.toml"),
        "tone_placement = \"old\"\ncode_table = \"tcvn3\"\nquick_telex = true\nexcluded_apps = [\"Game.exe\"]\nexcluded_apps_macos = [\"Game.exe\"]\n",
    )
    .unwrap();
    std::fs::write(d.join("macros.toml"), "btw = \"by the way\"\n").unwrap();
    let l = load(&d).unwrap();
    assert_eq!(l.settings.tone_placement, TonePlacement::Old);
    assert_eq!(l.settings.code_table, CodeTable::Tcvn3);
    assert!(l.settings.is_excluded("game.EXE") && !l.settings.is_excluded("notepad.exe"));
    assert_eq!(l.macros["btw"], "by the way");
    let c = l.settings.to_config(!l.macros.is_empty());
    // `old` style (hoà) is the engine's "modern orthography = 1"
    assert_eq!((c.use_modern_orthography, c.code_table, c.quick_telex, c.use_macro), (1, 1, 1, 1));
    assert_eq!(l.settings.to_config(false).use_macro, 0);
    assert!(stamp(&d).iter().all(|s| s.is_some()));
}

#[test]
fn typos_and_bad_values_are_errors_not_silently_ignored() {
    let d = temp_dir("bad");
    std::fs::write(d.join("config.toml"), "chek_spelling = false\n").unwrap();
    assert!(load(&d).is_err());
    std::fs::write(d.join("config.toml"), "input_type = \"qwerty\"\n").unwrap();
    assert!(load(&d).is_err());
}

#[test]
fn save_then_load_round_trips() {
    use evkey_engine::settings::{save, InputKind, Loaded, TempOffKey};
    let d = temp_dir("save");
    let mut l = Loaded::default();
    l.settings.input_type = InputKind::Vni;
    l.settings.temp_off_key = TempOffKey::Alt;
    l.settings.excluded_apps = vec!["game.exe".into()];
    l.macros.insert("vn".into(), "Việt Nam".into());
    save(&d.join("nested"), &l).unwrap();
    assert_eq!(load(&d.join("nested")).unwrap(), l);
}

#[test]
fn toggle_hotkey_is_validated_on_load() {
    let d = temp_dir("hotkey");
    let default_hotkey = if cfg!(target_os = "macos") { "ctrl+shift" } else { "alt+z" };
    assert_eq!(load(&d).unwrap().settings.toggle_hotkey, default_hotkey);
    std::fs::write(d.join("config.toml"), "toggle_hotkey = \"alt+zz\"\n").unwrap();
    let err = load(&d).unwrap_err();
    assert!(err.contains("toggle_hotkey"), "{err}");
    std::fs::write(d.join("config.toml"), "toggle_hotkey = \"ctrl+shift\"\n").unwrap();
    assert_eq!(load(&d).unwrap().settings.toggle_hotkey, "ctrl+shift");
}

#[test]
fn config_from_2x_with_backend_key_still_loads_and_drops_it_on_save() {
    let d = temp_dir("legacy-backend");
    std::fs::write(d.join("config.toml"), "backend = \"tsf\"\nquick_telex = true\n").unwrap();
    let l = load(&d).unwrap();
    assert!(l.settings.quick_telex);
    evkey_engine::settings::save(&d, &l).unwrap();
    assert!(!std::fs::read_to_string(d.join("config.toml")).unwrap().contains("backend"));
}

//! The Audio Output choice in the settings file (task-191): the keys `audio_*`, the default,
//! the values skipped when wrong, and another instance's choice surviving a save.

use std::fs;

use orb_ui::settings::{AudioChoice, Settings};

#[test]
fn the_audio_choice_round_trips_through_toml() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join("ui.toml");
    let mut s = Settings::load_from(&file);
    assert_eq!(s.audio, AudioChoice::default());
    assert_eq!(s.audio.buffer, 2048);
    s.save().unwrap();
    // the default writes only the buffer size
    let t: toml::Table = fs::read_to_string(&file).unwrap().parse().unwrap();
    assert_eq!(t["audio_buffer_size"].as_integer(), Some(2048));
    for k in [
        "audio_driver",
        "audio_device",
        "audio_device_name",
        "audio_sample_rate",
    ] {
        assert!(!t.contains_key(k), "{k}");
    }
    // a chosen driver, device, rate and size
    let chosen = AudioChoice {
        driver: Some("WASAPI".into()),
        device: Some("WASAPI:{0.0.0.00000000}.{abc}".into()),
        device_name: Some("Speakers (USB \"DAC\")".into()),
        rate: Some(48000),
        buffer: 512,
    };
    s.set_audio(chosen.clone());
    s.save().unwrap();
    let r = Settings::load_from(&file);
    assert_eq!(r.audio, chosen);
    assert_eq!(r, s);
    // the choice becomes the request that opens it, and back
    let req = chosen.request();
    assert_eq!(req.driver.as_deref(), Some("WASAPI"));
    assert_eq!(req.device.as_ref().unwrap().name, "Speakers (USB \"DAC\")");
    assert_eq!((req.rate, req.buffer), (Some(48000), 512));
    assert_eq!(AudioChoice::from_request(&req), chosen);
    assert!(AudioChoice::default().request().is_default());
    // another instance's save of an unrelated key keeps the choice
    let mut other = Settings::load_from(&file);
    other.set_startup_sound(false);
    other.save().unwrap();
    let mut third = Settings::load_from(&file);
    assert_eq!(third.audio, chosen);
    // and going back to the defaults removes the optional keys
    third.set_audio(AudioChoice::default());
    third.save().unwrap();
    let t: toml::Table = fs::read_to_string(&file).unwrap().parse().unwrap();
    assert!(!t.contains_key("audio_device"));
    assert_eq!(Settings::load_from(&file).audio, AudioChoice::default());
}

#[test]
fn a_bad_audio_value_is_skipped_and_the_others_kept() {
    let d = tempfile::tempdir().unwrap();
    let file = d.path().join("ui.toml");
    fs::write(
        &file,
        concat!(
            "audio_driver = 3\naudio_device = \"\"\naudio_sample_rate = 5\n",
            "audio_buffer_size = 100000\naudio_device_name = \"X\"\n"
        ),
    )
    .unwrap();
    let s = Settings::load_from(&file);
    assert_eq!(s.audio.driver, None);
    assert_eq!(s.audio.device, None);
    assert_eq!(s.audio.rate, None);
    assert_eq!(s.audio.buffer, 2048);
    assert_eq!(s.audio.device_name.as_deref(), Some("X"));
}

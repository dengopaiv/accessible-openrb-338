//! Probe of the installed ASIO drivers, step by step (task-252). Ignored: it loads the user's
//! real drivers one at a time (each briefly; no program audio, but with
//! `ORB_ASIO_PROBE_START=1` the driver plays whatever its own buffers hold for 1.5 s) and logs
//! which step of cpal's `Devices::next` each one would fail. Run on Windows x86_64 in release:
//!
//! `cargo test --release -p orb-player --test asio_probe -- --ignored --nocapture`
//!
//! No normal test needs it.
#![cfg(all(windows, target_arch = "x86_64"))]

use asio_sys::Asio;

#[test]
#[ignore = "loads the installed ASIO drivers; run by hand"]
fn probe_installed_asio_drivers() {
    {
        let asio = Asio::new();
        let names = asio.driver_names();
        eprintln!("asio-sys driver_names: {names:?}");
        for name in &names {
            eprintln!("--- {name}");
            // cpal's Devices::next drops the previous driver first: one driver at a time.
            let driver = match asio.load_driver(name) {
                Ok(d) => {
                    eprintln!("  load_driver (load + ASIOInit): ok");
                    d
                }
                Err(e) => {
                    eprintln!("  load_driver (load + ASIOInit): FAILED {e:?}  -> cpal skips it");
                    continue;
                }
            };
            let ch = driver.channels();
            eprintln!("  channels: {ch:?}");
            eprintln!("  sample_rate: {:?}", driver.sample_rate());
            eprintln!("  buffersize_range: {:?}", driver.buffersize_range());
            eprintln!("  input_data_type: {:?}", driver.input_data_type());
            eprintln!("  output_data_type: {:?}", driver.output_data_type());
            for r in [44100.0, 48000.0, 88200.0, 96000.0] {
                eprintln!("  can_sample_rate({r}): {:?}", driver.can_sample_rate(r));
            }
            let listed = match &ch {
                Err(_) => "no: channels() failed",
                Ok(c) if c.ins == 0 && c.outs == 0 => "no: zero channels",
                Ok(_) if driver.buffersize_range().is_err() => "no: buffersize_range() failed",
                Ok(_) => "yes",
            };
            eprintln!("  cpal would list it (Devices::next): {listed}");
            // What cpal's output_devices() adds: a device with no supported rate has no
            // configuration, so `supports_output` drops it.
            let rates: Vec<f64> = [44100.0, 48000.0, 88200.0, 96000.0]
                .into_iter()
                .filter(|&r| driver.can_sample_rate(r).unwrap_or(false))
                .collect();
            eprintln!("  rates cpal keeps (can_sample_rate Ok(true)): {rates:?}");
            // Beyond cpal: does the driver go on? Buffers only, nothing is started.
            eprintln!(
                "  set_sample_rate(44100): {:?}",
                driver.set_sample_rate(44100.0)
            );
            eprintln!("  sample_rate now: {:?}", driver.sample_rate());
            eprintln!(
                "  can_sample_rate(44100) now: {:?}",
                driver.can_sample_rate(44100.0)
            );
            let n = ch.map(|c| c.outs as usize).unwrap_or(0).min(2);
            if n > 0 {
                match driver.prepare_output_stream(None, n, None) {
                    Ok(_) => eprintln!("  prepare_output_stream({n} ch): ok"),
                    Err(e) => eprintln!("  prepare_output_stream({n} ch): FAILED {e:?}"),
                }
                // With ORB_ASIO_PROBE_START=1 also ASIOStart for 1.5 s on the driver's own,
                // unwritten buffers (no callback is registered: the driver plays whatever its
                // buffers hold, at the device's volume), then log the rate the driver reports.
                if std::env::var_os("ORB_ASIO_PROBE_START").is_some() {
                    eprintln!("  start: {:?}", driver.start());
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    eprintln!("  sample_rate running: {:?}", driver.sample_rate());
                    eprintln!("  stop: {:?}", driver.stop());
                }
                eprintln!("  dispose_buffers: {:?}", driver.dispose_buffers());
            }
        }
    }
    // The cpal view, for comparison.
    let listed = orb_player::devices::list_devices(Some("ASIO"));
    eprintln!("orb_player::devices::list_devices(ASIO): {listed:?}");
}

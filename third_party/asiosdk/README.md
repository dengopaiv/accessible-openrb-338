# The ASIO SDK subset (task-193)

Steinberg's ASIO SDK, version 2.3.4, the files the x86_64 Windows build compiles and no other.
cpal's `asio` feature builds `asio-sys`, whose build script compiles every `.cpp` of `common/`,
`host/` and `host/pc/` and generates its Rust bindings with bindgen from the headers it finds
(`asio.h`, `asiosys.h`, `asiodrivers.h`). The build gets this folder from `CPAL_ASIO_DIR`
(`.cargo/config.toml`, relative to the repository), so it downloads nothing.

- Version: ASIO SDK 2.3.4 (2025-10-15)
- Source: https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip
- SHA-256 of the zip: `d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca`
- The files are the zip's `ASIOSDK/` files byte for byte (CRLF line ends kept:
  `third_party/asiosdk/** -text` in `.gitattributes`). Nothing is modified.

| File | In the zip | SHA-256 |
|---|---|---|
| `LICENSE.txt` | `LICENSE.txt` | `61e56814f24b32216884b2db0b4dcc317111a5ab354d54d19dd9fc13d71b1c7a` |
| `common/asio.cpp` | same | `4ecee2bfaebd8b0777db96175d2b2ea8d0cd684dfaf9651480c44b0ebec0fd61` |
| `common/asio.h` | same | `93d2f80d49b4ac9c963514e7c1307ab5111d8104246192233745beb058534ea1` |
| `common/asiosys.h` | same | `f44a1ed0fd7c056ff047aeebc4c05a8c185f0585a24d8debc55c70107777dbef` |
| `common/iasiodrv.h` | same | `86ad4788452c74823653cf57d4bd557c48bb6542d7e6eb7fff80102c705a066d` |
| `host/asiodrivers.cpp` | same | `6f5c7529047c482998201df4b9c9e6bcdeee782ed764e6324d49b3bca4340601` |
| `host/asiodrivers.h` | same | `23f78334ed9aff7f9db4b0d36b228595005c6f6a3ad842e9d0ad33edfb041a8f` |
| `host/ginclude.h` | same | `96db001a91c469702afac88178f289bd50f78dd43ed12c174f898a44a7a56676` |
| `host/pc/asiolist.cpp` | same | `d3f7d83f561eaafed17393c3c95ba077f2ec109e2f7c762dcdf905f0e5c17f02` |
| `host/pc/asiolist.h` | same | `daab5066bcb9ca79552541bc2981cf43cde7ac72bd4572286764097c9cc92010` |

Left out: the driver side (`common/combase.*`, `dllentry.cpp`, `register.cpp`,
`debugmessage.cpp`, `asiodrvr.*`, `driver/`), `host/ASIOConvertSamples.*`, `host/sample/`,
the PDFs, the SDK's own README and the logo artwork. The build compiles without them (the
driver-side files are for writing a driver, and `asiodrvr.cpp` is the Mac one that asio-sys
skips).

## License

The SDK is dual licensed: Steinberg's proprietary ASIO License, or the GNU General Public
License Version 3 (`LICENSE.txt`). OpenRB-338 (GPL-3.0-only) uses it under the GPL Version 3
(task-192). The host helpers (`host/asiodrivers.*`, `host/pc/asiolist.*`, `host/ginclude.h`) carry
their own BSD 3-clause notice in their headers. The x86_64 Windows packages ship both in
`THIRD-PARTY-NOTICES.txt` (`scripts/asio-notices.sh`).

ASIO is a trademark and software of Steinberg Media Technologies GmbH.

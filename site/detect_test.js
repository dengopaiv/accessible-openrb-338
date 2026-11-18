// node site/detect_test.js (task-212): the detection on real user agent strings.
const assert = require("assert");
const { detect } = require("./detect.js");

// the groups the page shows for a detection: the arch of the system, or both when unsure
const ids = (f) => (f.os ? f.os + (f.arch ? "-" + f.arch : "-both") : "all");

const SAFARI_MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
const CHROME_MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const CHROME_WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const FF_WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0";
const FF_LINUX = "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0";
const CHROME_LINUX_ARM = "Mozilla/5.0 (X11; Linux aarch64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
const IPHONE = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1";
const ANDROID = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36";

// macOS: Safari/Firefox cannot tell Apple silicon from Intel: both dmgs
assert.deepStrictEqual(detect(SAFARI_MAC, "MacIntel", null), { os: "mac", arch: null });
assert.strictEqual(ids(detect(SAFARI_MAC, "MacIntel", null)), "mac-both");
// Chrome with userAgentData knows
assert.strictEqual(ids(detect(CHROME_MAC, "MacIntel", { platform: "macOS", architecture: "arm" })), "mac-arm64");
assert.strictEqual(ids(detect(CHROME_MAC, "MacIntel", { platform: "macOS", architecture: "x86" })), "mac-x64");
// Windows: the string says x64 even on ARM, so without userAgentData both installers
assert.strictEqual(ids(detect(FF_WIN, "Win32", null)), "windows-both");
assert.strictEqual(ids(detect(CHROME_WIN, "Win32", { platform: "Windows", architecture: "x86" })), "windows-x64");
assert.strictEqual(ids(detect(CHROME_WIN, "Win32", { platform: "Windows", architecture: "arm" })), "windows-arm64");
// Linux: the string is trusted
assert.strictEqual(ids(detect(FF_LINUX, "Linux x86_64", null)), "linux-x64");
assert.strictEqual(ids(detect(CHROME_LINUX_ARM, "Linux aarch64", null)), "linux-arm64");
assert.strictEqual(ids(detect("Mozilla/5.0 (X11; Linux) Gecko", "Linux", null)), "linux-both");
// phones and unknown systems: everything
assert.deepStrictEqual(detect(IPHONE, "iPhone", null), { os: null, arch: null });
assert.deepStrictEqual(detect(ANDROID, "Linux armv8l", null), { os: null, arch: null });
assert.strictEqual(ids(detect("", "", null)), "all");
console.log("detect_test: ok");

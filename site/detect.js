// Picks the downloads that fit the visitor's system (task-212). Pure functions, no DOM:
// site/detect_test.js runs them on macOS, Windows, Linux and other user agents with node.
//
// detect(ua, platform, uaData) returns {os, arch}:
//   os    "mac" | "windows" | "linux" | null (phones, tablets, unknown systems)
//   arch  "arm64" | "x64" | null (null: the browser cannot tell, so the page offers both)
// uaData is the result of navigator.userAgentData.getHighEntropyValues(["architecture",
// "bitness", "platform"]) or null. Safari and Firefox on an Apple silicon Mac say "Intel", and
// Windows on ARM says "x64" in the user agent, so only uaData (or an explicit "arm64"/"aarch64"
// in the string) decides the architecture; Linux "x86_64" is trusted.
(function (root) {
  function detect(ua, platform, uaData) {
    ua = ua || "";
    platform = platform || "";
    var os = null;
    var uaOs = uaData && uaData.platform ? String(uaData.platform).toLowerCase() : "";
    if (/android|iphone|ipad|ipod|cros|CrOS/i.test(ua) || uaOs === "android" || uaOs === "chrome os") {
      return { os: null, arch: null };
    }
    if (uaOs === "macos" || /Mac OS X|Macintosh/.test(ua) || /^Mac/.test(platform)) os = "mac";
    else if (uaOs === "windows" || /Windows|Win32|Win64/.test(ua) || /^Win/.test(platform)) os = "windows";
    else if (uaOs === "linux" || /Linux|X11/.test(ua) || /^Linux/.test(platform)) os = "linux";
    if (!os) return { os: null, arch: null };

    var arch = null;
    var a = uaData && uaData.architecture ? String(uaData.architecture).toLowerCase() : "";
    if (a === "arm") arch = "arm64";
    else if (a === "x86") arch = "x64";
    else if (/aarch64|arm64|ARM64/.test(ua) || /aarch64|arm64/i.test(platform)) arch = "arm64";
    else if (os === "linux" && /x86_64|amd64/i.test(ua + " " + platform)) arch = "x64";
    return { os: os, arch: arch };
  }

  var api = { detect: detect };
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.OrbDetect = api;
})(typeof self !== "undefined" ? self : this);

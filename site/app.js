// The download tabs, the macOS permission box and the screenshot zoom (task-212).
// Without JavaScript every system's panel, both architectures, and the plain image links stay.
(function () {
  var OS_NAME = { mac: "macOS", windows: "Windows", linux: "Linux" };
  var SVG = "http://www.w3.org/2000/svg";
  var panels = document.getElementById("panels");
  var tabs = document.querySelector(".tabs");
  var warn = document.getElementById("warn");

  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text) e.textContent = text;
    return e;
  }

  function show(os) {
    Array.prototype.forEach.call(panels.querySelectorAll(".os"), function (p) {
      p.hidden = p.getAttribute("data-os") !== os;
    });
    Array.prototype.forEach.call(tabs.children, function (t) {
      var on = t.getAttribute("data-os") === os;
      t.setAttribute("aria-selected", on ? "true" : "false");
      t.className = on ? "tab on" : "tab";
    });
    warn.hidden = true;
  }

  function init(uaData) {
    var found = OrbDetect.detect(navigator.userAgent, navigator.platform, uaData);
    ["mac", "windows", "linux"].forEach(function (os) {
      var t = el("button", "tab");
      // the system's mark (a symbol of index.html) before its name
      var svg = document.createElementNS(SVG, "svg");
      svg.setAttribute("class", "osi");
      svg.setAttribute("width", "16");
      svg.setAttribute("height", "16");
      svg.setAttribute("aria-hidden", "true");
      var use = document.createElementNS(SVG, "use");
      use.setAttribute("href", "#i-" + os);
      svg.appendChild(use);
      t.appendChild(svg);
      t.appendChild(document.createTextNode(OS_NAME[os]));
      t.type = "button";
      t.setAttribute("role", "tab");
      t.setAttribute("data-os", os);
      t.setAttribute("aria-controls", "os-" + os);
      t.onclick = function () { show(os); };
      tabs.appendChild(t);
    });
    // the detected architecture of each system, both when unsure
    Array.prototype.forEach.call(panels.querySelectorAll(".os"), function (p) {
      var groups = p.querySelectorAll(".arch");
      if (!found.arch || found.os !== p.getAttribute("data-os")) return;
      var other = [];
      Array.prototype.forEach.call(groups, function (g) {
        if (g.getAttribute("data-arch") !== found.arch) { g.hidden = true; other.push(g); }
      });
      var more = el("p", "small");
      var b = el("button", "linklike", "Another processor?");
      b.type = "button";
      b.onclick = function () {
        other.forEach(function (g) { g.hidden = false; });
        more.hidden = true;
      };
      more.appendChild(b);
      p.appendChild(more);
    });
    show(found.os || "mac");
  }

  // the macOS warning, for the file just downloaded (the download itself is the link's)
  panels.addEventListener("click", function (ev) {
    var a = ev.target.closest ? ev.target.closest("a.button") : null;
    if (!a || a.getAttribute("data-os") !== "mac") return;
    var file = decodeURIComponent(a.getAttribute("href").split("/").pop());
    var pkg = a.getAttribute("data-kind") === "pkg";
    warn.textContent = "";
    warn.appendChild(el("h4", null, "macOS will not open " + file + " at first"));
    warn.appendChild(el("p", null, "OpenRB-338 is not signed or notarized with an Apple " +
      "developer certificate, so macOS refuses to open it the first time. The download " +
      "has started; when it is done, allow the file to run, either in the interface or in " +
      "Terminal."));
    warn.appendChild(el("h5", null, "In the interface"));
    var ol = el("ol");
    if (pkg) {
      ol.appendChild(el("li", null, "Open the downloaded " + file + ". macOS refuses."));
      ol.appendChild(el("li", null, "Open System Settings > Privacy & Security, scroll to " +
        "Security and click \"Open Anyway\" next to " + file + ", then confirm. (On macOS 14 " +
        "and older: right-click the .pkg > Open > Open.)"));
      ol.appendChild(el("li", null, "The installer asks for an administrator password: it " +
        "installs OpenRB-338.vst3 and OpenRB-338.clap for all users."));
    } else {
      ol.appendChild(el("li", null, "Open the downloaded " + file + " and drag " +
        "OpenRB-338.app to Applications."));
      ol.appendChild(el("li", null, "Open OpenRB-338.app once. macOS refuses."));
      ol.appendChild(el("li", null, "Open System Settings > Privacy & Security, scroll to " +
        "Security and click \"Open Anyway\" next to OpenRB-338, then confirm. (On macOS 14 " +
        "and older: right-click the app > Open > Open.)"));
    }
    warn.appendChild(ol);
    warn.appendChild(el("h5", null, "In Terminal"));
    warn.appendChild(el("p", null, pkg ? "Remove the quarantine mark from the file, then open " +
      "it:" : "Remove the quarantine mark from the download, and from the app after you " +
      "have copied it to Applications:"));
    var cmds = ["xattr -d com.apple.quarantine ~/Downloads/" + file];
    if (!pkg) cmds.push("xattr -dr com.apple.quarantine /Applications/OpenRB-338.app");
    warn.appendChild(el("pre", null, cmds.join("\n")));
    warn.hidden = false;
  });

  var d = navigator.userAgentData;
  if (d && d.getHighEntropyValues) {
    d.getHighEntropyValues(["architecture", "bitness", "platform"]).then(init, function () {
      init({ platform: d.platform });
    });
  } else init(null);

  // the screenshot zoom: the link's full-size image in an overlay, closed by a click or Esc
  var box = null, opener = null;
  function close() {
    if (!box) return;
    document.body.removeChild(box);
    box = null;
    if (opener && opener.focus) opener.focus();
    opener = null;
  }
  Array.prototype.forEach.call(document.querySelectorAll("a.zoom"), function (a) {
    a.addEventListener("click", function (ev) {
      if (ev.ctrlKey || ev.metaKey || ev.shiftKey || ev.button) return;
      ev.preventDefault();
      close();
      opener = a;
      box = el("div", "lightbox");
      box.setAttribute("role", "dialog");
      box.setAttribute("aria-label", "Screenshot, click to close");
      var img = el("img");
      img.src = a.href;
      img.alt = a.querySelector("img").alt;
      box.appendChild(img);
      var x = el("button", "lightbox-close", "\u00d7");
      x.type = "button";
      x.setAttribute("aria-label", "Close");
      box.appendChild(x);
      box.onclick = close;
      document.body.appendChild(box);
      x.focus();
    });
  });
  document.addEventListener("keydown", function (ev) {
    if (ev.key === "Escape") close();
  });
})();

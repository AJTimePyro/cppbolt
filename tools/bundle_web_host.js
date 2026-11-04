#!/usr/bin/env node
// Wrap the compiled npm host module in a UMD file so both the browser
// playground and Node.js test harnesses use the single implementation in
// npm/src/host.ts.
//
// Usage: node tools/bundle_web_host.js <compiled-host.js> <output-file>

"use strict";

const fs = require("fs");

const [input, output] = process.argv.slice(2);
if (!input || !output) {
  console.error("Usage: bundle_web_host.js <compiled-host.js> <output-file>");
  process.exit(1);
}

const compiled = fs.readFileSync(input, "utf8");

const out = `(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory();
  } else {
    var api = factory();
    root.CppBoltHost = api;
    root.createWasmHost = api.createWasmHost;
  }
}(typeof self !== "undefined" ? self : this, function () {
  "use strict";
  var module = { exports: {} };
  var exports = module.exports;
${compiled}
  return module.exports;
}));
`;

fs.writeFileSync(output, out);

//===----------------------------------------------------------------------===//
// Distributed under the 3-Clause BSD License. See accompanying file LICENSE or
// copy at https://opensource.org/licenses/BSD-3-Clause).
// SPDX-License-Identifier: BSD-3-Clause
//===----------------------------------------------------------------------===//

// -----------------------------------------------------------------------------
// Detect the dominant newline character of a string, from  'detect-newline'
// https://github.com/sindresorhus/detect-newline
//
// MIT License
// -----------------------------------------------------------------------------
function detectNewline(string) {
  if (typeof string !== 'string') {
    throw new TypeError('Expected a string');
  }

  const newlines = string.match(/(?:\r?\n)/g) || [];

  if (newlines.length === 0) {
    return;
  }

  const crlf = newlines.filter(newline => newline === '\r\n').length;
  const lf = newlines.length - crlf;

  return crlf > lf ? '\r\n' : '\n';
}

function detectNewlineGraceful(string) {
  return (typeof string === 'string' && detectNewline(string)) || '\n';
}
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// Version reader/updated for standard-version that uses the mETA information in
// the CmakeLists.txt file
// -----------------------------------------------------------------------------

const major_rex = /set\(META_VERSION_MAJOR\s+\"(\d+)\"\)/;
const minor_rex = /set\(META_VERSION_MINOR\s+\"(\d+)\"\)/;
const patch_rex = /set\(META_VERSION_PATCH\s+\"(\d+)\"\)/;

module.exports.readVersion = function (contents) {
  var major = null, minor = null, patch = null;

  const lines = contents.split(/\r?\n/);
  for (let index in lines) {
    let line = lines[index];
    var match = null;
    if (major == null) {
      var match = major_rex.exec(line);
      if (match != null) {
        major = match[1];
      }
    }
    if (match == null && minor == null) {
      var match = minor_rex.exec(line);
      if (match != null) {
        minor = match[1];
      }
    }
    if (match == null && patch == null) {
      var match = patch_rex.exec(line);
      if (match != null) {
        patch = match[1];
      }
    }
    if (major != null && minor != null && patch != null) break;
  };

  if (major == null)
    console.error("Your CmakeLists.txt is missing META_VERSION_MAJOR variable!");
  if (minor == null)
    console.error("Your CmakeLists.txt is missing META_VERSION_MINOR variable!");
  if (patch == null)
    console.error("Your CmakeLists.txt is missing META_VERSION_PATCH variable!");

  return major + "." + minor + "." + patch;
}

module.exports.writeVersion = function (contents, version) {
  var [major, minor, patch] = version.split(".");
  var newContents = [];

  const lines = contents.split(/\r?\n/);
  lines.forEach(line => {
    var newLine = line.replace(major_rex, "set(META_VERSION_MAJOR       \"" + major + "\")")
      .replace(minor_rex, "set(META_VERSION_MINOR       \"" + minor + "\")")
      .replace(patch_rex, "set(META_VERSION_PATCH       \"" + patch + "\")");
    newContents.push(newLine);
  });

  let newline = detectNewlineGraceful(contents)
  return newContents.join(newline);
}

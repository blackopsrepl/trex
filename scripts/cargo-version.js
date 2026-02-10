// Version updater for Cargo.toml
// Used by commit-and-tag-version to read/write the version field

module.exports.readVersion = function (contents) {
  const match = contents.match(/^version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : null;
};

module.exports.writeVersion = function (contents, version) {
  return contents.replace(
    /^(version\s*=\s*")[^"]+(")/m,
    `$1${version}$2`
  );
};

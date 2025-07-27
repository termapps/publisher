function getBinaryPath() {
  // Determine package name for this platform
  const platformSpecificPackageName =
    BINARY_DISTRIBUTION_PACKAGES[`${process.platform}-${process.arch}`];

  try {
    // Resolving will fail if the optionalDependency was not installed
    return require.resolve(`${platformSpecificPackageName}/bin/${binaryName}`);
  } catch (e) {
    return require("path").join(__dirname, binaryName);
  }
}

require("child_process").execFileSync(getBinaryPath(), process.argv.slice(2), {
  stdio: "inherit",
});

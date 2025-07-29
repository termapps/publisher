const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const zlib = require("zlib");
const https = require("https");

const { familySync } = require("detect-libc");

const {
  BINARY_DISTRIBUTION_PACKAGES,
  BINARY_DISTRIBUTION_VERSION,
  BINARY_NAME,
} = require("./constants");

const libc = familySync();

const platformSpecificPackageName =
  BINARY_DISTRIBUTION_PACKAGES[
    `${process.platform}-${process.arch}${libc ? `-${libc}` : ""}`
  ];

if (!platformSpecificPackageName) {
  throw new Error("Platform not supported!");
}

// Compute the path of packaged binary
const packagedBinaryPath = `${platformSpecificPackageName}/bin/${BINARY_NAME}`;

// Compute the path we want to emit the fallback binary to
const fallbackBinaryPath = path.join(__dirname, BINARY_NAME);

function makeRequest(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (response.statusCode >= 200 && response.statusCode < 300) {
          const chunks = [];
          response.on("data", (chunk) => chunks.push(chunk));
          response.on("end", () => {
            resolve(Buffer.concat(chunks));
          });
        } else if (
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          // Follow redirects
          makeRequest(response.headers.location).then(resolve, reject);
        } else {
          reject(
            new Error(
              `npm responded with status code ${response.statusCode} when downloading the package!`
            )
          );
        }
      })
      .on("error", (error) => {
        reject(error);
      });
  });
}

function extractFileFromTarball(tarballBuffer, filepath) {
  // Tar archives are organized in 512 byte blocks.
  // Blocks can either be header blocks or data blocks.
  // Header blocks contain file names of the archive in the first 100 bytes, terminated by a null byte.
  // The size of a file is contained in bytes 124-135 of a header block and in octal format.
  // The following blocks will be data blocks containing the file.
  let offset = 0;
  while (offset < tarballBuffer.length) {
    const header = tarballBuffer.subarray(offset, offset + 512);
    offset += 512;

    const fileName = header.toString("utf-8", 0, 100).replace(/\0.*/g, "");
    const fileSize = parseInt(
      header.toString("utf-8", 124, 136).replace(/\0.*/g, ""),
      8
    );

    if (fileName === filepath) {
      return tarballBuffer.subarray(offset, offset + fileSize);
    }

    // Clamp offset to the uppoer multiple of 512
    offset = (offset + fileSize + 511) & ~511;
  }
}

async function downloadBinaryFromNpm() {
  const urlName = platformSpecificPackageName.replace("/", "%2F");
  const name = platformSpecificPackageName.split("/").pop();

  // Download the tarball of the right binary distribution package
  const tarballDownloadBuffer = await makeRequest(
    `https://registry.npmjs.org/${urlName}/-/${name}-${BINARY_DISTRIBUTION_VERSION}.tgz`
  );

  const tarballBuffer = zlib.unzipSync(tarballDownloadBuffer);

  // Extract binary from package and write to disk
  fs.writeFileSync(
    fallbackBinaryPath,
    extractFileFromTarball(tarballBuffer, `package/bin/${BINARY_NAME}`),
    { mode: 0o755 } // Make binary file executable
  );
}

function isPlatformSpecificPackageInstalled() {
  try {
    // Resolving will fail if the optionalDependency was not installed
    require.resolve(packagedBinaryPath);
    return true;
  } catch (e) {
    return false;
  }
}

function isBinaryDownloaded() {
  try {
    // Check if the fallback binary exists
    return fs.existsSync(fallbackBinaryPath);
  } catch (e) {
    return false;
  }
}

function getBinaryPath() {
  try {
    // Resolving will fail if the optionalDependency was not installed
    return require.resolve(packagedBinaryPath);
  } catch (e) {
    return fallbackBinaryPath;
  }
}

async function maybeInstall() {
  if (!isPlatformSpecificPackageInstalled() && !isBinaryDownloaded()) {
    await downloadBinaryFromNpm();
  }
}

function run() {
  const binaryPath = getBinaryPath();

  if (!fs.existsSync(binaryPath)) {
    throw new Error(
      `Binary not found at ${binaryPath}. Please ensure the binary is installed correctly.`
    );
  }

  if (
    isPlatformSpecificPackageInstalled() &&
    !["win32", "cygwin"].includes(process.platform)
  ) {
    // Ensure the binary is executable
    fs.chmodSync(binaryPath, 0o755);
  }

  try {
    execFileSync(binaryPath, process.argv.slice(2), {
      stdio: "inherit",
    });
  } catch (error) {
    if (error.status === undefined) {
      throw error;
    }

    process.exit(error.status);
  }
}

module.exports = {
  maybeInstall,
  run,
};

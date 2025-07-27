use clap::Parser;
use tracing::instrument;

use crate::{config::AppConfig, error::Result, generate::write_lines};

/// Generates CI pipeline to build release artifacts
#[derive(Debug, Parser)]
pub struct CI {}

impl CI {
    #[instrument(name = "ci", skip_all)]
    pub fn run(self, info: &AppConfig) -> Result {
        let AppConfig {
            name: cli_name,
            repository,
            ..
        } = info;

        let name = if repository.split('/').next_back().unwrap() == cli_name {
            "${{ github.event.repository.name }}"
        } else {
            cli_name
        };

        write_lines(".github/workflows/release.yml", || {
            vec![
                format!("name: Release"),
                format!("on:"),
                format!("  push:"),
                format!("    tags: [v*]"),
                format!("env:"),
                format!("  NAME: {name}"),
                format!("defaults:"),
                format!("  run:"),
                format!("    shell: bash"),
                format!("jobs:"),
                format!("  create-release:"),
                format!("    name: Create release"),
                format!("    runs-on: ubuntu-latest"),
                format!("    outputs:"),
                format!("      upload_url: ${{{{ steps.create-release.outputs.upload_url }}}}"),
                format!("    steps:"),
                format!("      - name: Create Release"),
                format!("        id: create-release"),
                format!("        uses: actions/create-release@v1"),
                format!("        env:"),
                format!("          GITHUB_TOKEN: ${{{{ github.token }}}}"),
                format!("        with:"),
                format!("          tag_name: ${{{{ github.ref }}}}"),
                format!("          release_name: ${{{{ github.ref }}}}"),
                format!("  read-version:"),
                format!("    name: Read version"),
                format!("    runs-on: ubuntu-latest"),
                format!("    outputs:"),
                format!(
                    "      source_name: ${{{{ env.NAME }}}}-${{{{ steps.version.outputs.VERSION }}}}"
                ),
                format!("    steps:"),
                format!("      - name: Read version"),
                format!("        id: version"),
                format!("        env:"),
                format!("          REF: ${{{{ github.ref }}}}"),
                format!("        run: echo \"VERSION=${{REF/refs\\/tags\\//}}\" >> $GITHUB_OUTPUT"),
                format!("  source-checksum-upload:"),
                format!("    name: Source checksum upload"),
                format!("    needs: [create-release, read-version]"),
                format!("    runs-on: ubuntu-latest"),
                format!("    steps:"),
                format!("      - name: Calculate checksum"),
                format!("        run: |"),
                format!(
                    "          curl -sL ${{{{ github.event.repository.html_url }}}}/archive/${{{{ github.ref }}}}.zip > upload.zip"
                ),
                format!("          echo $(sha256sum upload.zip | cut -d ' ' -f 1) > sha256sum.txt"),
                format!("      - name: Upload checksums"),
                format!("        uses: actions/upload-release-asset@v1"),
                format!("        env:"),
                format!("          GITHUB_TOKEN: ${{{{ github.token }}}}"),
                format!("        with:"),
                format!("          upload_url: ${{{{ needs.create-release.outputs.upload_url }}}}"),
                format!("          asset_path: ./sha256sum.txt"),
                format!(
                    "          asset_name: ${{{{ needs.read-version.outputs.source_name }}}}_sha256sum.txt"
                ),
                format!("          asset_content_type: text/plain"),
                format!("  build-upload:"),
                format!("    name: Build & Upload"),
                format!("    needs: [create-release, read-version]"),
                format!("    strategy:"),
                format!("      fail-fast: false"),
                format!("      matrix:"),
                format!("        include:"),
                format!("          - os: macos-13"),
                format!("            target: x86_64-apple-darwin"),
                format!("          - os: macos-latest"),
                format!("            target: aarch64-apple-darwin"),
                format!("          - os: ubuntu-latest"),
                format!("            target: x86_64-unknown-linux-gnu"),
                format!("          - os: ubuntu-latest"),
                format!("            target: i686-unknown-linux-gnu"),
                format!("          - os: ubuntu-latest"),
                format!("            target: x86_64-unknown-linux-musl"),
                format!("          - os: windows-latest"),
                format!("            target: x86_64-pc-windows-msvc"),
                format!("          - os: windows-latest"),
                format!("            target: i686-pc-windows-msvc"),
                format!("    runs-on: ${{{{ matrix.os }}}}"),
                format!("    steps:"),
                format!("      - name: Install rust"),
                format!("        uses: dtolnay/rust-toolchain@1.88.0"),
                format!("        with:"),
                format!("          target: ${{{{ matrix.target }}}}"),
                format!("      - name: Install linker"),
                format!("        if: matrix.os == 'ubuntu-latest'"),
                format!("        run: |"),
                format!("          sudo apt-get update"),
                format!("          sudo apt-get install musl-tools gcc-multilib"),
                format!("      - name: Checkout"),
                format!("        uses: actions/checkout@v4"),
                format!("      - name: Build"),
                format!("        run: cargo build --target ${{{{ matrix.target }}}} --release"),
                format!("      - name: Set variables"),
                format!("        id: vars"),
                format!("        env:"),
                format!(
                    "          BUILD_NAME: ${{{{ needs.read-version.outputs.source_name }}}}-${{{{ matrix.target }}}}"
                ),
                format!("        run: echo \"BUILD_NAME=$BUILD_NAME\" >> $GITHUB_OUTPUT"),
                format!("      - name: Ready artifacts"),
                format!("        run: |"),
                format!("          mkdir upload"),
                format!(
                    "          cp target/${{{{ matrix.target }}}}/release/$NAME LICENSE upload"
                ),
                format!("      - name: Compress artifacts"),
                format!("        uses: vimtor/action-zip@v1"),
                format!("        with:"),
                format!("          files: upload/"),
                format!("          recursive: true"),
                format!("          dest: upload.zip"),
                format!("      - name: Upload artifacts"),
                format!("        uses: actions/upload-release-asset@v1"),
                format!("        env:"),
                format!("          GITHUB_TOKEN: ${{{{ github.token }}}}"),
                format!("        with:"),
                format!("          upload_url: ${{{{ needs.create-release.outputs.upload_url }}}}"),
                format!("          asset_path: ./upload.zip"),
                format!("          asset_name: ${{{{ steps.vars.outputs.BUILD_NAME }}}}.zip"),
                format!("          asset_content_type: application/zip"),
                format!("      - name: Calculate checksum"),
                format!("        if: runner.os == 'macOS'"),
                format!(
                    "        run: echo $(shasum -a 256 upload.zip | cut -d ' ' -f 1) > sha256sum.txt"
                ),
                format!("      - name: Calculate checksum"),
                format!("        if: runner.os != 'macOS'"),
                format!(
                    "        run: echo $(sha256sum upload.zip | cut -d ' ' -f 1) > sha256sum.txt"
                ),
                format!("      - name: Upload checksums"),
                format!("        uses: actions/upload-release-asset@v1"),
                format!("        env:"),
                format!("          GITHUB_TOKEN: ${{{{ github.token }}}}"),
                format!("        with:"),
                format!("          upload_url: ${{{{ needs.create-release.outputs.upload_url }}}}"),
                format!("          asset_path: ./sha256sum.txt"),
                format!(
                    "          asset_name: ${{{{ steps.vars.outputs.BUILD_NAME }}}}_sha256sum.txt"
                ),
                format!("          asset_content_type: text/plain"),
            ]
        })?;

        Ok(())
    }
}

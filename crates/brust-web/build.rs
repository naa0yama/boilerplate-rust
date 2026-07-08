//! Build script: runs pnpm/Tailwind CSS pipeline and emits static assets to `$OUT_DIR`.
//! When `CSS_PIPELINE_STUB=1` or pnpm is absent, writes empty stub files instead.

use anyhow::Context as _;
use std::error::Error;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::var("OUT_DIR")?;

    // Rerun triggers — emitted in both modes
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=../../pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=src/styles");
    println!("cargo:rerun-if-env-changed=CSS_PIPELINE_STUB");

    let pnpm_available = Command::new("pnpm")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    // Also stub when lockfile is absent to allow initial `cargo check` before `pnpm install`.
    let lockfile_exists = Path::new("../../pnpm-lock.yaml").exists();
    let stub_mode =
        std::env::var("CSS_PIPELINE_STUB").is_ok() || !pnpm_available || !lockfile_exists;

    if stub_mode {
        println!("cargo:warning=CSS pipeline stub mode: emitting empty assets.");
        write_stubs(&out_dir)?;
        return Ok(());
    }

    // 1. pnpm install --frozen-lockfile
    let status = Command::new("pnpm")
        .args(["install", "--frozen-lockfile"])
        .status()
        .context("failed to spawn pnpm install")?;
    if !status.success() {
        return Err(
            "pnpm install failed. Run `pnpm install` locally and commit pnpm-lock.yaml.".into(),
        );
    }

    // 2. Tailwind CSS compile
    let out_css = Path::new(&out_dir).join("app.css");
    let status = Command::new("pnpm")
        .args(["exec", "tailwindcss", "-i", "src/styles/app.css", "-o"])
        .arg(&out_css)
        .arg("--minify")
        .status()
        .context("failed to spawn tailwindcss")?;
    if !status.success() {
        return Err("tailwindcss compilation failed.".into());
    }

    // 3. Template rerun-if-changed (walkdir)
    for entry in walkdir::WalkDir::new("templates")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    // 4. Font copy: @fontsource woff2 → $OUT_DIR/fonts/<family>/
    let fonts_out = Path::new(&out_dir).join("fonts");
    for family in ["ibm-plex-sans-jp", "ibm-plex-mono"] {
        let src = Path::new("node_modules/@fontsource")
            .join(family)
            .join("files");
        let dst = fonts_out.join(family);
        std::fs::create_dir_all(&dst)
            .with_context(|| format!("create_dir_all {}", dst.display()))?;
        for entry in walkdir::WalkDir::new(&src)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|x| x == "woff2"))
        {
            let font_dst = dst.join(entry.file_name());
            std::fs::copy(entry.path(), &font_dst).with_context(|| {
                format!("copy {} → {}", entry.path().display(), font_dst.display())
            })?;
        }
        println!("cargo:rerun-if-changed={}", src.display());
    }

    // 5. HTMX copy
    let htmx_src = Path::new("node_modules/htmx.org/dist/htmx.min.js");
    let htmx_dst = Path::new(&out_dir).join("htmx.min.js");
    std::fs::copy(htmx_src, &htmx_dst)
        .with_context(|| format!("copy htmx.min.js → {}", htmx_dst.display()))?;
    println!("cargo:rerun-if-changed=node_modules/htmx.org/dist/htmx.min.js");

    // 6. GIT_HASH (workspace layout: ../../.git/)
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/");
    println!("cargo:rerun-if-changed=../../.git/packed-refs");
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| String::from("unknown"), |s| s.trim().to_owned());
    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    Ok(())
}

fn write_stubs(out_dir: &str) -> anyhow::Result<()> {
    let base = Path::new(out_dir);
    std::fs::write(base.join("app.css"), b"")
        .with_context(|| format!("write {out_dir}/app.css"))?;
    std::fs::write(base.join("htmx.min.js"), b"")
        .with_context(|| format!("write {out_dir}/htmx.min.js"))?;
    std::fs::create_dir_all(base.join("fonts"))
        .with_context(|| format!("create_dir_all {out_dir}/fonts"))?;
    Ok(())
}

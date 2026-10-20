mod gen;

use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Result};
use bindgen::{
    callbacks::{DeriveInfo, ParseCallbacks},
    Builder,
};
use clap::Parser;
use directories::UserDirs;
use gen::{Arch, Chip};
use log::LevelFilter;

/// Files in `c/include` that esp-wireless-drivers-3rdparty doesn't provide.
const LOCAL_HEADERS: &[&str] = &[
    "include.h",
    "esp_netif.h",
    "hal/efuse_hal.h",
    "hal/gpio_types.h",
];

#[derive(Parser)]
struct Cli {
    /// Chips whose libraries, headers and bindings to update. Defaults to all chips.
    chips: Vec<String>,
    /// esp-wireless-drivers-3rdparty checkout. Defaults to `../esp-wireless-drivers-3rdparty`.
    #[arg(long)]
    drivers: Option<PathBuf>,
    /// Skip regenerating `src/include.rs`.
    #[arg(long)]
    skip_bindings: bool,
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_module("xtask", LevelFilter::Info)
        .init();

    let args = Cli::parse();

    // The directory containing the cargo manifest for the 'xtask' package is a
    // subdirectory within the cargo workspace:
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    let drivers = args
        .drivers
        .or_else(|| {
            Some(workspace.parent()?.join("esp-wireless-drivers-3rdparty")).filter(|p| p.is_dir())
        })
        .ok_or_else(|| anyhow!("can't find esp-wireless-drivers-3rdparty, pass `--drivers`"))?;
    if !drivers.is_dir() {
        anyhow::bail!("{} is not a directory", drivers.display());
    }

    let on_disk = gen::crate_dirs(&workspace)?;
    let selected = if args.chips.is_empty() {
        let mut all = on_disk.clone();
        for name in gen::driver_chips(&drivers) {
            if !all.contains(&name) {
                all.push(name);
            }
        }
        all.sort();
        all
    } else {
        args.chips
    };
    let mut names = on_disk;
    for name in &selected {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort();

    let chips = names
        .iter()
        .map(|name| gen::load(name, &drivers))
        .collect::<Result<Vec<_>>>()?;

    write_file(
        &workspace.join("Cargo.toml"),
        gen::workspace_toml(&fs::read_to_string(workspace.join("Cargo.toml"))?, &chips)?,
    )?;
    write_file(
        &workspace.join("README.md"),
        gen::root_readme(&fs::read_to_string(workspace.join("README.md"))?, &chips)?,
    )?;
    let include_h_changed = write_file(
        &workspace.join("c/include/include.h"),
        gen::include_h(
            &fs::read_to_string(workspace.join("c/include/include.h"))?,
            &chips,
        )?,
    )?;

    for chip in &chips {
        let dir = workspace.join(chip.crate_name());
        let current = fs::read_to_string(dir.join("Cargo.toml")).ok();
        write_file(
            &dir.join("Cargo.toml"),
            gen::cargo_toml(chip, current.as_deref()),
        )?;
        write_file(&dir.join("build.rs"), gen::build_rs(chip)?)?;
        write_file(&dir.join(".cargo/config.toml"), gen::cargo_config(chip))?;
        write_file(&dir.join("README.md"), gen::readme(chip))?;
        for (file, content) in gen::STATICS {
            write_file(&dir.join(file), content)?;
        }
    }

    let shared_changed = sync_dir(
        &drivers.join("include"),
        &workspace.join("c/include"),
        |rel| {
            let mut parts = rel.components();
            let chip_dir = parts
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .is_some_and(|name| name.starts_with("esp32"))
                && parts.next().is_some();
            chip_dir || LOCAL_HEADERS.iter().any(|h| rel == Path::new(h))
        },
    )? || include_h_changed;

    let selected_chips: Vec<&Chip> = chips
        .iter()
        .filter(|chip| selected.iter().any(|n| n == &chip.name))
        .collect();

    for chip in &selected_chips {
        sync_dir(
            &drivers.join("libs").join(&chip.name),
            &workspace.join(chip.crate_name()).join("libs"),
            |_| false,
        )?;
        sync_dir(
            &drivers.join("include").join(&chip.name),
            &workspace.join("c/include").join(&chip.name),
            |_| false,
        )?;
    }

    if !args.skip_bindings {
        // Determine the $HOME directory, and subsequently the Espressif tools
        // directory:
        let home = UserDirs::new().unwrap().home_dir().to_path_buf();
        let mut tools = home.join(".espressif").join("tools");

        if !tools.join("xtensa-esp-elf").exists() {
            println!("Tools not found in home - using ESP_TOOLS_DIR env variable");
            tools = PathBuf::from(std::env::var("ESP_TOOLS_DIR")?);
        }

        for chip in &selected_chips {
            generate_bindings_for_chip(&chip.name, chip.arch(), &workspace, &tools, chip.tool())?;
        }
    }

    let stale: Vec<&str> = chips
        .iter()
        .filter(|chip| args.skip_bindings || !selected.contains(&chip.name))
        .map(|chip| chip.name.as_str())
        .collect();
    if shared_changed && !stale.is_empty() {
        log::warn!(
            "shared headers in c/include changed, regenerate the bindings of {} too",
            stale.join(", ")
        );
    }

    Ok(())
}

/// Writes `content` to `path` unless it's already there. Returns whether the file changed.
fn write_file(path: &Path, content: impl AsRef<[u8]>) -> Result<bool> {
    let bytes = content.as_ref();
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(false);
    }
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, bytes)?;
    Ok(true)
}

/// Mirrors `from` into `to`, leaving `ignore`d paths alone. Returns whether anything changed.
fn sync_dir(from: &Path, to: &Path, ignore: impl Fn(&Path) -> bool) -> Result<bool> {
    let mut changed = false;
    let mut expected = BTreeSet::new();
    for file in files_in(from) {
        if ignore(&file) {
            continue;
        }
        expected.insert(file.clone());
        changed |= write_file(&to.join(&file), fs::read(from.join(&file))?)?;
    }
    if to.is_dir() {
        for file in files_in(to) {
            if !ignore(&file) && !expected.contains(&file) {
                fs::remove_file(to.join(&file))?;
                changed = true;
            }
        }
    }
    Ok(changed)
}

fn files_in(dir: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with('.'))
                {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else if path.is_file() {
                    out.push(path.strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn generate_bindings_for_chip(
    chip: &str,
    arch: Arch,
    workspace: &Path,
    tools: &Path,
    tool: &str,
) -> Result<()> {
    let sysroot_path = tools.join(format!("{tool}/esp-15.2.0_20251204/{tool}"));
    let include_path = sysroot_path.join(format!("{tool}/include"));
    let c_path = workspace.join("c");
    let crate_path = workspace.join(format!("esp-wifi-sys-{chip}"));

    // Generate the bindings using `bindgen`:
    log::info!("Generating bindings for: {chip}");
    let bindings = Builder::default()
        .clang_args([
            &format!("-DCONFIG_IDF_TARGET_{}", chip.to_uppercase()),
            // Don't shadow `c/include` with header copies: they go stale on ESP-IDF
            // updates and break the ABI (e.g. wrong `ESP_WIFI_MAX_CONN_NUM`).
            &format!(
                "-I{}",
                c_path
                    .join("include")
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                c_path
                    .join("include")
                    .join("local")
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                include_path
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                c_path
                    .join("include")
                    .join(chip)
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "--sysroot={}",
                sysroot_path
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "--target={}",
                if arch == Arch::Xtensa {
                    "xtensa"
                } else {
                    "riscv32"
                }
            ),
        ])
        .ctypes_prefix("crate::c_types")
        .derive_debug(false)
        .parse_callbacks(Box::new(AddPartialEq))
        .header(c_path.join("include/include.h").to_string_lossy())
        .layout_tests(false)
        .raw_line("#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code,improper_ctypes)]")
        .use_core()
        .generate()
        .map_err(|_| anyhow!("Failed to generate bindings"))?;

    // Write out the bindings to the appropriate path:
    let path = crate_path.join("src").join("include.rs");
    log::info!("Writing out bindings to: {}", path.display());
    bindings.write_to_file(&path)?;

    // We additionally need to implement a `Send` for a couple types:
    let mut file = File::options().append(true).open(&path)?;
    writeln!(
        file,
        "\nunsafe impl Sync for wifi_init_config_t {{}}\nunsafe impl Sync for wifi_osi_funcs_t {{}}"
    )?;

    // Format the bindings:
    Command::new("rustfmt")
        .arg(path.to_string_lossy().to_string())
        .output()?;

    Ok(())
}

// Types for which we want derive PartialEq added
const PARTIALEQ_TYPES: &[&str] = &[
    "wifi_ap_config_t",
    "wifi_sta_config_t",
    "wifi_scan_threshold_t",
    "wifi_pmf_config_t",
    "wifi_bss_max_idle_config_t",
];

#[derive(Debug)]
struct AddPartialEq;

impl ParseCallbacks for AddPartialEq {
    fn add_derives(&self, info: &DeriveInfo<'_>) -> Vec<String> {
        if PARTIALEQ_TYPES.contains(&info.name) {
            vec!["PartialEq".to_string()]
        } else {
            vec![]
        }
    }
}

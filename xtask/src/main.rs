use std::{fs::File, io::Write, path::PathBuf, process::Command};

use anyhow::{anyhow, Result};
use bindgen::Builder;
use directories::UserDirs;
use log::LevelFilter;

#[derive(Debug, PartialEq)]
enum Arch {
    RiscV,
    Xtensa,
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_module("xtask", LevelFilter::Info)
        .init();

    // The directory containing the cargo manifest for the 'xtask' package is a
    // subdirectory within the cargo workspace:
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    // Determine the $HOME directory, and subsequently the Espressif tools
    // directory:
    let home = UserDirs::new().unwrap().home_dir().to_path_buf();
    let mut tools = home.join(".espressif").join("tools");

    if !tools.join("xtensa-esp-elf").exists() {
        println!("Tools not found in home - using ESP_TOOLS_DIR env variable");
        tools = PathBuf::from(std::env::var("ESP_TOOLS_DIR")?);
    }

    generate_bindings_for_chip(
        "esp32",
        Arch::Xtensa,
        &workspace,
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/xtensa-esp-elf/include/"),
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32c2",
        Arch::RiscV,
        &workspace,
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/riscv32-esp-elf/include/"),
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32c3",
        Arch::RiscV,
        &workspace,
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/riscv32-esp-elf/include/"),
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32s2",
        Arch::Xtensa,
        &workspace,
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/xtensa-esp-elf/include/"),
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32s3",
        Arch::Xtensa,
        &workspace,
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/xtensa-esp-elf/include/"),
        tools.join("xtensa-esp-elf/esp-13.2.0_20230928/xtensa-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32c6",
        Arch::RiscV,
        &workspace,
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/riscv32-esp-elf/include/"),
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/"),
    )?;

    generate_bindings_for_chip(
        "esp32h2",
        Arch::RiscV,
        &workspace,
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/riscv32-esp-elf/include/"),
        tools.join("riscv32-esp-elf/esp-13.2.0_20230928/riscv32-esp-elf/"),
    )?;

    Ok(())
}

fn generate_bindings_for_chip(
    chip: &str,
    arch: Arch,
    workspace: &PathBuf,
    include_path: PathBuf,
    sysroot_path: PathBuf,
) -> Result<()> {
    let sys_path = workspace.join("esp-wifi-sys");

    println!(
        "{}",
        sys_path
            .join("include")
            .join(chip)
            .join("soc")
            .display()
            .to_string()
            .replace("/", "\\")
    );

    // Generate the bindings using `bindgen`:
    log::info!("Generating bindings for: {chip}");
    let bindings = Builder::default()
        .clang_args([
            &format!("-DCONFIG_IDF_TARGET_{}", chip.to_uppercase()),
            &format!(
                "-I{}",
                sys_path
                    .join("headers")
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                sys_path
                    .join("headers")
                    .join(chip)
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                sys_path
                    .join("headers")
                    .join("local")
                    .display()
                    .to_string()
                    .replace("\\", "/")
                    .replace("//?/C:", "")
            ),
            &format!(
                "-I{}",
                sys_path
                    .join("include")
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
                sys_path
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
        .header(sys_path.join("include/include.h").to_string_lossy())
        .layout_tests(false)
        .raw_line("#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code)]")
        .use_core()
        .generate()
        .map_err(|_| anyhow!("Failed to generate bindings"))?;

    // Write out the bindings to the appropriate path:
    let path = sys_path
        .join("src")
        .join("include")
        .join(format!("{chip}.rs"));
    log::info!("Writing out bindings to: {}", path.display());
    bindings.write_to_file(&path)?;

    // We additionally need to implement a `Send` for a couple types:
    let mut file = File::options().append(true).open(&path)?;
    writeln!(
        file,
        "\n{}\n{}",
        "unsafe impl Sync for wifi_init_config_t {}", "unsafe impl Sync for wifi_osi_funcs_t {}"
    )?;

    // Format the bindings:
    Command::new("rustfmt")
        .arg(path.to_string_lossy().to_string())
        .output()?;

    Ok(())
}

//! Chip facts, and the crate files `sync` writes from them.
//!
//! The target, CPU capabilities and radios in esp-wireless-drivers-3rdparty's
//! `include/<chip>/sdkconfig.h`, and the archives in `libs/<chip>/`, decide every per-crate
//! file. `xtask/templates/` holds the files that are the same for every chip.

use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};

/// `(feature, sdkconfig.h symbol, comment in Cargo.toml)`.
const RADIOS: &[(&str, &str, &str)] = &[
    (
        "wifi",
        "CONFIG_SOC_WIFI_SUPPORTED",
        "Link the Wi-Fi driver libraries",
    ),
    (
        "bt",
        "CONFIG_SOC_BT_SUPPORTED",
        "Link the Bluetooth controller libraries",
    ),
    (
        "ieee802154",
        "CONFIG_SOC_IEEE802154_SUPPORTED",
        "Link the libraries needed by the IEEE 802.15.4 driver",
    ),
];

/// `#include` in `c/include/include.h`, and the feature whose chips go in the `#if` above it.
const FEATURE_HEADERS: &[(&str, &str)] = &[("esp_bt.h", "bt"), ("esp_coex_i154.h", "ieee802154")];

/// Libraries that are shipped but not linked. esp-radio doesn't support BR/EDR.
const UNLINKED_LIBS: &[&str] = &["bredr_app"];

/// `(library, features that enable it; empty means always linked)`.
const LIBS: &[(&str, &[&str])] = &[
    ("ble_app", &["bt"]),
    ("btdm_common", &["bt"]),
    ("btbb", &["bt", "ieee802154"]),
    ("btdm_app", &["bt"]),
    ("coexist", &["wifi", "bt", "ieee802154"]),
    ("core", &["wifi"]),
    ("espnow", &["wifi"]),
    ("mesh", &["wifi"]),
    ("net80211", &["wifi"]),
    ("phy", &[]),
    ("pp", &["wifi"]),
    ("rtc", &[]),
    ("smartconfig", &["wifi"]),
    ("wapi", &["wifi"]),
    ("wpa_supplicant", &["wifi"]),
    ("printf", &[]),
    ("regulatory", &["wifi"]),
];

/// Files copied into every crate unchanged.
pub(crate) const STATICS: &[(&str, &str)] = &[
    ("src/lib.rs", include_str!("../templates/src/lib.rs")),
    (
        "src/c_types.rs",
        include_str!("../templates/src/c_types.rs"),
    ),
    ("src/fmt.rs", include_str!("../templates/src/fmt.rs")),
];

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Arch {
    RiscV,
    Xtensa,
}

#[derive(Clone)]
pub(crate) struct Chip {
    pub(crate) name: String,
    pub(crate) target: String,
    radios: Vec<&'static str>,
    libs: Vec<String>,
}

impl Chip {
    pub(crate) fn crate_name(&self) -> String {
        format!("esp-wifi-sys-{}", self.name)
    }

    pub(crate) fn arch(&self) -> Arch {
        if self.target.starts_with("xtensa") {
            Arch::Xtensa
        } else {
            Arch::RiscV
        }
    }

    pub(crate) fn tool(&self) -> &'static str {
        match self.arch() {
            Arch::RiscV => "riscv32-esp-elf",
            Arch::Xtensa => "xtensa-esp-elf",
        }
    }

    fn has(&self, feature: &str) -> bool {
        self.radios.contains(&feature)
    }
}

/// `esp32c6` -> `ESP32-C6`.
fn display_name(chip: &str) -> String {
    match chip.strip_prefix("esp32") {
        Some("") => "ESP32".to_string(),
        Some(variant) => format!("ESP32-{}", variant.to_uppercase()),
        None => chip.to_uppercase(),
    }
}

/// Every `esp-wifi-sys-*` directory, sorted.
pub(crate) fn crate_dirs(workspace: &Path) -> Result<Vec<String>> {
    let mut names = fs::read_dir(workspace)?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            Some(name.strip_prefix("esp-wifi-sys-")?.to_string())
        })
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}

/// Whether `source` has `#define <name> 1`.
fn defines(source: &str, name: &str) -> bool {
    source.lines().any(|line| {
        line.trim_start()
            .strip_prefix("#define ")
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix(name))
            .and_then(|rest| rest.strip_prefix(char::is_whitespace))
            .is_some_and(|value| matches!(value.trim(), "1" | "(1)"))
    })
}

/// The chip's target and radios. Fails when the drivers tree doesn't know the chip.
pub(crate) fn load(name: &str, drivers: &Path) -> Result<Chip> {
    let sdkconfig_path = drivers.join(format!("include/{name}/sdkconfig.h"));
    let sdkconfig = fs::read_to_string(&sdkconfig_path).with_context(|| {
        format!(
            "can't read {}, add {name} to esp-wireless-drivers-3rdparty first",
            sdkconfig_path.display()
        )
    })?;
    // ESP-IDF has no capability for the `a` extension, so PMA stands in for it: the cores
    // without atomics (ESP32-C2, ESP32-C3) have no PMA either.
    let target = if defines(&sdkconfig, "CONFIG_IDF_TARGET_ARCH_XTENSA") {
        format!("xtensa-{name}-none-elf")
    } else if defines(&sdkconfig, "CONFIG_SOC_CPU_HAS_FPU") {
        "riscv32imafc-unknown-none-elf".to_string()
    } else if defines(&sdkconfig, "CONFIG_SOC_CPU_HAS_PMA") {
        "riscv32imac-unknown-none-elf".to_string()
    } else {
        "riscv32imc-unknown-none-elf".to_string()
    };

    let radios = RADIOS
        .iter()
        .filter(|(_, symbol, _)| defines(&sdkconfig, symbol))
        .map(|(feature, _, _)| *feature)
        .collect();

    let mut libs = fs::read_dir(drivers.join(format!("libs/{name}")))
        .with_context(|| format!("can't read libs/{name} in {}", drivers.display()))?
        .flatten()
        .filter_map(|entry| {
            let file = entry.file_name().into_string().ok()?;
            Some(file.strip_prefix("lib")?.strip_suffix(".a")?.to_string())
        })
        .collect::<Vec<_>>();
    libs.sort();
    for lib in &libs {
        if !LIBS.iter().any(|(known, _)| known == lib) && !UNLINKED_LIBS.contains(&lib.as_str()) {
            anyhow::bail!(
                "libs/{name}/lib{lib}.a in {} is not in `LIBS` in xtask/src/gen.rs",
                drivers.display()
            );
        }
    }

    Ok(Chip {
        name: name.to_string(),
        target,
        radios,
        libs,
    })
}

/// The `version` in the crate's current `Cargo.toml`, or `0.3.0` for a new crate.
fn crate_version(existing: Option<&str>) -> String {
    existing
        .and_then(|toml| {
            toml.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "version").then(|| value.trim().trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "0.3.0".to_string())
}

pub(crate) fn cargo_toml(chip: &Chip, existing: Option<&str>) -> String {
    let mut radios = String::new();
    for (feature, _, comment) in RADIOS {
        if chip.has(feature) {
            radios.push_str(&format!("# {comment}\n{feature} = []\n"));
        }
    }
    let radios = if radios.is_empty() {
        "\n".to_string()
    } else {
        format!("\n{radios}\n")
    };
    let mut keywords = vec!["esp", "no-std"];
    if chip.has("bt") {
        keywords.push("bluetooth");
    }
    if chip.has("wifi") {
        keywords.push("wifi");
    }
    keywords.sort();
    let keywords = keywords
        .iter()
        .map(|k| format!("\"{k}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let phrase = match (chip.has("wifi"), chip.has("bt")) {
        (true, true) => "WiFi and Bluetooth",
        (true, false) => "WiFi",
        (false, true) => "Bluetooth",
        (false, false) => "radio",
    };
    format!(
        "[package]\n\
         name        = \"{crate}\"\n\
         version     = \"{version}\"\n\
         edition     = \"2021\"\n\
         description = \"Bindings to Espressif's {phrase} low-level drivers for {display}\"\n\
         repository  = \"https://github.com/esp-rs/esp-wifi-sys\"\n\
         license     = \"MIT OR Apache-2.0\"\n\
         \n\
         keywords   = [{keywords}]\n\
         categories = [\"embedded\", \"hardware-support\", \"no-std\"]\n\
         \n\
         [dependencies]\n\
         log   = {{ version = \"0.4.25\", optional = true }}\n\
         defmt = {{ version = \"1.0.1\", optional = true }}\n\
         \n\
         [features]\n\
         default = []\n\
         \n\
         # Enable log output from binary blobs - requires nightly\n\
         sys-logs = []\n\
         {radios}\
         [package.metadata.docs.rs]\n\
         features = []\n",
        crate = chip.crate_name(),
        version = crate_version(existing),
        phrase = phrase,
        display = display_name(&chip.name),
        keywords = keywords,
        radios = radios,
    )
}

pub(crate) fn build_rs(chip: &Chip) -> Result<String> {
    let mut vars = String::new();
    for (feature, _, _) in RADIOS {
        if chip.has(feature) {
            vars.push_str(&format!(
                "    let {feature} = env::var_os(\"CARGO_FEATURE_{}\").is_some();\n",
                feature.to_uppercase()
            ));
        }
    }
    let mut entries = String::new();
    for (lib, features) in LIBS {
        if !chip.libs.iter().any(|have| have == lib) {
            continue;
        }
        let cond = if features.is_empty() {
            "true".to_string()
        } else {
            let enabling: Vec<_> = features.iter().copied().filter(|f| chip.has(f)).collect();
            if enabling.is_empty() {
                anyhow::bail!(
                    "{}: lib{lib}.a ships, but the chip supports none of {features:?}",
                    chip.name
                );
            }
            enabling.join(" || ")
        };
        entries.push_str(&format!("        (\"{lib}\", {cond}),\n"));
    }
    Ok(format!(
        "use std::{{env, path::PathBuf}};\n\
         \n\
         fn main() {{\n\
         \x20   // Put the linker script somewhere the linker can find it\n\
         \x20   let out = PathBuf::from(env::var_os(\"OUT_DIR\").unwrap());\n\
         \n\
         {vars}\
         \n\
         \x20   let libs = [\n\
         {entries}\
         \x20   ];\n\
         \n\
         \x20   for (lib, _) in libs.into_iter().filter(|(_, enabled)| *enabled) {{\n\
         \x20       std::fs::copy(\n\
         \x20           format!(\"libs/lib{{}}.a\", lib),\n\
         \x20           out.join(format!(\"lib{{}}.a\", lib)),\n\
         \x20       )\n\
         \x20       .unwrap_or_else(|e| panic!(\"Failed to copy the {{lib}} library: {{e}}\"));\n\
         \x20       println!(\"cargo:rustc-link-lib={{lib}}\");\n\
         \x20   }}\n\
         \n\
         \x20   println!(\"cargo:rustc-link-search={{}}\", out.display());\n\
         }}\n"
    ))
}

/// The workspace `Cargo.toml` with `members` set to the chip crates and `xtask`. Any other member
/// is dropped.
pub(crate) fn workspace_toml(current: &str, chips: &[Chip]) -> Result<String> {
    let (before, rest) = current
        .split_once("members = [\n")
        .ok_or_else(|| anyhow!("Cargo.toml has no `members = [` block"))?;
    let (_, after) = rest
        .split_once("]\n")
        .ok_or_else(|| anyhow!("Cargo.toml has an unclosed `members = [` block"))?;
    let mut out = format!("{before}members = [\n");
    for chip in chips {
        out.push_str(&format!("    \"{}\",\n", chip.crate_name()));
    }
    out.push_str("    \"xtask\",\n]\n");
    out.push_str(after);
    Ok(out)
}

pub(crate) fn cargo_config(chip: &Chip) -> String {
    let mut config = format!("[build]\ntarget = \"{}\"\n", chip.target);
    if chip.arch() == Arch::Xtensa {
        config.push_str("\n[unstable]\nbuild-std = [\"core\"]\n");
    }
    config
}

pub(crate) fn readme(chip: &Chip) -> String {
    include_str!("../templates/README.md").replace("{crate}", &chip.crate_name())
}

/// `#if` for `feature`. Lists the chips that lack it when that list is shorter, so Bluetooth
/// stays `#if !defined(CONFIG_IDF_TARGET_ESP32S2)` instead of naming every other chip.
fn if_line(chips: &[Chip], feature: &str) -> String {
    let mut has = Vec::new();
    let mut lacks = Vec::new();
    for chip in chips {
        if chip.has(feature) {
            has.push(chip.name.as_str());
        } else {
            lacks.push(chip.name.as_str());
        }
    }
    has.sort();
    lacks.sort();
    let defined = |name: &str| format!("defined(CONFIG_IDF_TARGET_{})", name.to_uppercase());
    if has.is_empty() {
        "#if 0".to_string()
    } else if lacks.is_empty() {
        "#if 1".to_string()
    } else if lacks.len() < has.len() {
        let cond = lacks
            .iter()
            .map(|name| format!("!{}", defined(name)))
            .collect::<Vec<_>>()
            .join(" && ");
        format!("#if {cond}")
    } else {
        let cond = has
            .iter()
            .map(|name| defined(name))
            .collect::<Vec<_>>()
            .join(" || ");
        format!("#if {cond}")
    }
}

/// `c/include/include.h` with the `#if` above each feature header set to the chips that have it.
pub(crate) fn include_h(current: &str, chips: &[Chip]) -> Result<String> {
    let mut lines: Vec<String> = current.lines().map(str::to_string).collect();
    for (header, feature) in FEATURE_HEADERS {
        let include = format!("#include \"{header}\"");
        let at = lines
            .iter()
            .position(|line| line.trim() == include)
            .ok_or_else(|| anyhow!("c/include/include.h has no `{include}`"))?;
        let if_at = lines[..at]
            .iter()
            .rposition(|line| line.starts_with("#if") || line.starts_with("#endif"))
            .filter(|&i| lines[i].starts_with("#if"))
            .ok_or_else(|| anyhow!("c/include/include.h: `{include}` is not inside an `#if`"))?;
        lines[if_at] = if_line(chips, feature);
    }
    Ok(lines.join("\n") + "\n")
}

/// The root README, with the parenthetical chip list of the `ieee802154` feature replaced.
pub(crate) fn root_readme(current: &str, chips: &[Chip]) -> Result<String> {
    let mut lines: Vec<String> = current.lines().map(str::to_string).collect();
    let at = lines
        .iter()
        .position(|line| line.contains("- `ieee802154`:"))
        .ok_or_else(|| anyhow!("README.md has no `- `ieee802154`:` line"))?;
    let mut names: Vec<String> = chips
        .iter()
        .filter(|chip| chip.has("ieee802154"))
        .map(|chip| display_name(&chip.name))
        .collect();
    names.sort();
    let listed = names.join(", ");
    let before = lines[at]
        .rsplit_once('(')
        .map(|(before, _)| before.to_string())
        .ok_or_else(|| {
            anyhow!("README.md: the `ieee802154` line has no chip list in parentheses")
        })?;
    lines[at] = format!("{before}({listed})");
    Ok(lines.join("\n") + "\n")
}

/// Chip directory names under `drivers/libs`.
pub(crate) fn driver_chips(drivers: &Path) -> Vec<String> {
    let mut names = fs::read_dir(drivers.join("libs"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    names.sort();
    names
}

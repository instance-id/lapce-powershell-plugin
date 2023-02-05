// #![no_main]
#![allow(non_snake_case)]
// Deny usage of print and eprint as it won't have same resulti
// in WASI as if doing in standard program, you must really know
// what you are doing to disable that lint (and you don't know)
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use std::{
    fs::{self, File},
    io,
};

use zip::ZipArchive;

use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType,
            Url,
        },
        Request,
    },
    register_plugin, Http, LapcePlugin, PluginServerRpc, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

const PWSH_LSP_BASE: &str = "PowerShellEditorServices";
const PLUGIN_PREFIX: &str = "[lapce-powershell-lsp]";
const LANGUAGE: &str = "PowerShell";

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![DocumentFilter {
        language: Some(String::from("powershell")),
        pattern: Some(String::from("**/*.{ps1,psm1,psd1}")),
        scheme: None,
    }];

    let volt_uri = std::env::var("VOLT_URI")?;
    let plugin_path = volt_uri.replace("file://", "");

    PLUGIN_RPC.stderr(&format!("{PLUGIN_PREFIX} starting plugin..."));

    let mut server_args = vec![];

    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(lsp) = options.get("lsp") {
            if let Some(args) = lsp.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = lsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        let server_uri = Url::parse(&format!("urn:{server_path}"))?;

                        PLUGIN_RPC.stderr(&format!(
                            "Starting {LANGUAGE} Language Server from {server_uri}"
                        ));

                        PLUGIN_RPC.start_lsp(
                            server_uri,
                            server_args,
                            document_selector,
                            params.initialization_options,
                        );
                        return Ok(());
                    }
                }
            }
        }
    }

    // Architecture check
    let _ = match VoltEnvironment::architecture().as_deref() {
        Ok("x86_64") => "x86_64",
        Ok("aarch64") => "aarch64",
        _ => return Ok(()),
    };

    // OS check
    let _ = match VoltEnvironment::operating_system().as_deref() {
        Ok("macos") => "macos",
        Ok("linux") => "linux",
        Ok("windows") => "windows",
        _ => return Ok(()),
    };

    // --| Plugin Setup --------------------
    // let _ = match VoltEnvironment::operating_system().as_deref() {
    //     Ok("windows") => {
    //         format!("{filename}")
    //     }
    //     _ => format!("{filename}"),
    // };

    // Plugin working directory
    let volt_uri = VoltEnvironment::uri()?;

    // if you want to use server from PATH
    let server_uri = Url::parse(&format!("urn:pwsh"))?;

    std::env::set_var(
        "PSES_BUNDLE_PATH",
        format!("{plugin_path}/PowerShellEditorServices/"),
    );
    std::env::set_var("SESSION_TEMP_PATH", plugin_path);

    let PSES_BUNDLE_PATH = std::env::var("PSES_BUNDLE_PATH")?;
    let SESSION_TEMP_PATH = std::env::var("SESSION_TEMP_PATH")?;

    // @formatter:off
    server_args = vec![
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-NoLogo".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-Command".to_string(),
        format!("{PSES_BUNDLE_PATH}/PowerShellEditorServices/Start-EditorServices.ps1").to_string(),
        "-BundledModulesPath".to_string(),
        format!("{PSES_BUNDLE_PATH}").to_string(),
        "-LogPath".to_string(),
        format!("{SESSION_TEMP_PATH}/lapce_powershell.log").to_string(),
        "-SessionDetailsPath".to_string(),
        format!("{SESSION_TEMP_PATH}/lapce_powershell.session.json").to_string(),
        "-FeatureFlags".to_string(),
        "@()".to_string(),
        "-AdditionalModules".to_string(),
        "@()".to_string(),
        "-HostName".to_string(),
        "lapce".to_string(),
        "-HostProfileId".to_string(),
        "0".to_string(),
        "-HostVersion".to_string(),
        "1.0.0".to_string(),
        "-Stdio".to_string(),
        "-LogLevel".to_string(),
        "Normal".to_string(),
    ];
    // @formatter:on

    // let mut tmp_args = Vec::new();
    //
    // tmp_args.push("-c".to_string());
    // tmp_args.push(
    //     "/home/mosthated/.local/share/lapce-nightly/plugins/instanceid.lapce-powershell/pses_run.sh"
    //         .to_string(),
    // );
    //
    // PLUGIN_RPC
    //     .execute_process(format!("{volt_uri}/pses_run.sh").to_string(), Vec::new())
    //     .expect("Failed to start PowerShell Editor Services");

    // download_lsp().expect("Failed to download PowerShell Editor Services");

    PLUGIN_RPC.stderr(&format!("{}", server_uri));
    PLUGIN_RPC.stderr(&format!(
        "PSES_BUNDLE_PATH: {} SESSION_TEMP_PATH: {}",
        std::env::var("PSES_BUNDLE_PATH").unwrap(),
        std::env::var("SESSION_TEMP_PATH").unwrap()
    ));
    PLUGIN_RPC.stderr(&format!("{:?}", server_args));
    PLUGIN_RPC.stderr(&format!("{:?}", params.initialization_options));

    // Available language IDs
    // https://github.com/lapce/lapce/blob/HEAD/lapce-proxy/src/buffer.rs#L173
    PLUGIN_RPC.start_lsp(
        server_uri,
        server_args,
        document_selector,
        params.initialization_options,
    );
    PLUGIN_RPC.stderr("after");
    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.window_show_message(
                        MessageType::ERROR,
                        format!("plugin returned with error: {e}"),
                    )
                }
            }
            _ => {}
        }
    }
}

fn download_lsp() -> Result<(), Box<dyn std::error::Error>> {
    let latest_url = format!("https://github.com/{LANGUAGE}/{PWSH_LSP_BASE}/releases/latest/download/{PWSH_LSP_BASE}.zip");
    PLUGIN_RPC.stderr(&format!("Downloading LSP: URL: {latest_url}"));
    let zip_path = PathBuf::from(format!("{PWSH_LSP_BASE}.zip"));
    let zip_file = zip_path.as_path();

    let mut resp = Http::get(&latest_url)?;

    if resp.status_code.is_success() {
        {
            let body = resp.body_read_all()?;
            std::fs::write(zip_file, body).unwrap();

            let mut zip = ZipArchive::new(File::open(zip_file).unwrap()).unwrap();
            // for every zip file
            for i in 0..zip.len() {
                let mut file = zip.by_index(i).unwrap();
                let outpath = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue,
                };
                PLUGIN_RPC.stderr(&format!("Extracting {}", outpath.display()));
                if (*file.name()).ends_with('/') {
                    std::fs::create_dir_all(&outpath).unwrap();
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).unwrap();
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath).unwrap();
                    std::io::copy(&mut file, &mut outfile).unwrap();
                }
            }
            std::fs::remove_file(zip_file).unwrap();
        };
        Ok(())
    } else {
        panic!("Response error: {}", resp.status_code);
    }

    // match Command::new("curl")
    //     .args([&latest_url, "-o", format!("{PWSH_LSP_BASE}.zip")])
    //     .output()
    // {
    //     Ok(output) => {
    //         if !output.status.success() {
    //             let error = std::str::from_utf8(&output.stderr).unwrap();
    //             PLUGIN_RPC.stderr(&format!("Error downloading LSP: {error}"));
    //         } else {
    //             PLUGIN_RPC.stderr("LSP Download complete!");
    //         }
    //     }
    //     Err(error) => {
    //         PLUGIN_RPC.stderr(&format!("Error downloading LSP: {error}"));
    //     }
    // }
}

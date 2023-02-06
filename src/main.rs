#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![allow(non_snake_case)]

use anyhow::Result;
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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::File, path::PathBuf};
use zip::ZipArchive;

// --| Plugin Constants --------------------
const PWSH_LSP_BASE: &str = "PowerShellEditorServices";
const PLUGIN_PREFIX: &str = "[lapce-powershell-lsp]";
const FILENAME: &str = "PowerShellEditorServices.zip";
const LANGUAGE: &str = "PowerShell";
const PWSH_WIN: &str = "pwsh.exe";
const PWSH_NIX: &str = "pwsh";

#[derive(Default)]
struct State {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    arch: String,
    os: String,
    configuration: Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    language_id: String,
    options: Option<Value>,
}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![DocumentFilter {
        language: Some(String::from("powershell")),
        pattern: Some(String::from("**/*.{ps1,psm1,psd1}")),
        scheme: None,
    }];

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

    // --| Plugin Setup -----------------------------------
    // --|-------------------------------------------------
    let pwsh_exe = match VoltEnvironment::operating_system().as_deref() {
        Ok("windows") => {
            format!("{PWSH_WIN}")
        }
        _ => format!("{PWSH_NIX}"),
    };

    let volt_uri = std::env::var("VOLT_URI")?;
    let plugin_path = volt_uri.replace("file://", "");
    let server_uri = Url::parse(format!("urn:{pwsh_exe}").as_str())?;

    // --| Environment Variables Needed by PSES -----------
    // --|-------------------------------------------------
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
        "-NoLogo".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-Command".to_string(),
        format!("{PSES_BUNDLE_PATH}/PowerShellEditorServices/Start-EditorServices.ps1"),
        "-BundledModulesPath".to_string(),
        format!("{PSES_BUNDLE_PATH}"),
        "-LogPath".to_string(),
        format!("{SESSION_TEMP_PATH}/lapce_powershell.log"),
        "-SessionDetailsPath".to_string(),
        format!("{SESSION_TEMP_PATH}/lapce_powershell.session.json"),
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

    // --| Not working for some reason. Wasted a whole weekend trying --------------------
    // --| No matter what I try, I end up with 'no pre-opened file descriptors' error ----
    // --| Will continue to research this and see if there is a resolution ---------------

    // let file_name = PWSH_LSP_BASE.to_string();
    // let file_path = PathBuf::from(&file_name);
    // let gz_path = PathBuf::from(file_name + ".zip");

    // if !file_path.exists() {
    //     PLUGIN_RPC.stderr(&format!("Downloading {LANGUAGE} Language Server"));
    //
    //     let result: Result<()> = {
    //         let url = format!(
    //             "https://github.com/{LANGUAGE}/{PWSH_LSP_BASE}/releases/latest/download/{PWSH_LSP_BASE}.zip"
    //         );
    //
    //         let mut resp = Http::get(&url)?;
    //         let body = resp.body_read_all()?;
    //         std::fs::write(&gz_path, body)?;
    //
    //         // --| Extract zip file -------------------
    //         let mut zip = ZipArchive::new(File::open(gz_path).unwrap()).unwrap();
    //
    //         for i in 0..zip.len() {
    //             let mut file = zip.by_index(i).unwrap();
    //             let outpath = match file.enclosed_name() {
    //                 Some(path) => path.to_owned(),
    //                 None => continue,
    //             };
    //             PLUGIN_RPC.stderr(&format!("Extracting {}", outpath.display()));
    //             if (*file.name()).ends_with('/') {
    //                 std::fs::create_dir_all(&outpath).unwrap();
    //             } else {
    //                 if let Some(p) = outpath.parent() {
    //                     if !p.exists() {
    //                         std::fs::create_dir_all(p).unwrap();
    //                     }
    //                 }
    //                 let mut outfile = File::create(&outpath).unwrap();
    //                 std::io::copy(&mut file, &mut outfile).unwrap();
    //             }
    //         }
    //
    //         // let mut file = File::create(&file_path)?;
    //         // std::io::copy(&mut gz, &mut file)?;
    //         // std::fs::remove_file(&gz_path)?;
    //         Ok(())
    //     };
    //     if result.is_err() {
    //         PLUGIN_RPC.window_show_message(
    //             MessageType::ERROR,
    //             "can't download PowerShell Editor Services, please use server path in the settings.".to_string(),
    //         );
    //         return Ok(());
    //     }
    // }
    // download_lsp().expect("Failed to download PowerShell Editor Services");

    PLUGIN_RPC.stderr(&format!("{server_uri}"));
    PLUGIN_RPC.stderr(&format!(
        "PSES_BUNDLE_PATH: {} SESSION_TEMP_PATH: {}",
        std::env::var("PSES_BUNDLE_PATH").unwrap(),
        std::env::var("SESSION_TEMP_PATH").unwrap()
    ));
    PLUGIN_RPC.stderr(&format!("{server_args:?}"));
    PLUGIN_RPC.stderr(&format!("{:?}", params.initialization_options));

    PLUGIN_RPC.start_lsp(
        server_uri,
        server_args,
        document_selector,
        params.initialization_options,
    );

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

// --| Another try -----------
// fn download_lsp() {
//     let latest_url = format!("https://github.com/{LANGUAGE}/{PWSH_LSP_BASE}/releases/latest/download/{PWSH_LSP_BASE}.zip");
//     PLUGIN_RPC.stderr(&format!("Downloading LSP: URL: {latest_url}"));

//     let zip_path = PathBuf::from(format!("{PWSH_LSP_BASE}.zip"));
//     let zip_file = zip_path.as_path();

//     let file_name = format!("PWSH_LSP_BASE");
//     let file_path = PathBuf::from(&file_name);
//     let gz_path = PathBuf::from(file_name.clone() + ".zip");

//     if !file_path.exists() {
//         let result: Result<()> = {
//             let url = format!(
//                 "https://github.com/{LANGUAGE}/{PWSH_LSP_BASE}/releases/latest/download/{PWSH_LSP_BASE}.zip"
//             );

//             let mut resp = Http::get(&url)?;
//             let body = resp.body_read_all()?;
//             std::fs::write(&gz_path, body)?;
//             let mut gz = ZipArchive::new(File::open(&gz_path)?);
//             let mut file = File::create(&file_path)?;
//             std::io::copy(&mut gz, &mut file)?;
//             std::fs::remove_file(&gz_path)?;
//             Ok(())
//         };
//         if result.is_err() {
//             PLUGIN_RPC.window_show_message(
//                 MessageType::ERROR,
//                 format!("can't download rust-analyzer, please use server path in the settings."),
//             );
//             return Ok(());
//         }
//     }
//     // if let Ok(mut resp) = Http::get(&latest_url) {
//     //     if resp.status_code.is_success() {
//     //         {
//     //             let body = resp.body_read_all()?;
//     //
//     //             match std::fs::write(zip_file, body) {
//     //                 Ok(_) => PLUGIN_RPC.stderr(&format!("Downloaded LSP: {}", zip_file.display())),
//     //                 Err(e) => PLUGIN_RPC.stderr(&format!("Failed to download LSP: {e}")),
//     //             }
//     //
//     //             let mut zip = ZipArchive::new(File::open(zip_file).unwrap()).unwrap();
//     //             // for every zip file
//     //             for i in 0..zip.len() {
//     //                 let mut file = zip.by_index(i).unwrap();
//     //                 let outpath = match file.enclosed_name() {
//     //                     Some(path) => path.to_owned(),
//     //                     None => continue,
//     //                 };
//     //                 PLUGIN_RPC.stderr(&format!("Extracting {}", outpath.display()));
//     //                 if (*file.name()).ends_with('/') {
//     //                     std::fs::create_dir_all(&outpath).unwrap();
//     //                 } else {
//     //                     if let Some(p) = outpath.parent() {
//     //                         if !p.exists() {
//     //                             std::fs::create_dir_all(p).unwrap();
//     //                         }
//     //                     }
//     //                     let mut outfile = File::create(&outpath).unwrap();
//     //                     std::io::copy(&mut file, &mut outfile).unwrap();
//     //                 }
//     //             }
//     //             std::fs::remove_file(zip_file).unwrap()
//     //         };
//     //         Ok(())
//     //     } else {
//     //         PLUGIN_RPC.stderr(&format!("Error: {0}", resp.status_code));
//     //         Ok(())
//     //     }
//     // } else {
//     //     todo!()
//     // }
// }

use std::{
    env, error,
    fmt::Debug,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use derive_more::Display;
use keyvalues_parser::{Value, Vdf};

/// The Steam ID for Crusader Kings III.
/// Source: [SteamDB](https://steamdb.info/app/1158310/)
const CK3_ID: &str = "1158310";

#[cfg(target_os = "linux")]
const DEFAULT_STEAM_PATH: [&str; 2] = [
    ".local/share/Steam/steamapps/",
    ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/",
];
#[cfg(target_os = "windows")]
const DEFAULT_STEAM_PATH: [&str; 1] = ["C:\\Program Files (x86)\\Steam\\steamapps\\"];
#[cfg(target_os = "macos")]
const DEFAULT_STEAM_PATH: [&str; 1] = ["Library/Application Support/Steam/steamapps/"];

/// The default path from the Steam directory to the libraryfolders.vdf file.
const DEFAULT_VDF_PATH: &str = "libraryfolders.vdf";

/// The default path from the library to the CK3 directory.
pub const CK3_PATH: &str = "steamapps/common/Crusader Kings III/game";

#[derive(Debug, Display)]
pub enum SteamError {
    /// The Steam directory was not found.
    #[display("steam directory not found")]
    SteamDirNotFound,
    /// The VDF file was not found.
    #[display("VDF file not found")]
    VdfNotFound,
    /// An error occurred while parsing the VDF file.
    #[display("library error parsing VDF file: {_0}")]
    VdfParseError(keyvalues_parser::error::Error),
    /// An error occurred while processing the VDF file.
    #[display("error processing VDF file: {_0}")]
    VdfProcessingError(&'static str),
    /// The CK3 directory was not found.
    #[display("CK3 directory not found")]
    Ck3NotFound,
    /// CK3 is missing from the library.
    #[display("CK3 missing from library")]
    CK3Missing,
}

impl error::Error for SteamError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            SteamError::VdfParseError(e) => Some(e),
            _ => None,
        }
    }
}

/// Tries to get the steam path
fn get_steam_path() -> Result<PathBuf, SteamError> {
    for path in DEFAULT_STEAM_PATH.iter() {
        let steam_path = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            #[allow(deprecated)]
            // home_dir is deprecated, because Windows is bad, but we don't care since we are only using it for Linux
            env::home_dir().unwrap().join(path)
        } else {
            Path::new(path).to_path_buf()
        };

        if steam_path.is_dir() {
            return Ok(steam_path.to_path_buf());
        }
    }
    Err(SteamError::SteamDirNotFound)
}

/// Get the path to the Crusader Kings III directory.
///
/// # Errors
///
/// This function shouldn't panic, but it can return an error.
/// Generally speaking error checking regarding VDF expected format is not performed.
/// So if the VDF file is weird, then [SteamError::CK3Missing] will be returned.
///
/// # Returns
///
/// The path to the Crusader Kings III directory.
pub fn get_ck3_path() -> Result<String, SteamError> {
    let vdf_path = get_steam_path()?.join(DEFAULT_VDF_PATH);
    if !vdf_path.exists() {
        return Err(SteamError::VdfNotFound);
    }
    let mut library_path = None;
    let vdf_contents = read_to_string(&vdf_path).unwrap();
    match Vdf::parse(&vdf_contents) {
        Ok(vdf) => {
            // vdf was parsed successfully
            if let Value::Obj(folders) = vdf.value {
                // root of the VDF file is an object
                for folder_objs in folders.values() {
                    // foreach value set in the root object
                    for folder in folder_objs {
                        // foreach value in the value set
                        if let Value::Obj(folder) = folder {
                            // if the value is an object
                            if let Some(apps_objs) = folder.get("apps") {
                                // if the object has an "apps" key
                                for app in apps_objs {
                                    // foreach value in the "apps" object
                                    if let Value::Obj(app) = app {
                                        // if the value is an object
                                        if app.keys().any(|k| k == CK3_ID) {
                                            // if the object has a key with the CK3 ID
                                            if let Some(path) = folder.get("path") {
                                                let path = path.get(0).unwrap();
                                                if let Value::Str(path) = path {
                                                    library_path = Some(path.to_owned());
                                                    break;
                                                } else {
                                                    return Err(SteamError::VdfProcessingError(
                                                        "Path is not a string",
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                                if library_path.is_some() {
                                    break;
                                }
                            } else {
                                // we could error here, but what's the point?
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    if library_path.is_some() {
                        break;
                    }
                }
            } else {
                return Err(SteamError::VdfProcessingError(
                    "Root of VDF file is not an object",
                ));
            }
        }
        Err(e) => {
            return Err(SteamError::VdfParseError(e));
        }
    }
    if let Some(library_path) = library_path {
        let ck3_path = Path::new(library_path.as_ref()).join(CK3_PATH);
        if ck3_path.exists() {
            Ok(ck3_path.to_string_lossy().to_string())
        } else {
            Err(SteamError::Ck3NotFound)
        }
    } else {
        return Err(SteamError::CK3Missing);
    }
}

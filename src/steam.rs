use std::{
    env, error,
    fmt::{self, Debug, Display},
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use derive_more::Display;
use keyvalues_parser::{Value, Vdf};

/// The Steam ID for Crusader Kings III.
/// Source: [SteamDB](https://steamdb.info/app/1158310/)
const CK3_ID: &str = "1158310";

// constant literal, but I dont like repeated data
const APPS_PATH: &str = "steamapps";

#[cfg(target_os = "linux")]
const DEFAULT_STEAM_PATH: [&str; 2] = [
    ".local/share/Steam/",
    ".var/app/com.valvesoftware.Steam/.local/share/Steam/",
];
#[cfg(target_os = "windows")]
const DEFAULT_STEAM_PATH: [&str; 1] = ["C:\\Program Files (x86)\\Steam\\"];
#[cfg(target_os = "macos")]
const DEFAULT_STEAM_PATH: [&str; 1] = ["Library/Application Support/Steam/"];

/// The default path from the Steam directory to the libraryfolders.vdf file.
const DEFAULT_VDF_PATH: &str = "libraryfolders.vdf";

const MOD_PATH: &str = "workshop/content/";

/// The default path from the library to the CK3 directory.
pub const CK3_PATH: &str = "common/Crusader Kings III/game";

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
    /// The CK3 directory was not found. So it was in the manifest, but not in the library.
    #[display("CK3 directory not found")]
    Ck3NotFound,
    /// According to the internal manifest, there is no CK3 installation.
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

/// Returns the path to the Steam directory.
/// If the Steam directory is not found, it will return an error.
fn get_steam_path() -> Result<PathBuf, SteamError> {
    for path in DEFAULT_STEAM_PATH.iter() {
        let mut steam_path = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            #[allow(deprecated)]
            // home_dir is deprecated, because Windows is bad, but we don't care since we are only using it for Linux
            env::home_dir().unwrap().join(path)
        } else {
            Path::new(path).to_path_buf()
        };

        steam_path.push(APPS_PATH);

        if steam_path.is_dir() {
            return Ok(steam_path.to_path_buf());
        }
    }
    Err(SteamError::SteamDirNotFound)
}

/// Get the path to the Steam library that has CK3 installed.
///
/// # Errors
///
/// This function shouldn't panic, but it can return an error.
/// Generally speaking error checking regarding VDF expected format is not performed.
/// So if the VDF file is weird, then [SteamError::CK3Missing] will be returned.
///
/// # Returns
///
/// The path to the Steam library that has CK3 installed.
pub fn get_library_path() -> Result<PathBuf, SteamError> {
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
        let lib_path = Path::new(library_path.as_ref()).join(APPS_PATH);
        if lib_path.exists() {
            Ok(lib_path)
        } else {
            Err(SteamError::Ck3NotFound)
        }
    } else {
        return Err(SteamError::CK3Missing);
    }
}

pub fn get_game_path(library_path: &PathBuf) -> Result<PathBuf, SteamError> {
    let ck3_path = library_path.join(CK3_PATH);
    if ck3_path.exists() {
        Ok(ck3_path)
    } else {
        Err(SteamError::Ck3NotFound)
    }
}

pub struct ModDescriptor {
    name: String,
    path: PathBuf,
}

impl Display for ModDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.name, self.path.display())
    }
}

impl ModDescriptor {
    fn new(name: String, path: PathBuf) -> Self {
        ModDescriptor { name, path }
    }
}

impl AsRef<PathBuf> for ModDescriptor {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

pub fn get_mod_paths(
    library_path: &PathBuf,
    out: &mut Vec<ModDescriptor>,
) -> Result<(), SteamError> {
    let mut mods_path = library_path.join(MOD_PATH);
    mods_path.push(CK3_ID);
    if mods_path.exists() {
        if let Ok(dir) = read_dir(&mods_path) {
            for mod_folder in dir {
                if let Ok(mod_folder) = mod_folder {
                    if !mod_folder.file_type().unwrap().is_dir() {
                        continue;
                    }
                    let mod_path = mod_folder.path();
                    if let Ok(descriptor_contents) = read_to_string(mod_path.join("descriptor.mod"))
                    {
                        for line in descriptor_contents.lines() {
                            if line.starts_with("name") {
                                let name = line.split('=').nth(1).unwrap().trim();
                                let name = name.trim_matches('"').to_string();
                                out.push(ModDescriptor::new(name, mod_path));
                                break;
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Err(SteamError::Ck3NotFound);
    }
    Ok(())
}

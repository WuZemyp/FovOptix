use once_cell::sync::Lazy;
use std::{
    env::{
        self,
        consts::{DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX, OS},
    },
    path::{Path, PathBuf},
};

pub fn exec_fname(name: &str) -> String {
    format!("{name}{EXE_SUFFIX}")
}

pub fn dynlib_fname(name: &str) -> String {
    format!("{DLL_PREFIX}{name}{DLL_SUFFIX}")
}

pub fn target_dir() -> PathBuf {
    // use `.parent().unwrap()` instead of `../` to maintain canonicalized form
    Path::new(env!("OUT_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

pub fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

pub fn crate_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(name)
}

pub fn deps_dir() -> PathBuf {
    workspace_dir().join("deps")
}

pub fn build_dir() -> PathBuf {
    workspace_dir().join("build")
}

pub fn streamer_build_dir() -> PathBuf {
    build_dir().join(format!("alvr_streamer_{OS}"))
}

pub fn installer_path() -> PathBuf {
    env::temp_dir().join(exec_fname("alvr_installer"))
}

pub fn dashboard_fname() -> &'static str {
    if cfg!(windows) {
        "ALVR Dashboard.exe"
    } else {
        "alvr_dashboard"
    }
}

// Layout of the ALVR installation. All paths are absolute
#[derive(Clone)]
pub struct Layout {
    // directory containing the dashboard executable
    pub executables_dir: PathBuf,
    // (linux only) directory where alvr_vulkan_layer.so is saved
    pub libraries_dir: PathBuf,
    // parent directory of resources like the dashboard and presets folders
    pub static_resources_dir: PathBuf,
    // directory for storing configuration files (session.json)
    pub config_dir: PathBuf,
    // directory for storing log
    pub log_dir: PathBuf,
    // directory to register in openVR driver path
    pub openvr_driver_root_dir: PathBuf,
    // (linux only) parent directory of the executable to wrap vrcompositor
    pub vrcompositor_wrapper_dir: PathBuf,
    // (linux only) directory where the vulkan layer manifest is saved
    pub vulkan_layer_manifest_dir: PathBuf,
}

impl Layout {
    pub fn new(root: &Path) -> Self {
        if cfg!(target_os = "linux") {
            // Get paths from environment or use FHS compliant paths
            let executables_dir = if !env!("executables_dir").is_empty() {
                PathBuf::from(env!("executables_dir"))
            } else {
                root.join("bin")
            };
            let libraries_dir = if !env!("libraries_dir").is_empty() {
                PathBuf::from(env!("libraries_dir"))
            } else {
                root.join("lib64")
            };
            let static_resources_dir = if !env!("static_resources_dir").is_empty() {
                PathBuf::from(env!("static_resources_dir"))
            } else {
                root.join("share/alvr")
            };
            let config_dir = if !env!("config_dir").is_empty() {
                PathBuf::from(env!("config_dir"))
            } else {
                dirs::config_dir().unwrap().join("alvr")
            };
            let log_dir = if !env!("log_dir").is_empty() {
                PathBuf::from(env!("log_dir"))
            } else {
                dirs::home_dir().unwrap()
            };
            let openvr_driver_root_dir = if !env!("openvr_driver_root_dir").is_empty() {
                PathBuf::from(env!("openvr_driver_root_dir"))
            } else {
                root.join("lib64/alvr")
            };
            let vrcompositor_wrapper_dir = if !env!("vrcompositor_wrapper_dir").is_empty() {
                PathBuf::from(env!("vrcompositor_wrapper_dir"))
            } else {
                root.join("libexec/alvr")
            };
            let vulkan_layer_manifest_dir = if !env!("vulkan_layer_manifest_dir").is_empty() {
                PathBuf::from(env!("vulkan_layer_manifest_dir"))
            } else {
                root.join("share/vulkan/explicit_layer.d")
            };

            Self {
                executables_dir,
                libraries_dir,
                static_resources_dir,
                config_dir,
                log_dir,
                openvr_driver_root_dir,
                vrcompositor_wrapper_dir,
                vulkan_layer_manifest_dir,
            }
        } else {
            Self {
                executables_dir: root.to_owned(),
                libraries_dir: root.to_owned(),
                static_resources_dir: root.to_owned(),
                config_dir: root.to_owned(),
                log_dir: root.to_owned(),
                openvr_driver_root_dir: root.to_owned(),
                vrcompositor_wrapper_dir: root.to_owned(),
                vulkan_layer_manifest_dir: root.to_owned(),
            }
        }
    }

    pub fn dashboard_exe(&self) -> PathBuf {
        self.executables_dir.join(dashboard_fname())
    }

    pub fn resources_dir(&self) -> PathBuf {
        self.openvr_driver_root_dir.join("resources")
    }

    pub fn dashboard_dir(&self) -> PathBuf {
        self.static_resources_dir.join("dashboard")
    }

    pub fn presets_dir(&self) -> PathBuf {
        self.static_resources_dir.join("presets")
    }

    pub fn session(&self) -> PathBuf {
        self.config_dir.join("session.json")
    }

    pub fn session_log(&self) -> PathBuf {
        if cfg!(target_os = "linux") {
            self.log_dir.join("alvr_session_log.txt")
        } else {
            self.log_dir.join("session_log.txt")
        }
    }

    pub fn crash_log(&self) -> PathBuf {
        self.log_dir.join("crash_log.txt")
    }

    pub fn openvr_driver_lib_dir(&self) -> PathBuf {
        let platform = if cfg!(windows) {
            "win64"
        } else if cfg!(target_os = "linux") {
            "linux64"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            unimplemented!()
        };

        self.openvr_driver_root_dir.join("bin").join(platform)
    }

    // path to the shared library to be loaded by openVR
    pub fn openvr_driver_lib(&self) -> PathBuf {
        self.openvr_driver_lib_dir()
            .join(format!("driver_alvr_server.{DLL_EXTENSION}"))
    }

    // path to the manifest file for openVR
    pub fn openvr_driver_manifest(&self) -> PathBuf {
        self.openvr_driver_root_dir.join("driver.vrdrivermanifest")
    }

    pub fn vrcompositor_wrapper(&self) -> PathBuf {
        self.vrcompositor_wrapper_dir.join("vrcompositor-wrapper")
    }

    pub fn vulkan_layer(&self) -> PathBuf {
        self.libraries_dir.join(dynlib_fname("alvr_vulkan_layer"))
    }

    pub fn vulkan_layer_manifest(&self) -> PathBuf {
        self.vulkan_layer_manifest_dir.join("alvr_x86_64.json")
    }
}

static LAYOUT_FROM_ENV: Lazy<Option<Layout>> =
    Lazy::new(|| (!env!("root").is_empty()).then(|| Layout::new(Path::new(env!("root")))));

// The path should include the executable file name
// The path argument is used only if ALVR is built as portable
pub fn filesystem_layout_from_dashboard_exe(path: &Path) -> Layout {
    LAYOUT_FROM_ENV.clone().unwrap_or_else(|| {
        let root = if cfg!(target_os = "linux") {
            // FHS path is expected
            path.parent().unwrap().parent().unwrap().to_owned()
        } else {
            path.parent().unwrap().to_owned()
        };

        Layout::new(&root)
    })
}

// The dir argument is used only if ALVR is built as portable
pub fn filesystem_layout_from_openvr_driver_root_dir(dir: &Path) -> Layout {
    LAYOUT_FROM_ENV.clone().unwrap_or_else(|| {
        let root = if cfg!(target_os = "linux") {
            // FHS path is expected
            dir.parent().unwrap().parent().unwrap().to_owned()
        } else {
            dir.to_owned()
        };

        Layout::new(&root)
    })
}

// Use this when there is no way of determining the current path. The resulting Layout paths will
// be invalid, except for the ones that disregard the relative path (for example the config dir) and
// the ones that have been overridden.
pub fn filesystem_layout_invalid() -> Layout {
    LAYOUT_FROM_ENV
        .clone()
        .unwrap_or_else(|| Layout::new(Path::new("")))
}

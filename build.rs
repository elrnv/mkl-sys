use bindgen::callbacks::{IntKind, ParseCallbacks};
use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

/// Paths required for linking to MKL
struct InstallationDirectories {
    mkl_lib_dir: String,
    omp_lib_dir: String,
    tbb_lib_dir: String,
    include_dir: String,
}

impl InstallationDirectories {
    /// Constructs paths required for linking MKL from the root folder specified in env.
    ///
    /// Checks if paths exist.
    fn try_from_env_root() -> Result<Self, String> {
        env::var("ONEAPI_ROOT")
        .map_err(|err|
            format!("ONEAPI_ROOT {err} -- remember to run the setvars script bundled with oneAPI in order to set up the required environment variables")
        ).and_then(|oneapi_root| Self::try_from_root(&oneapi_root))
    }

    /// Constructs paths required for linking MKL from the specified root folder.
    ///
    /// Checks if paths exist.
    fn try_from_root(root: &str) -> Result<Self, String> {
        // TODO determine if we need to support ia32 as well
        let itype = "intel64";

        let tbb_lib_subdir = if cfg!(target_os = "linux") {
            format!("/{}/gcc4.8", itype)
        } else if cfg!(target_os = "windows") {
            format!("/{}/vc14", itype)
        } else {
            String::new()
        };

        let tbb_root: String = format!("{}/tbb/latest", root);
        let mkl_root: String = format!("{}/mkl/latest", root);
        let compiler_root: String = format!("{}/compiler/latest", root);
        let mkl_lib_dir = format!("{mkl_root}/lib",);
        let tbb_lib_dir = format!("{tbb_root}/lib{tbb_lib_subdir}",);
        let omp_lib_dir = format!("{compiler_root}/lib",);
        let include_dir = format!("{}/include", mkl_root);

        let mkl_root_path = PathBuf::from(mkl_root);

        let mkl_root_str = mkl_root_path
            .to_str()
            .ok_or("Unable to convert 'mkl_root' to string")?;

        // Check if paths exist

        if !mkl_root_path.exists() {
            println!(
                "cargo:warning=The 'mkl_root' folder with path '{}' does not exist.",
                mkl_root_str
            );
        }

        Self::try_custom(&mkl_lib_dir, &include_dir, &tbb_lib_dir, &omp_lib_dir)
    }

    /// Constructs paths required for linking MKL using system paths.
    ///
    /// Checks if paths exist.
    fn try_system() -> Result<Self, String> {
        if !cfg!(target_os = "linux") {
            return Err("Cannot determine default MKL installation path in target OS".into());
        };

        let mkl_lib_dir = "/lib/x86_64-linux-gnu".to_string();
        let tbb_lib_dir = mkl_lib_dir.clone();
        let omp_lib_dir = mkl_lib_dir.clone();
        let include_dir = format!("/usr/include/mkl");

        Self::try_custom(&mkl_lib_dir, &include_dir, &tbb_lib_dir, &omp_lib_dir)
    }

    /// Constructs paths required for linking MKL from specific environment variables.
    ///
    /// Checks if paths exist.
    fn try_custom_env() -> Result<Self, String> {
        let mkl_lib_dir = env::var("MKL_LIB_DIR").map_err(|err| format!("MKL_LIB_DIR {err}"))?;
        let mkl_include_dir =
            env::var("MKL_INCLUDE_DIR").map_err(|err| format!("MKL_INCLUDE_DIR {err}"))?;
        let tbb_lib_dir = if cfg!(feature = "tbb") {
            env::var("TBB_LIB_DIR").map_err(|err| format!("TBB_LIB_DIR {err}"))?
        } else {
            String::new()
        };
        let omp_lib_dir = if cfg!(feature = "openmp") {
            env::var("OMP_LIB_DIR").map_err(|err| format!("OMP_LIB_DIR {err}"))?
        } else {
            String::new()
        };
        Self::try_custom(&mkl_lib_dir, &mkl_include_dir, &tbb_lib_dir, &omp_lib_dir)
    }

    /// Constructs paths required for linking MKL using custom paths.
    ///
    /// Checks if paths exist.
    fn try_custom(
        mkl_lib_dir: &str,
        mkl_include_dir: &str,
        tbb_lib_dir: &str,
        omp_lib_dir: &str,
    ) -> Result<Self, String> {
        let mkl_lib_dir_path = PathBuf::from(mkl_lib_dir);
        let omp_lib_dir_path = PathBuf::from(omp_lib_dir);
        let tbb_lib_dir_path = PathBuf::from(tbb_lib_dir);
        let mkl_include_dir_path = PathBuf::from(mkl_include_dir);

        let lib_dir_str = mkl_lib_dir_path
            .to_str()
            .ok_or("Unable to convert 'mkl_lib_dir_path' to string")?;
        let omp_lib_dir_str = omp_lib_dir_path
            .to_str()
            .ok_or("Unable to convert 'omp_lib_dir_path' to string")?;
        let tbb_lib_dir_str = tbb_lib_dir_path
            .to_str()
            .ok_or("Unable to convert 'tbb_lib_dir_path' to string")?;
        let mkl_include_dir_str = mkl_include_dir_path
            .to_str()
            .ok_or("Unable to convert 'mkl_include_dir_path' to string")?;

        // Check if paths exist

        if !mkl_lib_dir_path.exists() {
            println!(
                "cargo:warning=The 'mkl_lib_dir_path' folder with path '{}' does not exist.",
                lib_dir_str
            );
        }

        if cfg!(feature = "openmp") && !omp_lib_dir_path.exists() {
            println!(
                "cargo:warning=The 'omp_lib_dir_path' folder with path '{}' does not exist.",
                omp_lib_dir_str
            );
        }

        if cfg!(feature = "tbb") && !tbb_lib_dir_path.exists() {
            println!(
                "cargo:warning=The 'tbb_lib_dir_path' folder with path '{}' does not exist.",
                tbb_lib_dir_str
            );
        }

        if !mkl_include_dir_path.exists() {
            println!(
                "cargo:warning=The 'mkl_include_dir_path' folder with path '{}' does not exist.",
                mkl_include_dir_str
            );
        }

        Ok(InstallationDirectories {
            mkl_lib_dir: lib_dir_str.into(),
            omp_lib_dir: omp_lib_dir_str.into(),
            tbb_lib_dir: tbb_lib_dir_str.into(),
            include_dir: mkl_include_dir_str.into(),
        })
    }
}

fn get_lib_dirs(install_dirs: &InstallationDirectories) -> Vec<String> {
    let mut lib_dirs = vec![install_dirs.mkl_lib_dir.clone()];
    if cfg!(feature = "openmp") {
        lib_dirs.push(install_dirs.omp_lib_dir.clone());
    } else if cfg!(feature = "tbb") {
        lib_dirs.push(install_dirs.tbb_lib_dir.clone());
    }
    lib_dirs
}

fn get_dynamic_link_libs_windows() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "tbb") {
        libs.push("tbb");
    }

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_dynamic_link_libs_linux() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "openmp") {
        libs.push("iomp5");
    } else if cfg!(feature = "tbb") {
        libs.push("tbb");
    }
    // stdc++ is only needed for more recent MKL versions.
    // Adding it here anyways since most systems will have this dependency anyways.
    libs.extend(vec!["pthread", "m", "dl", "stdc++"]);

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_dynamic_link_libs_macos() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "openmp") {
        libs.push("iomp5");
    }
    libs.extend(vec!["pthread", "m", "dl"]);

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_static_link_libs_windows() -> Vec<String> {
    let mut libs = Vec::new();

    if cfg!(feature = "ilp64") {
        libs.push("mkl_intel_ilp64");
    } else {
        libs.push("mkl_intel_lp64");
    };

    if cfg!(feature = "openmp") {
        libs.push("mkl_intel_thread");
    } else if cfg!(feature = "tbb") {
        libs.push("mkl_tbb_thread");
    } else {
        libs.push("mkl_sequential");
    };

    libs.push("mkl_core");

    if cfg!(feature = "openmp") {
        libs.push("libiomp5md");
    }

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_static_link_libs_macos() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "ilp64") {
        libs.push("mkl_intel_ilp64");
    } else {
        libs.push("mkl_intel_lp64");
    };

    if cfg!(feature = "openmp") {
        libs.push("mkl_intel_thread");
    } else if cfg!(feature = "tbb") {
        libs.push("mkl_tbb_thread");
    } else {
        libs.push("mkl_sequential");
    };

    libs.push("mkl_core");

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_static_link_libs_linux() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "ilp64") {
        libs.push("mkl_intel_ilp64");
    } else {
        libs.push("mkl_intel_lp64");
    };

    if cfg!(feature = "openmp") {
        libs.push("mkl_intel_thread");
    } else if cfg!(feature = "tbb") {
        libs.push("mkl_tbb_thread");
    } else {
        libs.push("mkl_sequential");
    };

    libs.push("mkl_core");

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_dynamic_link_libs() -> Vec<String> {
    if cfg!(target_os = "windows") {
        get_dynamic_link_libs_windows()
    } else if cfg!(target_os = "linux") {
        get_dynamic_link_libs_linux()
    } else if cfg!(target_os = "macos") {
        get_dynamic_link_libs_macos()
    } else {
        panic!("Target OS not supported");
    }
}

fn get_static_link_libs() -> Vec<String> {
    if cfg!(target_os = "windows") {
        get_static_link_libs_windows()
    } else if cfg!(target_os = "linux") {
        get_static_link_libs_linux()
    } else if cfg!(target_os = "macos") {
        get_static_link_libs_macos()
    } else {
        panic!("Target OS not supported");
    }
}

fn get_cflags_windows(install_dirs: Option<&InstallationDirectories>) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    if let Some(install_dirs) = install_dirs {
        cflags.push("--include-directory".into());
        cflags.push(format!("{}", install_dirs.include_dir));
    }
    cflags
}

fn get_cflags_linux(install_dirs: Option<&InstallationDirectories>) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    if let Some(install_dirs) = install_dirs {
        cflags.push("-I".into());
        cflags.push(format!("{}", install_dirs.include_dir));
    } else {
        cflags.push("-I".into());
        cflags.push(format!("/usr/include/mkl"));
    }

    cflags
}

fn get_cflags_macos(install_dirs: Option<&InstallationDirectories>) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    if let Some(install_dirs) = install_dirs {
        cflags.push("-I".into());
        cflags.push(format!("{}", install_dirs.include_dir));
    }
    cflags
}

fn get_cflags(install_dirs: Option<&InstallationDirectories>) -> Vec<String> {
    if cfg!(target_os = "windows") {
        get_cflags_windows(install_dirs)
    } else if cfg!(target_os = "linux") {
        get_cflags_linux(install_dirs)
    } else if cfg!(target_os = "macos") {
        get_cflags_macos(install_dirs)
    } else {
        panic!("Target OS not supported");
    }
}

#[derive(Debug)]
pub struct Callbacks;

impl ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        // This forces all MKL constants to be signed. Otherwise `bindgen` might
        // give different types to different constants, which is inconvenient.
        // MKL expects these constants to be compatible with MKL_INT.
        if &name[..4] == "MKL_" {
            // Important: this should be the same as MKL_INT
            if cfg!(feature = "ilp64") {
                Some(IntKind::I64)
            } else {
                Some(IntKind::I32)
            }
        } else {
            None
        }
    }
}

fn main() {
    if cfg!(not(any(
        feature = "all",
        feature = "dss",
        feature = "axpy",
        feature = "sparse-matrix-checker",
        feature = "extended-eigensolver",
        feature = "inspector-executor"
    ))) {
        panic!(
            "No MKL modules selected.
To use this library, please select the features corresponding \
to MKL modules that you would like to use, or enable the `all` feature if you would \
like to generate symbols for all modules."
        );
    }

    // Link with the proper MKL libraries and simultaneously set up arguments for bindgen.
    // Otherwise we don't get e.g. the correct MKL preprocessor definitions).
    let clang_args = {
        // First try loading fully custom paths. This serves as a potential override to the "usual" way of installing MKL.
        // This is also useful when MKL is installed by another package manager like NuGet on windows.
        let install_dirs = InstallationDirectories::try_custom_env()
            .map_err(|err| println!("WARNING: {}", err))
            .ok()
            .or_else(|| {
                // Next try using the environment variable for ONEAPI_ROOT.
                InstallationDirectories::try_from_env_root()
                    .map_err(|err| println!("WARNING: {}", err))
                    .ok()
            })
            .or_else(|| {
                // Finally try a system installed version of MKL.
                InstallationDirectories::try_system()
                    .map_err(|err| println!("WARNING: {}", err))
                    .ok()
            });

        if let Some(install_dirs) = install_dirs.as_ref() {
            for lib_dir in get_lib_dirs(install_dirs) {
                println!("cargo:rustc-link-search=native={}", lib_dir);
            }
        }

        for lib in get_static_link_libs() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }

        for lib in get_dynamic_link_libs() {
            println!("cargo:rustc-link-lib={}", lib);
        }

        let args = get_cflags(install_dirs.as_ref());
        args
    };

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(Callbacks))
        .default_enum_style(EnumVariation::ModuleConsts)
        .impl_debug(true)
        .derive_debug(true)
        .clang_args(clang_args);

    // If only part of MKL is needed, we use features to construct whitelists of
    // the needed functionality. These can be overridden with the "all" feature, which
    // avoids whitelisting and instead encompasses everything.
    #[cfg(not(feature = "all"))]
    {
        #[cfg(feature = "axpy")]
        {
            builder = builder
                .allowlist_function("daxpy")
                .allowlist_function("cblas_daxpy")
        }
        #[cfg(feature = "dss")]
        {
            let dss_regex = "(dss_.*)|(DSS_.*)|(MKL_DSS.*)";
            builder = builder
                .allowlist_function(dss_regex)
                .allowlist_type(dss_regex)
                .allowlist_var(dss_regex);
        }

        #[cfg(feature = "sparse-matrix-checker")]
        {
            builder = builder
                .allowlist_function("sparse_matrix_checker*")
                .allowlist_function("sparse_matrix_checker_init*");
        }

        #[cfg(feature = "extended-eigensolver")]
        {
            builder = builder
                .allowlist_function(".*feast.*")
                .allowlist_function("mkl_sparse_ee_init")
                .allowlist_function("mkl_sparse_._svd")
                .allowlist_function("mkl_sparse_._ev")
                .allowlist_function("mkl_sparse_._gv");
        }

        #[cfg(feature = "inspector-executor")]
        {
            builder = builder.allowlist_function("mkl_sparse_.*");
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

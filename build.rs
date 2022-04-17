use bindgen::callbacks::{IntKind, ParseCallbacks};
use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

/// Paths required for linking to MKL from MKLROOT folder
struct OneAPIDirectories {
    lib_dir: String,
    omp_lib_dir: String,
    include_dir: String,
}

impl OneAPIDirectories {
    /// Constructs paths required for linking MKL from the specified root folder. Checks if paths exist.
    fn try_new(openapi_root: &str) -> Result<Self, String> {
        let os = if cfg!(target_os = "windows") {
            "win"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            return Err("Target OS not supported".into());
        };

        // TODO determine if we need to support ia32 as well
        let itype = "intel64";
        let omp_lib_subdir = if cfg!(target_os = "linux") {
            format!("/{}_lin", itype)
        } else {
            String::new()
        };

        let lib_subdir = if cfg!(target_os = "linux") {
            format!("/{}", itype)
        } else {
            String::new()
        };

        let mkl_root: String = format!("{}/mkl/latest", openapi_root);
        let compiler_root: String = format!("{}/compiler/latest", openapi_root);
        let lib_dir = format!(
            "{mkl_root}/lib{lib_subdir}",
        );
        let omp_lib_dir = format!(
            "{compiler_root}/{os}/compiler/lib{omp_lib_subdir}",
        );
        let include_dir = format!("{}/include", mkl_root);

        let mkl_root_path = PathBuf::from(mkl_root);
        let lib_dir_path = PathBuf::from(lib_dir);
        let omp_lib_dir_path = PathBuf::from(omp_lib_dir);
        let include_dir_path = PathBuf::from(include_dir);

        let mkl_root_str = mkl_root_path
            .to_str()
            .ok_or("Unable to convert 'mkl_root' to string")?;
        let lib_dir_str = lib_dir_path
            .to_str()
            .ok_or("Unable to convert 'lib_dir_path' to string")?;
        let omp_lib_dir_str = omp_lib_dir_path
            .to_str()
            .ok_or("Unable to convert 'omp_lib_dir_path' to string")?;
        let include_dir_str = include_dir_path
            .to_str()
            .ok_or("Unable to convert 'include_dir_path' to string")?;

        // Check if paths exist

        if !mkl_root_path.exists() {
            println!(
                "cargo:warning=The 'mkl_root' folder with path '{}' does not exist.",
                mkl_root_str
            );
        }

        if !lib_dir_path.exists() {
            println!(
                "cargo:warning=The 'lib_dir_path' folder with path '{}' does not exist.",
                lib_dir_str
            );
        }

        if cfg!(feature = "openmp") && !omp_lib_dir_path.exists() {
            println!(
                "cargo:warning=The 'omp_lib_dir_path' folder with path '{}' does not exist.",
                omp_lib_dir_str
            );
        }

        if !include_dir_path.exists() {
            println!(
                "cargo:warning=The 'include_dir_path' folder with path '{}' does not exist.",
                include_dir_str
            );
        }

        Ok(OneAPIDirectories {
            lib_dir: lib_dir_str.into(),
            omp_lib_dir: omp_lib_dir_str.into(),
            include_dir: include_dir_str.into(),
        })
    }
}

fn get_lib_dirs(oneapi_dirs: &OneAPIDirectories) -> Vec<String> {
    if cfg!(feature = "openmp") {
        vec![oneapi_dirs.lib_dir.clone(), oneapi_dirs.omp_lib_dir.clone()]
    } else {
        vec![oneapi_dirs.lib_dir.clone()]
    }
}

fn get_dynamic_link_libs_windows() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    if cfg!(feature = "ilp64") {
        libs.push("mkl_intel_ilp64_dll");
    } else {
        libs.push("mkl_intel_lp64_dll");
    };

    if cfg!(feature = "openmp") {
        libs.push("mkl_intel_thread_dll");
    } else {
        libs.push("mkl_sequential_dll");
    };

    libs.push("mkl_core_dll");

    if cfg!(feature = "openmp") {
        libs.push("libiomp5md");
    }

    libs.into_iter().map(|s| s.into()).collect()
}

fn get_dynamic_link_libs_linux() -> Vec<String> {
    // Note: The order of the libraries is very important
    let mut libs = Vec::new();

    // if cfg!(feature = "ilp64") {
    //     libs.push("mkl_intel_ilp64");
    // } else {
    //     libs.push("mkl_intel_lp64");
    // };
    //
    // if cfg!(feature = "openmp") {
    //     libs.push("mkl_intel_thread");
    // } else {
    //     libs.push("mkl_sequential");
    // };
    //
    // libs.push("mkl_core");

    if cfg!(feature = "openmp") {
        libs.push("iomp5");
    }
    libs.extend(vec!["pthread", "m", "dl"]);

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
        vec![]
    } else if cfg!(target_os = "linux") {
        get_static_link_libs_linux()
    } else if cfg!(target_os = "macos") {
        get_static_link_libs_macos()
    } else {
        panic!("Target OS not supported");
    }
}

fn get_cflags_windows(oneapi_dirs: &OneAPIDirectories) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    cflags.push("--include-directory".into());
    cflags.push(format!("{}", oneapi_dirs.include_dir));
    cflags
}

fn get_cflags_linux(oneapi_dirs: &OneAPIDirectories) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    cflags.push("-I".into());
    cflags.push(format!("{}", oneapi_dirs.include_dir));
    cflags
}

fn get_cflags_macos(oneapi_dirs: &OneAPIDirectories) -> Vec<String> {
    let mut cflags = Vec::new();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    cflags.push("-I".into());
    cflags.push(format!("{}", oneapi_dirs.include_dir));
    cflags
}

fn get_cflags(oneapi_dirs: &OneAPIDirectories) -> Vec<String> {
    if cfg!(target_os = "windows") {
        get_cflags_windows(oneapi_dirs)
    } else if cfg!(target_os = "linux") {
        get_cflags_linux(oneapi_dirs)
    } else if cfg!(target_os = "macos") {
        get_cflags_macos(oneapi_dirs)
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
        let oneapi_root = match env::var("ONEAPI_ROOT") {
            Ok(oneapi_root) => oneapi_root,
            Err(_) => panic!(
"Environment variable 'ONEAPI_ROOT' is not defined. Remember to run the setvars script bundled
with oneAPI in order to set up the required environment variables."),
        };

        let oneapi_dirs = OneAPIDirectories::try_new(&oneapi_root).unwrap();

        for lib_dir in get_lib_dirs(&oneapi_dirs) {
            println!("cargo:rustc-link-search=native={}", lib_dir);
        }

        for lib in get_dynamic_link_libs() {
            println!("cargo:rustc-link-lib={}", lib);
        }
        for lib in get_static_link_libs() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }

        let args = get_cflags(&oneapi_dirs);
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

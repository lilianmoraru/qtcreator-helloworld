extern crate cmake;
extern crate dotenv;

use std::process::Command;
use std::path::{ Path, PathBuf };
use cmake::Config;

fn main() {
    // Making sure that we clone the submodules
    let mut git_submodule_update = Command::new("git");
    run(git_submodule_update.args(&["submodule", "update", "--init", "--recursive"]),
        "git");

    // Try to find "QMAKE_PATH" environment variable
    let qmake_system_env = option_env!("QMAKE_PATH");
    let qmake_system_env = qmake_system_env.and_then(|path| {
        if path.is_empty() {
            None
        } else {
            Some(String::from(path))
        }
    });

    ::dotenv::dotenv().ok();
    let qmake_dotenv_env = ::std::env::var("QMAKE_PATH").ok();
    let qmake_dotenv_env = qmake_dotenv_env.and_then(|path| {
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    });

    // dotenv has priority(the user can overwrite system variables)
    let qmake_path = match qmake_dotenv_env.or(qmake_system_env) {
        Some(path) => path,
        None => fail("\"QMAKE_PATH\" was not defined. \
                     Please define the variable to point to the \"qmake\" binary"),
    };

    // Lets try to run "qmake --version" to make sure it doesn't throw an error
    let mut assert_qmake = Command::new(qmake_path);
    run(assert_qmake.arg("--version"), "qmake");

    // Compiling Clang and QtCreator
    let deps_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("deps");
    add_clang_link(deps_folder.as_path());
    compile_clang(deps_folder.as_path());
    compile_qtcreator(deps_folder.as_path());
}

fn compile_qtcreator(deps_folder: &Path) {
    // CMAKE_CXX_COMPILER = deps/clang-toolchain/bin/clang++
    // -C deps/cxxbasics/CXXBasics.cmake
}

fn compile_clang(deps_folder: &Path) {
    use std::ffi::OsString;

    // Can't use CXXBasics, limitation in the `cmake` crate
//    let cxxbasics_path = deps_folder.join("cxxbasics/CXXBasics.cmake");
    let install_path = deps_folder.join("clang-toolchain");
    if install_path.join("bin").join("clang").exists() {
        return;
    }

    let llvm_src_path = deps_folder.join("llvm");
    let build_path    = deps_folder.join("build-clang");

    // Select a generator
    let mut cmake_config = Config::new(llvm_src_path);
    if cfg!(unix) {
        cmake_config.generator("Unix Makefiles");
    } else if cfg!(windows) {
        cmake_config.generator("NMake Makefiles");
    } else {
        fail("Unsupported HOST system");
    }

    // Set install prefix
    cmake_config.define("CMAKE_INSTALL_PREFIX", install_path.into_os_string());

    // We always want to build clang in Release mode
    cmake_config.profile("Release");
    cmake_config.define("LLVM_ENABLE_RTTI", "ON");

    // Fire it up
    cmake_config.build();
}

fn add_clang_link(deps_folder: &Path) {
    let expected_clang_path = deps_folder.join("llvm").join("tools").join("clang");
    if !expected_clang_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(deps_folder.join("clang"), expected_clang_path).unwrap();
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(deps_folder.join("clang"), expected_clang_path).unwrap();
        }
    }
}

fn run(cmd: &mut Command, program: &str) {
    use std::io::ErrorKind;

    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            fail(&format!("failed to execute command: {}\nis `{}` not installed?",
                          e, program));
        }
        Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
    if !status.success() {
        fail(&format!("command did not execute successfully, got: {}", status));
    }
}

fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed, must exit now", s)
}
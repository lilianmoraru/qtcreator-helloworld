extern crate cmake;
extern crate dotenv;

use std::process::Command;
use std::path::{ Path, PathBuf };

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
    // destination: deps/clang-toolchain
}

fn add_clang_link(deps_folder: &Path) {
    // deps/llvm/tools/clang -> deps/clang
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
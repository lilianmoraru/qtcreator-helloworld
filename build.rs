extern crate cmake;
extern crate dotenv;
extern crate num_cpus;

use std::process::Command;
use std::path::{ Path, PathBuf };
use cmake::Config;

fn main() {
    let qmake_path = {
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
        match qmake_dotenv_env.or(qmake_system_env) {
            Some(path) => path,
            None => fail("\"QMAKE_PATH\" was not defined. \
                         Please define the variable to point to the \"qmake\" binary"),
        }
    };

    // Lets try to run "qmake --version" to make sure it doesn't throw an error
    {
        let mut assert_qmake = Command::new(&qmake_path);
        run(assert_qmake.arg("--version"), "qmake");
    }

    // Lets make sure we have `cmake` before doing heavier operations
    {
        let mut assert_cmake = Command::new("cmake");
        run(assert_cmake.arg("--version"), "cmake");
    }

    let deps_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("deps");
    // Making sure that we clone the submodules
    git_submodule_update(&deps_folder);

    // Compiling Clang and QtCreator
    add_clang_link(&deps_folder);
    compile_clang(&deps_folder);
    compile_qtcreator(&deps_folder, &qmake_path);
}

fn compile_qtcreator(deps_folder: &Path, qmake_path: &String) {
    // If the "qtcreator" binary already exists, don't try to build
    if deps_folder.join("qt-creator").join("build")
                  .join("bin").join("qtcreator").exists() {
        return;
    }

    // Lets also build Clang Code Model
    ::std::env::set_var("LLVM_INSTALL_DIR", deps_folder.join("clang-toolchain"));

    // Adding the newly compiled toolchain to the "PATH" environment variable
    {
        let clang_bin_path = deps_folder.join("clang-toolchain").join("bin");
        let mut path_final = String::from(clang_bin_path.to_str().unwrap());
        path_final.push(':');
        path_final.push_str(env!("PATH"));

        ::std::env::set_var("PATH", path_final);
    }

    let qtcreator_path  = deps_folder.join("qt-creator");
    let qtcreator_build = qtcreator_path.join("build");
    let qtcreator_src = qtcreator_path.to_str().unwrap();

    // Create build directory
    ::std::fs::create_dir_all(qtcreator_build.as_path()).unwrap();

    // Running "qmake CONFIG+=[PROFILE] -r"
    let mut qmake_command = Command::new(qmake_path.as_str());
    qmake_command.current_dir(qtcreator_build.as_path());

    let config = {
        let mut config = String::from("CONFIG+=");
        let profile = ::std::env::var("PROFILE").unwrap();
        config.push_str(profile.as_str());

        config
    };

    if cfg!(target_os = "linux") {
        qmake_command.args(&[&config, "-spec", "linux-clang", "-r", qtcreator_src]);
    } else if cfg!(macos) {
        qmake_command.args(&[&config, "-spec", "macx-clang", "-r", qtcreator_src]);
    } else {
        qmake_command.args(&[&config, "-r", qtcreator_src]);
    }
    run(&mut qmake_command, "qmake");

    // Lets fire it up( make -j $(nproc) )
    let cpus = ::num_cpus::get().to_string();
    let mut make_command = Command::new("make");
    make_command.current_dir(qtcreator_build.as_path());
    make_command.args(&["-j", &cpus]);
    run(&mut make_command, "make");
}

fn compile_clang(deps_folder: &Path) {
    // Can't use CXXBasics, limitation in the `cmake` crate
    //    let cxxbasics_path = deps_folder.join("cxxbasics/CXXBasics.cmake");
    let install_path = deps_folder.join("clang-toolchain");
    if install_path.join("bin").join("clang").exists() {
        return;
    }

    let llvm_src_path = deps_folder.join("llvm");

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

fn git_submodule_update(deps_folder: &Path) {
    let mut should_update = false;
    for path in &["clang", "cxxbasics", "llvm", "qt-creator"] {
        if !deps_folder.join(path).join(".git").exists() {
            should_update = true;
        }
    }

    if should_update {
        let mut git_submodule_update = Command::new("git");
        run(git_submodule_update.args(&["submodule", "update", "--init", "--recursive"]),
            "git");
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
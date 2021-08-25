use std::env;
use std::path::PathBuf;
use std::process::Command;

// spell-checker: disable

// Taken from ViGEmClient.vcxproj <AdditionalDependencies> as well as msvcrtd
const LIBS: &[&str] = &[
    "setupapi", "kernel32", "user32", "gdi32", "winspool", "comdlg32", "advapi32", "shell32",
    "ole32", "oleaut32", "uuid", "odbc32", "odbccp32",
];

fn main() {
    // Find the finder by using environment variables.. kinda ironic
    let vswhere =
        env::var("PROGRAMFILES(X86)").unwrap() + r"\Microsoft Visual Studio\Installer\vswhere.exe";

    // Find msbuild using vswhere
    let msbuild = String::from_utf8(
        Command::new(vswhere)
            .args(&[
                "-latest",
                "-prerelease",
                "-products",
                "*",
                "-requires",
                "Microsoft.Component.MSBuild",
                "-find",
                r"MSBuild\**\Bin\MSBuild.exe",
            ])
            .output()
            .expect("could not locate msbuild")
            .stdout,
    )
    .unwrap();

    // Build ViGemClient and check status
    let status = Command::new(msbuild.trim())
        .arg("src/ViGEmClient/ViGEmClient.sln")
        .status()
        .unwrap();
    assert!(status.success());

    // Link msvcrt
    println!("cargo:rustc-link-lib=msvcrtd");

    // Tell cargo to link all necessary windows libraries
    for lib in LIBS {
        println!("cargo:rustc-link-lib={}", lib)
    }

    // Tell cargo to link ViGemClient and where to find it
    println!(
        "cargo:rustc-link-search={}/src/ViGemClient/lib/debug/x64",
        env!("CARGO_MANIFEST_DIR")
    );
    println!("cargo:rustc-link-lib=static=ViGEmClient");

    // Generate bindings for ViGemClient
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .allowlist_type("vigem.*")
        .allowlist_function("vigem.*")
        .allowlist_var("vigem.*")
        .clang_arg("-Isrc/ViGEmClient/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

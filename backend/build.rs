use std::process::Command;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }

    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=../frontend/index.html");

    println!("cargo:warning=Building frontend...");

    let pnpm = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    let status = Command::new(pnpm)
        .args(&["build"])
        .current_dir("../frontend")
        .status();

    match status {
        Ok(code) => {
            if !code.success() {
                // Try npm fallback
                let status_npm = Command::new(npm)
                    .args(&["run", "build"])
                    .current_dir("../frontend")
                    .status();

                if let Ok(code_npm) = status_npm {
                    if !code_npm.success() {
                        println!(
                            "cargo:warning=Frontend build failed (npm exit code: {:?})",
                            code_npm.code()
                        );
                    }
                } else {
                    println!("cargo:warning=Frontend build failed (npm failed to run)");
                }
            }
        }
        Err(_) => {
            // Try npm fallback
            let status_npm = Command::new(npm)
                .args(&["run", "build"])
                .current_dir("../frontend")
                .status();

            if let Ok(code_npm) = status_npm {
                if !code_npm.success() {
                    println!(
                        "cargo:warning=Frontend build failed (npm exit code: {:?})",
                        code_npm.code()
                    );
                }
            } else {
                println!("cargo:warning=Could not run pnpm or npm");
            }
        }
    }
}

use std::process::Command;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }

    // Get version from git
    let output = Command::new("git")
        .args(&["describe", "--tags", "--always", "--dirty"])
        .output();

    let version = match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // Format: v0.1.0-6-g123abc -> 0.1.0+6
            // Format: v0.1.0 -> 0.1.0
            if s.starts_with('v') {
                let s = &s[1..];
                if let Some(pos) = s.find('-') {
                    let (tag, rest) = s.split_at(pos);
                    // rest is like -6-g123abc or -dirty
                    // extract the number of commits if present
                    let mut parts = rest.split('-');
                    // skip first empty string from split
                    let _ = parts.next();

                    if let Some(commits) = parts.next() {
                        // Check if it is a number (commits count)
                        if commits.chars().all(char::is_numeric) {
                            format!("{}+{}", tag, commits)
                        } else {
                            // maybe it's just -dirty
                            format!("{}", tag)
                        }
                    } else {
                        s.to_string()
                    }
                } else {
                    s.to_string()
                }
            } else {
                s.to_string()
            }
        }
        _ => "0.1.0".to_string(), // Fallback
    };

    println!("cargo:rustc-env=APP_VERSION={}", version);
    // Rerun if .git/HEAD changes (roughly)
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");

    println!("cargo:rerun-if-changed=../frontend/src");
    println!("cargo:rerun-if-changed=../frontend/package.json");
    println!("cargo:rerun-if-changed=../frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=../frontend/index.html");

    if std::env::var("SKIP_FRONTEND_BUILD").is_ok() {
        println!("cargo:warning=Skipping frontend build");
        return;
    }

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

// src/installer.rs

use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;
use include_dir::{include_dir, Dir};
use std::env;
use tinytemplate::TinyTemplate;
use std::collections::HashMap;
use crate::Args;

static SCRIPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates/service");

fn is_binary_in_path(binary_name: &str) -> bool {
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let binary_path = path.join(binary_name);
            if binary_path.exists() {
                return true;
            }
        }
    }
    false
}

/// Installs the service to the host system.
///
/// This function installs the currently running binary to `/usr/local/bin/<service_name>`.
/// It then creates the service directory structure in `/etc/sv/<service_name>`,
/// and symlink it to `/etc/service/<service_name>`.
///
/// Note that this function will not overwrite an existing service directory.
/// If the service is already installed, this function will return an error.
///
/// # Errors
///
/// This function will return an error if the service is already installed,
/// or if there is an error while creating the service directory structure.
pub fn install_service(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Install the binary if it's not in the PATH
    if !is_binary_in_path(&args.service_name) {
        let binary_path = env::current_exe()?;
        let target_path = format!("/usr/local/bin/{}", &args.service_name);

        if !std::path::Path::new(&target_path).exists() {
            fs::copy(&binary_path, &target_path)
                .map_err(|e| format!("Failed to copy binary to {}: {}", target_path, e))?;
            fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions on {}: {}", target_path, e))?;
        }
    }

    // Check that log directory exists
    if !std::path::Path::new(&args.log_directory).exists() {
        fs::create_dir_all(&args.log_directory)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }

    let service_dir = format!("/etc/sv/{}", &args.service_name);
    let log_dir = format!("{}/log", service_dir);

    // Create the service and log directories if they do not exist
    if !std::path::Path::new(&service_dir).exists() {
        fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create service directories: {}", e))?;
    }

    let mut tt = TinyTemplate::new();
    tt.add_template(
        "run",
        SCRIPTS_DIR
            .get_file("run")
            .expect("Missing run script")
            .contents_utf8()
            .unwrap(),
    )?;
    tt.add_template(
        "log_run",
        SCRIPTS_DIR
            .get_file("log_run")
            .expect("Missing log run script")
            .contents_utf8()
            .unwrap(),
    )?;

    let mut context = HashMap::new();
    context.insert("service_name", args.service_name.clone());
    context.insert("log_directory", args.log_directory.clone());
    context.insert("services_dir", args.services_dir.clone());
    context.insert("log_level", args.log_level.clone());

    let rendered_run_script = tt.render("run", &context)?;
    let rendered_log_run_script = tt.render("log_run", &context)?;

    fs::write(format!("{}/run", service_dir), rendered_run_script)
        .map_err(|e| format!("Failed to write run script: {}", e))?;
    fs::set_permissions(format!("{}/run", service_dir), fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions on run script: {}", e))?;

    fs::write(format!("{}/run", log_dir), rendered_log_run_script)
        .map_err(|e| format!("Failed to write log run script: {}", e))?;
    fs::set_permissions(format!("{}/run", log_dir), fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions on log run script: {}", e))?;

    // Create a symlink in /etc/service if it doesn't exist
    let symlink_path = format!("/etc/service/{}", &args.service_name);
    if !std::path::Path::new(&symlink_path).exists() {
        symlink(&service_dir, symlink_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    Ok(())
}

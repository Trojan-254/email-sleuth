use email_sleuth_core::{AppError, Config, Result};
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Default paths for service files
pub fn default_paths() -> (PathBuf, PathBuf, PathBuf) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let service_dir = PathBuf::from(&format!("{}/.email-sleuth/service", home));
    let drivers_dir = PathBuf::from(&format!("{}/.email-sleuth/drivers", home));

    // Create directories if they don't exist
    fs::create_dir_all(&service_dir).ok();
    fs::create_dir_all(&drivers_dir).ok();

    let pid_file = service_dir.join("chromedriver.pid");
    let log_file = service_dir.join("chromedriver.log");
    let driver_path = drivers_dir.join("chromedriver");

    (driver_path, pid_file, log_file)
}

/// Detects the ChromeDriver executable location
pub fn detect_driver_path(config: &Config) -> Result<PathBuf> {
    if let Some(ref custom_path) = config.chromedriver_path {
        if !custom_path.is_empty() {
            let path = PathBuf::from(custom_path);
            if path.exists() && path.is_file() {
                tracing::info!("Using user-specified ChromeDriver path: {}", path.display());
                return Ok(path);
            } else {
                tracing::warn!(
                    "Specified ChromeDriver path '{}' not found or is not a file",
                    custom_path
                );
            }
        }
    }

    let (default_driver_path, _, _) = default_paths();

    if default_driver_path.exists() && default_driver_path.is_file() {
        return Ok(default_driver_path);
    }

    let mut common_paths = Vec::new();

    // Unix-like systems (Linux, macOS)
    #[cfg(unix)]
    {
        // Common to both Linux and macOS
        common_paths.push("/usr/local/bin/chromedriver");
        common_paths.push("/usr/bin/chromedriver");

        // macOS specific paths
        #[cfg(target_os = "macos")]
        {
            common_paths.push("/Applications/Google Chrome.app/Contents/MacOS/chromedriver");

            // Homebrew on Intel Macs
            common_paths.push("/usr/local/Cellar/chromedriver/latest/bin/chromedriver");

            // Homebrew on Apple Silicon
            common_paths.push("/opt/homebrew/bin/chromedriver");

            // User Applications folder
            if let Ok(home) = std::env::var("HOME") {
                common_paths.push(&format!("{}/Applications/chromedriver", home));
            }
        }
    }

    // Windows-specific paths
    #[cfg(windows)]
    {
        common_paths.push("C:\\Program Files\\Google\\Chrome\\Application\\chromedriver.exe");
        common_paths.push("C:\\Program Files (x86)\\Google\\Chrome\\Application\\chromedriver.exe");

        // User profile location
        if let Ok(profile) = std::env::var("USERPROFILE") {
            common_paths.push(&format!(
                "{}\\AppData\\Local\\Google\\Chrome\\Application\\chromedriver.exe",
                profile
            ));
        }
    }

    for path_str in common_paths {
        let path = PathBuf::from(path_str);
        if path.exists() && path.is_file() {
            tracing::debug!("Found ChromeDriver at: {}", path.display());
            return Ok(path);
        }
    }

    Err(AppError::Initialization(
        "ChromeDriver executable not found. Please install it or specify its location.".to_string(),
    ))
}

/// Checks if ChromeDriver is responsive
async fn is_responsive() -> bool {
    match reqwest::Client::new()
        .get("http://localhost:4444/status")
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => true,
        _ => false,
    }
}

/// Starts the ChromeDriver service
pub async fn start(config: &Config) -> Result<()> {
    let (_, pid_file, log_file) = default_paths();

    let driver_path = detect_driver_path(config)?;

    if pid_file.exists() {
        let pid_str = fs::read_to_string(&pid_file)?;
        let pid = pid_str
            .trim()
            .parse::<u32>()
            .map_err(|e| AppError::Initialization(format!("Invalid PID in file: {}", e)))?;

        // On Unix, check if process exists
        #[cfg(unix)]
        {
            let output = Command::new("ps").arg("-p").arg(pid.to_string()).output()?;

            if output.status.success() {
                // Process exists
                tracing::info!("ChromeDriver already running with PID: {}", pid);

                // Check if responsive
                if is_responsive().await {
                    tracing::info!("ChromeDriver service is responsive at http://localhost:4444");
                    return Ok(());
                } else {
                    tracing::warn!(
                        "ChromeDriver process exists but is not responsive. Restarting..."
                    );
                    stop(config).await?;
                }
            } else {
                // Process doesn't exist, remove stale PID file
                tracing::warn!("Found stale PID file, removing");
                fs::remove_file(&pid_file)?;
            }
        }

        // Just leaving Windows for the moment, I'm not sure how to do it.
        // #[cfg(windows)]
        // {
        //     tracing::warn!("Found existing PID file but cannot verify process on this platform. Attempting restart.");
        //     stop(config).await?;
        // }
    }

    // Start ChromeDriver
    tracing::info!("Starting ChromeDriver at {}", driver_path.display());

    // Ensure log file directory exists
    if let Some(log_dir) = log_file.parent() {
        fs::create_dir_all(log_dir)?;
    }

    let log_file_handle = File::create(&log_file)?;

    let child = Command::new(&driver_path)
        .arg("--port=4444")
        .arg("--whitelisted-ips=\"\"")
        .stdout(std::process::Stdio::from(log_file_handle.try_clone()?))
        .stderr(std::process::Stdio::from(log_file_handle))
        .spawn()?;

    let pid = child.id();
    fs::write(&pid_file, pid.to_string())?;

    // Give it a moment to start
    sleep(Duration::from_secs(2)).await;

    // Check if responsive
    if !is_responsive().await {
        sleep(Duration::from_secs(3)).await;
        if !is_responsive().await {
            tracing::error!("ChromeDriver started but is not responsive");
            return Err(AppError::Initialization(
                "ChromeDriver started but is not responding at http://localhost:4444".to_string(),
            ));
        }
    }

    tracing::info!("ChromeDriver started successfully with PID {}", pid);
    Ok(())
}

/// Stops the ChromeDriver service
pub async fn stop(_config: &Config) -> Result<()> {
    let (_, pid_file, _) = default_paths();

    if !pid_file.exists() {
        tracing::info!("ChromeDriver is not running (no PID file found)");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid = match pid_str.trim().parse::<u32>() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Invalid PID in file: {}", e);
            fs::remove_file(&pid_file)?;
            return Ok(());
        }
    };

    tracing::info!("Stopping ChromeDriver (PID: {})", pid);

    // Kill process - platform-specific code
    #[cfg(unix)]
    {
        Command::new("kill").arg(pid.to_string()).output()?;

        for _ in 0..10 {
            let output = Command::new("ps").arg("-p").arg(pid.to_string()).output()?;

            if !output.status.success() {
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }

        // Force kill if still running
        let output = Command::new("ps").arg("-p").arg(pid.to_string()).output()?;

        if output.status.success() {
            tracing::warn!("ChromeDriver did not terminate gracefully, forcing...");
            Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output()?;
        }
    }

    // Again leaving this.
    // #[cfg(windows)]
    // {
    //     Command::new("taskkill")
    //         .args(&["/F", "/PID", &pid.to_string()])
    //         .output()?;
    // }

    // Remove PID file
    fs::remove_file(&pid_file)?;
    tracing::info!("ChromeDriver stopped");

    Ok(())
}

/// Checks the status of the ChromeDriver service
pub async fn status(_config: &Config) -> Result<bool> {
    let (_, pid_file, _) = default_paths();

    if !pid_file.exists() {
        tracing::info!("ChromeDriver is not running (no PID file found)");
        return Ok(false);
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid = match pid_str.trim().parse::<u32>() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Invalid PID in file: {}", e);
            fs::remove_file(&pid_file)?;
            return Ok(false);
        }
    };

    // Check if process is running - platform-specific code
    #[cfg(unix)]
    {
        let output = Command::new("ps").arg("-p").arg(pid.to_string()).output()?;

        if !output.status.success() {
            tracing::info!("ChromeDriver is not running (stale PID file)");
            fs::remove_file(&pid_file)?;
            return Ok(false);
        }
    }

    // Check if the service is responsive
    let is_responsive = is_responsive().await;

    if is_responsive {
        tracing::info!("ChromeDriver is running with PID {} and is responsive", pid);
    } else {
        tracing::warn!(
            "ChromeDriver process exists (PID {}) but is not responding",
            pid
        );
    }

    Ok(is_responsive)
}

/// Gets the recent logs from the ChromeDriver service
pub fn logs(lines: usize) -> Result<String> {
    let (_, _, log_file) = default_paths();

    if !log_file.exists() {
        return Err(AppError::Initialization(
            "ChromeDriver log file not found".to_string(),
        ));
    }

    let content = fs::read_to_string(&log_file)?;
    let log_lines: Vec<&str> = content.lines().collect();

    let lines_to_show = std::cmp::min(lines, log_lines.len());
    let start_idx = log_lines.len().saturating_sub(lines_to_show);

    let recent_logs = log_lines[start_idx..].join("\n");
    Ok(recent_logs)
}

/// Restarts the ChromeDriver service
pub async fn restart(config: &Config) -> Result<()> {
    stop(config).await?;

    // Small delay before restart
    sleep(Duration::from_secs(1)).await;

    start(config).await
}

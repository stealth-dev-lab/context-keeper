//! ADB/Fastboot collector - detects connected Android devices

use crate::context::AdbDevice;

/// Collect connected ADB and Fastboot devices
pub fn collect_adb_devices() -> Vec<AdbDevice> {
    let mut devices = Vec::new();

    // Collect ADB devices
    if let Ok(output) = std::process::Command::new("adb")
        .args(["devices", "-l"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                // Skip "List of devices attached"
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let serial = parts[0].to_string();
                    let state = parts[1].to_string();

                    // Skip offline devices
                    if state == "offline" {
                        continue;
                    }

                    devices.push(AdbDevice {
                        serial,
                        state,
                        device_type: "adb".to_string(),
                    });
                }
            }
        }
    }

    // Collect Fastboot devices
    if let Ok(output) = std::process::Command::new("fastboot")
        .args(["devices", "-l"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let serial = parts[0].to_string();
                    devices.push(AdbDevice {
                        serial,
                        state: "fastboot".to_string(),
                        device_type: "fastboot".to_string(),
                    });
                }
            }
        }
    }

    devices
}

use crate::state::{ConnectionConfig, DatabaseType};
use anyhow::Result;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "com.slick-dataui.app";
const CONFIG_DIR_NAME: &str = "slick-dataui";
const CONNECTIONS_FILE_NAME: &str = "connections.json";

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionMetadata {
    name: String,
    db_type: DatabaseType,
    // Fallback for when system keyring fails (e.g. dev environment)
    // This is NOT secure, but ensures the app works.
    unsafe_password: Option<String>,
}

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join(CONFIG_DIR_NAME);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

fn get_connections_file_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(CONNECTIONS_FILE_NAME))
}

pub fn save_connections(connections: &[ConnectionConfig]) -> Result<()> {
    println!("Saving {} connections...", connections.len());
    // 1. Save metadata to JSON
    let metadata: Vec<ConnectionMetadata> = connections
        .iter()
        .map(|c| ConnectionMetadata {
            name: c.name.clone(),
            db_type: c.db_type.clone(),
            // We store the password here as a fallback.
            // In a production app, we would encrypt this or strictly enforce keyring.
            unsafe_password: Some(c.connection_string.clone()),
        })
        .collect();

    let file_path = get_connections_file_path()?;
    let json = serde_json::to_string_pretty(&metadata)?;
    fs::write(file_path, json)?;

    // 2. Save secrets to Keyring (Best effort)
    for conn in connections {
        println!("Saving password for '{}'", conn.name);
        let entry = Entry::new(SERVICE_NAME, &conn.name);

        if let Ok(entry) = entry
            && let Err(e) = entry.set_password(&conn.connection_string)
        {
            eprintln!(
                "Failed to save password to keyring for '{}': {}",
                conn.name, e
            );
        }
    }

    println!("Connections saved successfully.");
    Ok(())
}

pub fn load_connections() -> Result<Vec<ConnectionConfig>> {
    println!("Loading connections...");
    let file_path = get_connections_file_path()?;
    if !file_path.exists() {
        println!("No connections file found.");
        return Ok(vec![]);
    }

    let json = fs::read_to_string(file_path)?;
    let metadata: Vec<ConnectionMetadata> = serde_json::from_str(&json)?;
    println!("Found metadata for {} connections.", metadata.len());

    let mut connections = Vec::new();

    for meta in metadata {
        let mut password = String::new();
        let mut found_in_keyring = false;

        // Try Keyring first
        let entry = Entry::new(SERVICE_NAME, &meta.name);
        if let Ok(entry) = entry
            && let Ok(p) = entry.get_password()
        {
            password = p;
            found_in_keyring = true;
            println!("Loaded password for '{}' from Keyring", meta.name);
        }

        // Fallback to JSON if not found in keyring
        if !found_in_keyring {
            if let Some(unsafe_pass) = meta.unsafe_password {
                password = unsafe_pass;
                println!("Loaded password for '{}' from JSON fallback", meta.name);
            } else {
                eprintln!(
                    "Failed to get password for connection {}: Not in Keyring or JSON",
                    meta.name
                );
            }
        }

        connections.push(ConnectionConfig {
            name: meta.name,
            db_type: meta.db_type,
            connection_string: password,
        });
    }

    println!("Successfully loaded {} connections.", connections.len());
    Ok(connections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_direct() {
        let service = "com.slick-dataui.app.test";
        let user = "test_user";
        let password = "test_password";

        let entry = Entry::new(service, user).unwrap();

        // Clean up potential leftovers
        let _ = entry.delete_credential();

        // Set
        entry.set_password(password).unwrap();

        // Get
        let retrieved = entry.get_password().unwrap();
        assert_eq!(retrieved, password);

        // Delete
        entry.delete_credential().unwrap();
    }
}

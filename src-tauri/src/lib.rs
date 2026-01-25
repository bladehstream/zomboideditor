mod parsers;

use parsers::{SandboxSettings, PlayerData, PlayersDatabase};
use parsers::sandbox::SandboxValue;
use parsers::player::{Perk, SkillData};
use parsers::database::{PlayerRecord, WhitelistEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// Application state holding loaded data
struct AppState {
    sandbox: Mutex<Option<(PathBuf, SandboxSettings)>>,
    player: Mutex<Option<(PathBuf, PlayerData)>>,
    database: Mutex<Option<PlayersDatabase>>,
}

/// Response type for API calls
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn err(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

/// Information about the loaded save
#[derive(Debug, Serialize)]
struct SaveInfo {
    sandbox_loaded: bool,
    sandbox_path: Option<String>,
    sandbox_settings_count: usize,
    player_loaded: bool,
    player_path: Option<String>,
    player_name: Option<String>,
    database_loaded: bool,
    database_path: Option<String>,
    player_count: usize,
}

/// Available perks info for the UI
#[derive(Debug, Serialize)]
struct PerkInfo {
    name: String,
    display_name: String,
    category: String,
    max_level: i32,
}

// ==================== SANDBOX COMMANDS ====================

#[tauri::command]
fn load_sandbox(path: String, state: State<AppState>) -> ApiResponse<SandboxSettings> {
    match fs::read(&path) {
        Ok(data) => {
            match SandboxSettings::parse(&data) {
                Ok(settings) => {
                    let mut sandbox = state.sandbox.lock().unwrap();
                    *sandbox = Some((PathBuf::from(&path), settings.clone()));
                    ApiResponse::ok(settings)
                }
                Err(e) => ApiResponse::err(&format!("Failed to parse sandbox file: {}", e)),
            }
        }
        Err(e) => ApiResponse::err(&format!("Failed to read file: {}", e)),
    }
}

#[tauri::command]
fn save_sandbox(path: Option<String>, state: State<AppState>) -> ApiResponse<String> {
    let sandbox = state.sandbox.lock().unwrap();

    match sandbox.as_ref() {
        Some((original_path, settings)) => {
            let save_path = path.map(PathBuf::from).unwrap_or_else(|| original_path.clone());

            match settings.serialize() {
                Ok(data) => {
                    match fs::write(&save_path, data) {
                        Ok(_) => ApiResponse::ok(save_path.to_string_lossy().to_string()),
                        Err(e) => ApiResponse::err(&format!("Failed to write file: {}", e)),
                    }
                }
                Err(e) => ApiResponse::err(&format!("Failed to serialize: {}", e)),
            }
        }
        None => ApiResponse::err("No sandbox file loaded"),
    }
}

#[tauri::command]
fn get_sandbox_settings(state: State<AppState>) -> ApiResponse<SandboxSettings> {
    let sandbox = state.sandbox.lock().unwrap();

    match sandbox.as_ref() {
        Some((_, settings)) => ApiResponse::ok(settings.clone()),
        None => ApiResponse::err("No sandbox file loaded"),
    }
}

#[tauri::command]
fn update_sandbox_setting(key: String, value: SandboxValue, state: State<AppState>) -> ApiResponse<()> {
    let mut sandbox = state.sandbox.lock().unwrap();

    match sandbox.as_mut() {
        Some((_, settings)) => {
            settings.set(key, value);
            ApiResponse::ok(())
        }
        None => ApiResponse::err("No sandbox file loaded"),
    }
}

#[tauri::command]
fn get_sandbox_metadata() -> ApiResponse<Vec<parsers::sandbox::SettingMetadata>> {
    ApiResponse::ok(SandboxSettings::get_all_metadata())
}

// ==================== PLAYER COMMANDS ====================

#[tauri::command]
fn load_player(path: String, state: State<AppState>) -> ApiResponse<PlayerData> {
    match fs::read(&path) {
        Ok(data) => {
            match PlayerData::parse(&data) {
                Ok(player) => {
                    let mut player_state = state.player.lock().unwrap();
                    *player_state = Some((PathBuf::from(&path), player.clone()));
                    ApiResponse::ok(player)
                }
                Err(e) => ApiResponse::err(&format!("Failed to parse player file: {}", e)),
            }
        }
        Err(e) => ApiResponse::err(&format!("Failed to read file: {}", e)),
    }
}

#[tauri::command]
fn save_player(path: Option<String>, state: State<AppState>) -> ApiResponse<String> {
    let player = state.player.lock().unwrap();

    match player.as_ref() {
        Some((original_path, data)) => {
            let save_path = path.map(PathBuf::from).unwrap_or_else(|| original_path.clone());

            match data.serialize() {
                Ok(bytes) => {
                    match fs::write(&save_path, bytes) {
                        Ok(_) => ApiResponse::ok(save_path.to_string_lossy().to_string()),
                        Err(e) => ApiResponse::err(&format!("Failed to write file: {}", e)),
                    }
                }
                Err(e) => ApiResponse::err(&format!("Failed to serialize: {}", e)),
            }
        }
        None => ApiResponse::err("No player file loaded"),
    }
}

#[tauri::command]
fn get_player_data(state: State<AppState>) -> ApiResponse<PlayerData> {
    let player = state.player.lock().unwrap();

    match player.as_ref() {
        Some((_, data)) => ApiResponse::ok(data.clone()),
        None => ApiResponse::err("No player file loaded"),
    }
}

#[tauri::command]
fn update_player_skill(perk: String, level: i32, state: State<AppState>) -> ApiResponse<()> {
    let mut player = state.player.lock().unwrap();

    match player.as_mut() {
        Some((_, data)) => {
            data.set_skill_level(&perk, level);
            ApiResponse::ok(())
        }
        None => ApiResponse::err("No player file loaded"),
    }
}

#[tauri::command]
fn update_player_position(x: f32, y: f32, z: i32, state: State<AppState>) -> ApiResponse<()> {
    let mut player = state.player.lock().unwrap();

    match player.as_mut() {
        Some((_, data)) => {
            data.position.x = x;
            data.position.y = y;
            data.position.z = z;
            ApiResponse::ok(())
        }
        None => ApiResponse::err("No player file loaded"),
    }
}

#[tauri::command]
fn heal_player(state: State<AppState>) -> ApiResponse<()> {
    let mut player = state.player.lock().unwrap();

    match player.as_mut() {
        Some((_, data)) => {
            data.heal_all();
            data.reset_stats();
            ApiResponse::ok(())
        }
        None => ApiResponse::err("No player file loaded"),
    }
}

#[tauri::command]
fn get_all_perks() -> ApiResponse<Vec<PerkInfo>> {
    let perks: Vec<PerkInfo> = Perk::all().iter().map(|p| PerkInfo {
        name: format!("{:?}", p),
        display_name: p.display_name().to_string(),
        category: p.category().to_string(),
        max_level: p.max_level(),
    }).collect();

    ApiResponse::ok(perks)
}

// ==================== DATABASE COMMANDS ====================

#[tauri::command]
fn load_database(path: String, state: State<AppState>) -> ApiResponse<Vec<PlayerRecord>> {
    match PlayersDatabase::open(&path) {
        Ok(db) => {
            match db.get_all_players() {
                Ok(players) => {
                    let mut db_state = state.database.lock().unwrap();
                    *db_state = Some(db);
                    ApiResponse::ok(players)
                }
                Err(e) => ApiResponse::err(&format!("Failed to read players: {}", e)),
            }
        }
        Err(e) => ApiResponse::err(&format!("Failed to open database: {}", e)),
    }
}

#[tauri::command]
fn get_database_players(state: State<AppState>) -> ApiResponse<Vec<PlayerRecord>> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.get_all_players() {
                Ok(players) => ApiResponse::ok(players),
                Err(e) => ApiResponse::err(&format!("Failed to read players: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn update_player_display_name(username: String, display_name: String, state: State<AppState>) -> ApiResponse<()> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.update_display_name(&username, &display_name) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(&format!("Failed to update: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn update_database_player_position(username: String, x: f64, y: f64, z: i32, state: State<AppState>) -> ApiResponse<()> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.update_position(&username, x, y, z) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(&format!("Failed to update: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn set_player_admin(username: String, is_admin: bool, state: State<AppState>) -> ApiResponse<()> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.set_admin(&username, is_admin) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(&format!("Failed to update: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn set_player_banned(username: String, is_banned: bool, state: State<AppState>) -> ApiResponse<()> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.set_banned(&username, is_banned) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(&format!("Failed to update: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn delete_database_player(username: String, state: State<AppState>) -> ApiResponse<()> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.delete_player(&username) {
                Ok(_) => ApiResponse::ok(()),
                Err(e) => ApiResponse::err(&format!("Failed to delete: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn get_whitelist(state: State<AppState>) -> ApiResponse<Vec<WhitelistEntry>> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.get_whitelist() {
                Ok(entries) => ApiResponse::ok(entries),
                Err(e) => ApiResponse::err(&format!("Failed to read whitelist: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

#[tauri::command]
fn get_database_schema(state: State<AppState>) -> ApiResponse<Vec<(String, String)>> {
    let db = state.database.lock().unwrap();

    match db.as_ref() {
        Some(database) => {
            match database.get_schema_info() {
                Ok(schema) => ApiResponse::ok(schema),
                Err(e) => ApiResponse::err(&format!("Failed to read schema: {}", e)),
            }
        }
        None => ApiResponse::err("No database loaded"),
    }
}

// ==================== UTILITY COMMANDS ====================

#[tauri::command]
fn get_save_info(state: State<AppState>) -> SaveInfo {
    let sandbox = state.sandbox.lock().unwrap();
    let player = state.player.lock().unwrap();
    let database = state.database.lock().unwrap();

    SaveInfo {
        sandbox_loaded: sandbox.is_some(),
        sandbox_path: sandbox.as_ref().map(|(p, _)| p.to_string_lossy().to_string()),
        sandbox_settings_count: sandbox.as_ref().map(|(_, s)| s.settings.len()).unwrap_or(0),
        player_loaded: player.is_some(),
        player_path: player.as_ref().map(|(p, _)| p.to_string_lossy().to_string()),
        player_name: player.as_ref().map(|(_, d)| d.player_name.clone()),
        database_loaded: database.is_some(),
        database_path: database.as_ref().map(|d| d.path().to_string()),
        player_count: database.as_ref().and_then(|d| d.get_all_players().ok()).map(|p| p.len()).unwrap_or(0),
    }
}

#[tauri::command]
fn get_default_save_path() -> ApiResponse<String> {
    // Windows default path
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let path = PathBuf::from(user_profile).join("Zomboid").join("Saves");
        if path.exists() {
            return ApiResponse::ok(path.to_string_lossy().to_string());
        }
    }

    // Linux/Wine path
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(home).join("Zomboid").join("Saves");
        if path.exists() {
            return ApiResponse::ok(path.to_string_lossy().to_string());
        }
    }

    ApiResponse::err("Could not find default Zomboid saves path")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            sandbox: Mutex::new(None),
            player: Mutex::new(None),
            database: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            // Sandbox
            load_sandbox,
            save_sandbox,
            get_sandbox_settings,
            update_sandbox_setting,
            get_sandbox_metadata,
            // Player
            load_player,
            save_player,
            get_player_data,
            update_player_skill,
            update_player_position,
            heal_player,
            get_all_perks,
            // Database
            load_database,
            get_database_players,
            update_player_display_name,
            update_database_player_position,
            set_player_admin,
            set_player_banned,
            delete_database_player,
            get_whitelist,
            get_database_schema,
            // Utility
            get_save_info,
            get_default_save_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

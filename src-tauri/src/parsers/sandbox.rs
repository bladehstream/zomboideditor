use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Represents a value in the sandbox settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum SandboxValue {
    Boolean(bool),
    Integer(i32),
    Float(f64),
    String(String),
    Enum(i32), // Stored as integer but represents enum choices
}

impl SandboxValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SandboxValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            SandboxValue::Integer(i) | SandboxValue::Enum(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            SandboxValue::Float(f) => Some(*f),
            SandboxValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            SandboxValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Metadata about a sandbox setting for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingMetadata {
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
    pub value_type: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub enum_options: Option<Vec<String>>,
}

/// Sandbox settings parsed from map_sand.bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSettings {
    pub version: i32,
    pub settings: HashMap<String, SandboxValue>,
    #[serde(skip)]
    raw_data: Vec<u8>,
}

impl Default for SandboxSettings {
    fn default() -> Self {
        Self {
            version: 5,
            settings: HashMap::new(),
            raw_data: Vec::new(),
        }
    }
}

impl SandboxSettings {
    /// Parse sandbox settings from map_sand.bin file bytes
    pub fn parse(data: &[u8]) -> Result<Self, SandboxError> {
        let mut settings = SandboxSettings {
            version: 5,
            settings: HashMap::new(),
            raw_data: data.to_vec(),
        };

        let mut cursor = Cursor::new(data);

        // The file format appears to be a series of:
        // - String (prefixed with length as big-endian i16)
        // - Value (type depends on the setting)

        // Try to detect the format by looking for known patterns
        // Project Zomboid uses Java's DataOutputStream format

        while cursor.position() < data.len() as u64 {
            // Try to read a string length (2 bytes, big-endian)
            let str_len = match cursor.read_i16::<BigEndian>() {
                Ok(len) if len > 0 && len < 1000 => len as usize,
                _ => break,
            };

            // Read the string
            let mut str_buf = vec![0u8; str_len];
            if cursor.read_exact(&mut str_buf).is_err() {
                break;
            }

            let key = match String::from_utf8(str_buf) {
                Ok(s) => s,
                Err(_) => break,
            };

            // Skip empty or invalid keys
            if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
                continue;
            }

            // Try to determine value type based on known settings
            let value = Self::read_value_for_key(&key, &mut cursor)?;

            if let Some(v) = value {
                settings.settings.insert(key, v);
            }
        }

        // If we couldn't parse structured data, try alternate format
        if settings.settings.is_empty() {
            settings.parse_alternate_format(data)?;
        }

        Ok(settings)
    }

    /// Read a value based on the known type for a key
    fn read_value_for_key(key: &str, cursor: &mut Cursor<&[u8]>) -> Result<Option<SandboxValue>, SandboxError> {
        // Known boolean settings
        let boolean_settings = [
            "Electricity", "Water", "Helicopter", "AllClothesUnlocked",
            "EnableVehicles", "ZombiesDragDown", "ZombiesEatBody", "ZombiesEatFlesh",
            "ZombiesFenceClimber", "ZombiesAttackDoors", "ZombiesAttackWindows",
            "LockedHouses", "StartingKit", "OpenAllHours", "DoBurnCorpses",
            "NoFire", "MultiHitZombies", "RearVulnerability", "AttackBlockMovements",
            "AllowExteriorGenerator", "RemoveZombiesCorpses", "SleepAllowed",
            "SleepNeeded", "InjurySeverity", "BoneFracture", "EnablePoisoning",
        ];

        // Known integer/enum settings
        let integer_settings = [
            "Zombies", "Distribution", "DayLength", "StartYear", "StartMonth",
            "StartDay", "StartTime", "WaterShut", "WaterShutModifier",
            "ElecShut", "ElecShutModifier", "FoodLoot", "WeaponLoot", "AmmoLoot",
            "MedicalLoot", "MechanicsLoot", "LiteratureLoot", "SurvivalLoot",
            "OtherLoot", "Temperature", "Rain", "ErosionSpeed", "ErosionDays",
            "XpMultiplier", "StatsDecrease", "NatureAbundance", "Alarm",
            "LockedHouses", "StarterKit", "Nutrition", "PlantResilience",
            "PlantAbundance", "CompostTime", "FarmingNeverToDry", "Farming",
            "CarSpawnRate", "ChanceHasGas", "InitialGas", "FuelStationGas",
            "LockedCar", "GeneratorSpawning", "GeneratorFuelConsumption",
            "SurvivorHouseChance", "VehicleEasyToFind", "ZombiesRespawn",
            "ZombiesRespawnDelay", "ZombiesRespawnAmount", "Speed", "Strength",
            "Toughness", "Transmission", "Mortality", "Reanimate", "Cognition",
            "Memory", "Decomp", "Sight", "Hearing", "ThumpNoChasing",
            "ThumpOnConstruction", "ActiveOnly", "TriggerHouseAlarm",
            "ZombiesDragDown", "Population", "PopulationModifier", "PopulationPeak",
            "PopulationPeakDay", "PopulationStart", "RespawnHours", "RespawnUnseenHours",
            "RespawnMultiplier", "RedistributeHours", "FollowSoundDistance",
            "RallyGroupSize", "RallyTravelDistance", "RallyGroupSeparation",
            "RallyGroupRadius",
        ];

        // Known float settings
        let float_settings = [
            "ZombieUpdateDelta", "HoursForCorpseRemoval", "DecayingCorpseHealthImpact",
            "BloodLevel", "ClothingDegradation",
        ];

        // Try to read the appropriate type
        if boolean_settings.iter().any(|s| key.contains(s)) {
            // Boolean: stored as single byte
            let val = cursor.read_u8()?;
            return Ok(Some(SandboxValue::Boolean(val != 0)));
        } else if float_settings.iter().any(|s| key.contains(s)) {
            // Float: stored as f64 big-endian
            let val = cursor.read_f64::<BigEndian>()?;
            return Ok(Some(SandboxValue::Float(val)));
        } else if integer_settings.iter().any(|s| key.contains(s)) {
            // Integer: stored as i32 big-endian
            let val = cursor.read_i32::<BigEndian>()?;
            return Ok(Some(SandboxValue::Integer(val)));
        }

        // For unknown settings, try to guess the type
        // Peek at the next few bytes
        let pos = cursor.position();

        // Try reading as int32
        if let Ok(val) = cursor.read_i32::<BigEndian>() {
            if val >= -1000000 && val <= 1000000 {
                return Ok(Some(SandboxValue::Integer(val)));
            }
        }
        cursor.seek(SeekFrom::Start(pos))?;

        // Try reading as float
        if let Ok(val) = cursor.read_f64::<BigEndian>() {
            if val.is_finite() && val >= -1000000.0 && val <= 1000000.0 {
                return Ok(Some(SandboxValue::Float(val)));
            }
        }
        cursor.seek(SeekFrom::Start(pos))?;

        // Default: skip 4 bytes
        cursor.seek(SeekFrom::Current(4))?;
        Ok(None)
    }

    /// Try parsing with alternate format (raw key=value text)
    fn parse_alternate_format(&mut self, data: &[u8]) -> Result<(), SandboxError> {
        // Try to find readable strings in the data
        let text = String::from_utf8_lossy(data);

        // Look for patterns like "KeyName" followed by values
        // This is a fallback for files that might have a different structure
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Try to extract key=value pairs
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value_str = line[pos + 1..].trim();

                if key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    // Try to parse the value
                    if value_str == "true" {
                        self.settings.insert(key, SandboxValue::Boolean(true));
                    } else if value_str == "false" {
                        self.settings.insert(key, SandboxValue::Boolean(false));
                    } else if let Ok(i) = value_str.parse::<i32>() {
                        self.settings.insert(key, SandboxValue::Integer(i));
                    } else if let Ok(f) = value_str.parse::<f64>() {
                        self.settings.insert(key, SandboxValue::Float(f));
                    } else {
                        self.settings.insert(key, SandboxValue::String(value_str.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a setting value by key
    pub fn get(&self, key: &str) -> Option<&SandboxValue> {
        self.settings.get(key)
    }

    /// Set a setting value
    pub fn set(&mut self, key: String, value: SandboxValue) {
        self.settings.insert(key, value);
    }

    /// Serialize back to binary format
    /// Note: This creates a new file based on the settings, which may not be
    /// byte-for-byte identical to the original but should be functionally equivalent
    pub fn serialize(&self) -> Result<Vec<u8>, SandboxError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        // Sort keys for consistent output
        let mut keys: Vec<_> = self.settings.keys().collect();
        keys.sort();

        for key in keys {
            let value = &self.settings[key];

            // Write key length (big-endian i16)
            cursor.write_i16::<BigEndian>(key.len() as i16)?;

            // Write key bytes
            cursor.write_all(key.as_bytes())?;

            // Write value based on type
            match value {
                SandboxValue::Boolean(b) => {
                    cursor.write_u8(if *b { 1 } else { 0 })?;
                }
                SandboxValue::Integer(i) | SandboxValue::Enum(i) => {
                    cursor.write_i32::<BigEndian>(*i)?;
                }
                SandboxValue::Float(f) => {
                    cursor.write_f64::<BigEndian>(*f)?;
                }
                SandboxValue::String(s) => {
                    cursor.write_i16::<BigEndian>(s.len() as i16)?;
                    cursor.write_all(s.as_bytes())?;
                }
            }
        }

        Ok(buffer)
    }

    /// Get metadata for all known settings
    pub fn get_all_metadata() -> Vec<SettingMetadata> {
        vec![
            // Population settings
            SettingMetadata {
                name: "Population".into(),
                display_name: "Zombie Population".into(),
                category: "Population".into(),
                description: "Base zombie population multiplier".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(5.0),
                enum_options: Some(vec![
                    "Insane".into(), "Very High".into(), "High".into(),
                    "Normal".into(), "Low".into(), "Very Low".into(), "None".into()
                ]),
            },
            SettingMetadata {
                name: "PopulationModifier".into(),
                display_name: "Population Modifier".into(),
                category: "Population".into(),
                description: "Multiplier for zombie population".into(),
                value_type: "float".into(),
                min_value: Some(0.0),
                max_value: Some(4.0),
                enum_options: None,
            },
            // Zombie Lore settings
            SettingMetadata {
                name: "Speed".into(),
                display_name: "Zombie Speed".into(),
                category: "Zombie Lore".into(),
                description: "How fast zombies move".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(4.0),
                enum_options: Some(vec![
                    "Sprinters".into(), "Fast Shamblers".into(),
                    "Shamblers".into(), "Random".into()
                ]),
            },
            SettingMetadata {
                name: "Strength".into(),
                display_name: "Zombie Strength".into(),
                category: "Zombie Lore".into(),
                description: "How strong zombies are".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(4.0),
                enum_options: Some(vec![
                    "Superhuman".into(), "Normal".into(),
                    "Weak".into(), "Random".into()
                ]),
            },
            SettingMetadata {
                name: "Toughness".into(),
                display_name: "Zombie Toughness".into(),
                category: "Zombie Lore".into(),
                description: "How much damage zombies can take".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(4.0),
                enum_options: Some(vec![
                    "Tough".into(), "Normal".into(),
                    "Fragile".into(), "Random".into()
                ]),
            },
            SettingMetadata {
                name: "Transmission".into(),
                display_name: "Transmission".into(),
                category: "Zombie Lore".into(),
                description: "How the infection spreads".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(4.0),
                enum_options: Some(vec![
                    "Blood + Saliva".into(), "Saliva Only".into(),
                    "Everyone's Infected".into(), "None".into()
                ]),
            },
            SettingMetadata {
                name: "Mortality".into(),
                display_name: "Infection Mortality".into(),
                category: "Zombie Lore".into(),
                description: "How long until infected die".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(7.0),
                enum_options: Some(vec![
                    "Instant".into(), "0-30 seconds".into(), "0-1 minute".into(),
                    "0-12 hours".into(), "2-3 days".into(), "1-2 weeks".into(), "Never".into()
                ]),
            },
            SettingMetadata {
                name: "Memory".into(),
                display_name: "Zombie Memory".into(),
                category: "Zombie Lore".into(),
                description: "How long zombies remember targets".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(4.0),
                enum_options: Some(vec![
                    "None".into(), "Short".into(),
                    "Normal".into(), "Long".into()
                ]),
            },
            // Time settings
            SettingMetadata {
                name: "DayLength".into(),
                display_name: "Day Length".into(),
                category: "Time".into(),
                description: "Length of a day in real-time minutes".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(25.0),
                enum_options: Some(vec![
                    "15 min".into(), "30 min".into(), "1 hour".into(), "2 hours".into(),
                    "3 hours".into(), "4 hours".into(), "5 hours".into(), "6 hours".into(),
                    "7 hours".into(), "8 hours".into(), "9 hours".into(), "10 hours".into(),
                    "11 hours".into(), "12 hours".into(), "Real-time".into()
                ]),
            },
            SettingMetadata {
                name: "StartMonth".into(),
                display_name: "Start Month".into(),
                category: "Time".into(),
                description: "Month the game starts in".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(12.0),
                enum_options: Some(vec![
                    "January".into(), "February".into(), "March".into(), "April".into(),
                    "May".into(), "June".into(), "July".into(), "August".into(),
                    "September".into(), "October".into(), "November".into(), "December".into()
                ]),
            },
            SettingMetadata {
                name: "StartDay".into(),
                display_name: "Start Day".into(),
                category: "Time".into(),
                description: "Day of the month the game starts on".into(),
                value_type: "integer".into(),
                min_value: Some(1.0),
                max_value: Some(28.0),
                enum_options: None,
            },
            // Loot settings
            SettingMetadata {
                name: "FoodLoot".into(),
                display_name: "Food Loot".into(),
                category: "Loot".into(),
                description: "Rarity of food items".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(6.0),
                enum_options: Some(vec![
                    "None".into(), "Extremely Rare".into(), "Rare".into(),
                    "Normal".into(), "Common".into(), "Abundant".into()
                ]),
            },
            SettingMetadata {
                name: "WeaponLoot".into(),
                display_name: "Weapon Loot".into(),
                category: "Loot".into(),
                description: "Rarity of weapons".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(6.0),
                enum_options: Some(vec![
                    "None".into(), "Extremely Rare".into(), "Rare".into(),
                    "Normal".into(), "Common".into(), "Abundant".into()
                ]),
            },
            SettingMetadata {
                name: "AmmoLoot".into(),
                display_name: "Ammo Loot".into(),
                category: "Loot".into(),
                description: "Rarity of ammunition".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(6.0),
                enum_options: Some(vec![
                    "None".into(), "Extremely Rare".into(), "Rare".into(),
                    "Normal".into(), "Common".into(), "Abundant".into()
                ]),
            },
            SettingMetadata {
                name: "MedicalLoot".into(),
                display_name: "Medical Loot".into(),
                category: "Loot".into(),
                description: "Rarity of medical supplies".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(6.0),
                enum_options: Some(vec![
                    "None".into(), "Extremely Rare".into(), "Rare".into(),
                    "Normal".into(), "Common".into(), "Abundant".into()
                ]),
            },
            // Utility settings
            SettingMetadata {
                name: "WaterShut".into(),
                display_name: "Water Shutoff".into(),
                category: "World".into(),
                description: "When water shuts off (0-2 = Instant, 3-6 = 0-30 days, etc)".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(8.0),
                enum_options: Some(vec![
                    "Instant".into(), "0-30 Days".into(), "0-2 Months".into(),
                    "0-6 Months".into(), "0-1 Year".into(), "0-5 Years".into(),
                    "2-6 Months".into(), "6-12 Months".into()
                ]),
            },
            SettingMetadata {
                name: "ElecShut".into(),
                display_name: "Electricity Shutoff".into(),
                category: "World".into(),
                description: "When electricity shuts off".into(),
                value_type: "enum".into(),
                min_value: Some(1.0),
                max_value: Some(8.0),
                enum_options: Some(vec![
                    "Instant".into(), "0-30 Days".into(), "0-2 Months".into(),
                    "0-6 Months".into(), "0-1 Year".into(), "0-5 Years".into(),
                    "2-6 Months".into(), "6-12 Months".into()
                ]),
            },
            // XP settings
            SettingMetadata {
                name: "XpMultiplier".into(),
                display_name: "XP Multiplier".into(),
                category: "Character".into(),
                description: "Experience gain multiplier".into(),
                value_type: "float".into(),
                min_value: Some(0.001),
                max_value: Some(1000.0),
                enum_options: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_value_conversions() {
        let bool_val = SandboxValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));

        let int_val = SandboxValue::Integer(42);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let float_val = SandboxValue::Float(3.14);
        assert_eq!(float_val.as_float(), Some(3.14));
    }
}

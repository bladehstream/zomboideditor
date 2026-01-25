use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(i32),
}

/// All skill/perk types in Project Zomboid
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Perk {
    // Passive Skills
    Strength,
    Fitness,
    // Combat Skills
    Blunt,
    SmallBlunt,
    LongBlade,
    SmallBlade,
    Axe,
    Spear,
    Maintenance,
    // Firearm Skills
    Aiming,
    Reloading,
    // Agility Skills
    Sprinting,
    Nimble,
    Sneaking,
    Lightfoot,
    // Crafting Skills
    Carpentry,
    Woodwork,
    Cooking,
    Farming,
    Doctor,
    Electricity,
    Mechanics,
    MetalWelding,
    Tailoring,
    // Survivalist Skills
    Fishing,
    Trapping,
    Foraging,
}

impl Perk {
    pub fn all() -> Vec<Perk> {
        vec![
            Perk::Strength, Perk::Fitness,
            Perk::Blunt, Perk::SmallBlunt, Perk::LongBlade, Perk::SmallBlade,
            Perk::Axe, Perk::Spear, Perk::Maintenance,
            Perk::Aiming, Perk::Reloading,
            Perk::Sprinting, Perk::Nimble, Perk::Sneaking, Perk::Lightfoot,
            Perk::Carpentry, Perk::Woodwork, Perk::Cooking, Perk::Farming,
            Perk::Doctor, Perk::Electricity, Perk::Mechanics, Perk::MetalWelding,
            Perk::Tailoring,
            Perk::Fishing, Perk::Trapping, Perk::Foraging,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Perk::Strength => "Strength",
            Perk::Fitness => "Fitness",
            Perk::Blunt => "Long Blunt",
            Perk::SmallBlunt => "Short Blunt",
            Perk::LongBlade => "Long Blade",
            Perk::SmallBlade => "Short Blade",
            Perk::Axe => "Axe",
            Perk::Spear => "Spear",
            Perk::Maintenance => "Maintenance",
            Perk::Aiming => "Aiming",
            Perk::Reloading => "Reloading",
            Perk::Sprinting => "Sprinting",
            Perk::Nimble => "Nimble",
            Perk::Sneaking => "Sneaking",
            Perk::Lightfoot => "Lightfoot",
            Perk::Carpentry => "Carpentry",
            Perk::Woodwork => "Woodwork",
            Perk::Cooking => "Cooking",
            Perk::Farming => "Farming",
            Perk::Doctor => "First Aid",
            Perk::Electricity => "Electrical",
            Perk::Mechanics => "Mechanics",
            Perk::MetalWelding => "Metalworking",
            Perk::Tailoring => "Tailoring",
            Perk::Fishing => "Fishing",
            Perk::Trapping => "Trapping",
            Perk::Foraging => "Foraging",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Perk::Strength | Perk::Fitness => "Passive",
            Perk::Blunt | Perk::SmallBlunt | Perk::LongBlade | Perk::SmallBlade |
            Perk::Axe | Perk::Spear | Perk::Maintenance => "Combat",
            Perk::Aiming | Perk::Reloading => "Firearm",
            Perk::Sprinting | Perk::Nimble | Perk::Sneaking | Perk::Lightfoot => "Agility",
            Perk::Carpentry | Perk::Woodwork | Perk::Cooking | Perk::Farming |
            Perk::Doctor | Perk::Electricity | Perk::Mechanics | Perk::MetalWelding |
            Perk::Tailoring => "Crafting",
            Perk::Fishing | Perk::Trapping | Perk::Foraging => "Survivalist",
        }
    }

    pub fn max_level(&self) -> i32 {
        match self {
            Perk::Strength | Perk::Fitness => 10,
            _ => 10,
        }
    }
}

/// Skill data for a single perk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub perk: Perk,
    pub level: i32,
    pub xp: f32,
    pub xp_to_next_level: f32,
    pub boost: i32, // From traits/professions
}

impl SkillData {
    pub fn new(perk: Perk) -> Self {
        Self {
            perk,
            level: 0,
            xp: 0.0,
            xp_to_next_level: Self::xp_required_for_level(1),
            boost: 0,
        }
    }

    /// Calculate XP required for a specific level
    pub fn xp_required_for_level(level: i32) -> f32 {
        // PZ uses an exponential XP curve
        match level {
            0 => 0.0,
            1 => 75.0,
            2 => 150.0,
            3 => 300.0,
            4 => 450.0,
            5 => 750.0,
            6 => 1500.0,
            7 => 3000.0,
            8 => 4500.0,
            9 => 6000.0,
            10 => 7500.0,
            _ => 10000.0,
        }
    }
}

/// Body part health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartHealth {
    pub name: String,
    pub health: f32,      // 0.0 to 100.0
    pub is_bleeding: bool,
    pub is_bitten: bool,
    pub is_scratched: bool,
    pub is_infected: bool,
    pub is_bandaged: bool,
    pub is_stitched: bool,
    pub is_deep_wounded: bool,
    pub is_burnt: bool,
    pub is_fractured: bool,
}

impl BodyPartHealth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            health: 100.0,
            is_bleeding: false,
            is_bitten: false,
            is_scratched: false,
            is_infected: false,
            is_bandaged: false,
            is_stitched: false,
            is_deep_wounded: false,
            is_burnt: false,
            is_fractured: false,
        }
    }

    pub fn all_body_parts() -> Vec<&'static str> {
        vec![
            "Head", "Neck", "Torso_Upper", "Torso_Lower",
            "UpperArm_L", "UpperArm_R", "ForeArm_L", "ForeArm_R",
            "Hand_L", "Hand_R",
            "Groin", "UpperLeg_L", "UpperLeg_R",
            "LowerLeg_L", "LowerLeg_R", "Foot_L", "Foot_R"
        ]
    }
}

/// Character stats (hunger, thirst, fatigue, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub hunger: f32,      // 0.0 (full) to 1.0 (starving)
    pub thirst: f32,      // 0.0 (hydrated) to 1.0 (dying of thirst)
    pub fatigue: f32,     // 0.0 (rested) to 1.0 (exhausted)
    pub stress: f32,      // 0.0 (calm) to 1.0 (panicked)
    pub boredom: f32,     // 0.0 (entertained) to 1.0 (bored to tears)
    pub unhappiness: f32, // 0.0 (happy) to 1.0 (depressed)
    pub pain: f32,        // 0.0 (no pain) to 1.0 (agony)
    pub drunk: f32,       // 0.0 (sober) to 1.0 (wasted)
    pub endurance: f32,   // 0.0 (exhausted) to 1.0 (full stamina)
    pub weight: f32,      // Character weight in kg
    pub calories: f32,    // Current calories
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            hunger: 0.0,
            thirst: 0.0,
            fatigue: 0.0,
            stress: 0.0,
            boredom: 0.0,
            unhappiness: 0.0,
            pain: 0.0,
            drunk: 0.0,
            endurance: 1.0,
            weight: 80.0,
            calories: 0.0,
        }
    }
}

/// Position in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: i32, // Floor level (0 = ground)
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0 }
    }
}

/// Complete player data from map_p.bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub version: i32,
    pub player_name: String,
    pub position: Position,
    pub stats: CharacterStats,
    pub health: HashMap<String, BodyPartHealth>,
    pub skills: HashMap<String, SkillData>,
    pub hours_survived: f64,
    pub zombies_killed: i32,
    pub is_infected: bool,
    pub infection_time: f32,
    #[serde(skip)]
    raw_data: Vec<u8>,
    #[serde(skip)]
    modified_regions: Vec<(usize, usize)>, // Track which regions we've modified
}

impl Default for PlayerData {
    fn default() -> Self {
        let mut skills = HashMap::new();
        for perk in Perk::all() {
            let skill = SkillData::new(perk);
            skills.insert(format!("{:?}", perk), skill);
        }

        let mut health = HashMap::new();
        for part in BodyPartHealth::all_body_parts() {
            health.insert(part.to_string(), BodyPartHealth::new(part));
        }

        Self {
            version: 195, // Build 41 version
            player_name: "Survivor".to_string(),
            position: Position::default(),
            stats: CharacterStats::default(),
            health,
            skills,
            hours_survived: 0.0,
            zombies_killed: 0,
            is_infected: false,
            infection_time: 0.0,
            raw_data: Vec::new(),
            modified_regions: Vec::new(),
        }
    }
}

impl PlayerData {
    /// Parse player data from map_p.bin file bytes
    ///
    /// The format is Java ByteBuffer serialization (big-endian)
    /// This is a best-effort parser based on reverse engineering
    pub fn parse(data: &[u8]) -> Result<Self, PlayerError> {
        let mut player = PlayerData::default();
        player.raw_data = data.to_vec();

        if data.len() < 10 {
            return Err(PlayerError::InvalidFormat("File too small".into()));
        }

        let mut cursor = Cursor::new(data);

        // Try to detect Build 41 format
        // The file starts with various header data

        // Read potential version/flags
        let _ = cursor.read_u8(); // Skip first byte (often 0 or flag)

        // Try to read world version
        if let Ok(version) = cursor.read_i32::<BigEndian>() {
            if version > 0 && version < 300 {
                player.version = version;
            }
        }

        // Search for patterns in the data
        Self::extract_known_values(&mut player, data)?;

        Ok(player)
    }

    /// Extract known values by searching for patterns
    fn extract_known_values(player: &mut PlayerData, data: &[u8]) -> Result<(), PlayerError> {
        // Search for floating point values that look like coordinates
        // PZ coordinates are typically in the range of 0-20000
        let mut cursor = Cursor::new(data);

        let mut potential_coords = Vec::new();
        while cursor.position() < data.len() as u64 - 4 {
            if let Ok(val) = cursor.read_f32::<BigEndian>() {
                if val > 1000.0 && val < 20000.0 && val.is_finite() {
                    potential_coords.push((cursor.position() as usize - 4, val));
                }
            }
        }

        // If we found potential X/Y coordinates close together, use them
        for window in potential_coords.windows(2) {
            let (pos1, val1) = window[0];
            let (pos2, val2) = window[1];

            // Coordinates should be consecutive (8 bytes apart for two f32s)
            if pos2 - pos1 == 4 || pos2 - pos1 == 8 {
                player.position.x = val1;
                player.position.y = val2;
                break;
            }
        }

        // Search for hours survived (f64 in the range 0-10000)
        cursor.set_position(0);
        while cursor.position() < data.len() as u64 - 8 {
            if let Ok(val) = cursor.read_f64::<BigEndian>() {
                if val >= 0.0 && val < 100000.0 && val.is_finite() {
                    // This could be hours survived
                    player.hours_survived = val;
                    break;
                }
            }
        }

        // Try to find skill levels (i32 values 0-10)
        // This is speculative without more format documentation

        Ok(())
    }

    /// Get a skill by perk name
    pub fn get_skill(&self, perk: &str) -> Option<&SkillData> {
        self.skills.get(perk)
    }

    /// Set a skill level
    pub fn set_skill_level(&mut self, perk: &str, level: i32) {
        if let Some(skill) = self.skills.get_mut(perk) {
            skill.level = level.clamp(0, skill.perk.max_level());
            skill.xp = SkillData::xp_required_for_level(level);
            skill.xp_to_next_level = SkillData::xp_required_for_level(level + 1);
        }
    }

    /// Set a skill XP
    pub fn set_skill_xp(&mut self, perk: &str, xp: f32) {
        if let Some(skill) = self.skills.get_mut(perk) {
            skill.xp = xp.max(0.0);
            // Calculate level from XP
            let mut level = 0;
            for l in 0..=10 {
                if skill.xp >= SkillData::xp_required_for_level(l) {
                    level = l;
                } else {
                    break;
                }
            }
            skill.level = level;
            skill.xp_to_next_level = SkillData::xp_required_for_level(level + 1);
        }
    }

    /// Heal all body parts
    pub fn heal_all(&mut self) {
        for (_, part) in self.health.iter_mut() {
            part.health = 100.0;
            part.is_bleeding = false;
            part.is_bitten = false;
            part.is_scratched = false;
            part.is_infected = false;
            part.is_deep_wounded = false;
            part.is_burnt = false;
            part.is_fractured = false;
        }
        self.is_infected = false;
        self.infection_time = 0.0;
    }

    /// Reset all stats to optimal values
    pub fn reset_stats(&mut self) {
        self.stats = CharacterStats {
            hunger: 0.0,
            thirst: 0.0,
            fatigue: 0.0,
            stress: 0.0,
            boredom: 0.0,
            unhappiness: 0.0,
            pain: 0.0,
            drunk: 0.0,
            endurance: 1.0,
            weight: self.stats.weight, // Preserve weight
            calories: self.stats.calories,
        };
    }

    /// Serialize back to binary format
    ///
    /// WARNING: Since the exact format isn't fully documented, this creates
    /// a modified version of the original file with patched values where
    /// we've identified them. Full serialization from scratch is not supported.
    pub fn serialize(&self) -> Result<Vec<u8>, PlayerError> {
        // For now, return the raw data with modifications
        // In a real implementation, we'd need to patch specific offsets
        if self.raw_data.is_empty() {
            return Err(PlayerError::InvalidFormat(
                "Cannot serialize without original file data. Load a save first.".into()
            ));
        }

        // Return original data (modifications would need offset tracking)
        Ok(self.raw_data.clone())
    }

    /// Create a simple summary for display
    pub fn summary(&self) -> String {
        format!(
            "Player: {}\nPosition: ({:.0}, {:.0}, {})\nHours Survived: {:.1}\nZombies Killed: {}",
            self.player_name,
            self.position.x,
            self.position.y,
            self.position.z,
            self.hours_survived,
            self.zombies_killed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_xp_calculation() {
        let skill = SkillData::new(Perk::Carpentry);
        assert_eq!(skill.level, 0);
        assert_eq!(skill.xp, 0.0);
    }

    #[test]
    fn test_default_player() {
        let player = PlayerData::default();
        assert_eq!(player.skills.len(), Perk::all().len());
        assert_eq!(player.health.len(), BodyPartHealth::all_body_parts().len());
    }
}

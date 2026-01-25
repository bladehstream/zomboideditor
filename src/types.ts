// API Response types
export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

// Sandbox types
export interface SandboxValue {
  type: 'Boolean' | 'Integer' | 'Float' | 'String' | 'Enum';
  value: boolean | number | string;
}

export interface SandboxSettings {
  version: number;
  settings: Record<string, SandboxValue>;
}

export interface SettingMetadata {
  name: string;
  display_name: string;
  category: string;
  description: string;
  value_type: string;
  min_value: number | null;
  max_value: number | null;
  enum_options: string[] | null;
}

// Player types
export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface CharacterStats {
  hunger: number;
  thirst: number;
  fatigue: number;
  stress: number;
  boredom: number;
  unhappiness: number;
  pain: number;
  drunk: number;
  endurance: number;
  weight: number;
  calories: number;
}

export interface BodyPartHealth {
  name: string;
  health: number;
  is_bleeding: boolean;
  is_bitten: boolean;
  is_scratched: boolean;
  is_infected: boolean;
  is_bandaged: boolean;
  is_stitched: boolean;
  is_deep_wounded: boolean;
  is_burnt: boolean;
  is_fractured: boolean;
}

export interface SkillData {
  perk: string;
  level: number;
  xp: number;
  xp_to_next_level: number;
  boost: number;
}

export interface PlayerData {
  version: number;
  player_name: string;
  position: Position;
  stats: CharacterStats;
  health: Record<string, BodyPartHealth>;
  skills: Record<string, SkillData>;
  hours_survived: number;
  zombies_killed: number;
  is_infected: boolean;
  infection_time: number;
}

export interface PerkInfo {
  name: string;
  display_name: string;
  category: string;
  max_level: number;
}

// Database types
export interface PlayerRecord {
  id: number;
  username: string;
  display_name: string;
  password_hash: string;
  x: number;
  y: number;
  z: number;
  is_admin: boolean;
  access_level: string;
  last_connection: string | null;
  steam_id: string | null;
  hours_played: number;
  is_banned: boolean;
}

export interface WhitelistEntry {
  id: number;
  username: string;
  password_hash: string;
  is_admin: boolean;
  steam_id: string | null;
}

// Save info
export interface SaveInfo {
  sandbox_loaded: boolean;
  sandbox_path: string | null;
  sandbox_settings_count: number;
  player_loaded: boolean;
  player_path: string | null;
  player_name: string | null;
  database_loaded: boolean;
  database_path: string | null;
  player_count: number;
}

// Tab types
export type TabType = 'sandbox' | 'player' | 'database' | 'about';

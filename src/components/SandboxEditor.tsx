import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SandboxSettings, SandboxValue, SettingMetadata, ApiResponse } from '../types';

interface Props {
  isLoaded: boolean;
  onStatusChange: (msg: string, isError?: boolean) => void;
  onRefresh: () => void;
}

// Group settings by category for better organization
const SETTING_GROUPS = {
  'Zombie Lore': ['Speed', 'Strength', 'Toughness', 'Transmission', 'Mortality', 'Memory', 'Cognition', 'Sight', 'Hearing'],
  'Population': ['Population', 'PopulationModifier', 'PopulationPeak', 'PopulationPeakDay', 'PopulationStart'],
  'Time': ['DayLength', 'StartMonth', 'StartDay', 'StartYear', 'StartTime'],
  'Loot': ['FoodLoot', 'WeaponLoot', 'AmmoLoot', 'MedicalLoot', 'MechanicsLoot', 'LiteratureLoot', 'SurvivalLoot', 'OtherLoot'],
  'World': ['WaterShut', 'ElecShut', 'Temperature', 'Rain', 'ErosionSpeed'],
  'Character': ['XpMultiplier', 'StatsDecrease'],
  'Vehicles': ['CarSpawnRate', 'ChanceHasGas', 'InitialGas', 'LockedCar'],
};

export default function SandboxEditor({ isLoaded, onStatusChange, onRefresh }: Props) {
  const [settings, setSettings] = useState<SandboxSettings | null>(null);
  const [metadata, setMetadata] = useState<SettingMetadata[]>([]);
  const [filter, setFilter] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    loadMetadata();
  }, []);

  useEffect(() => {
    if (isLoaded) {
      loadSettings();
    } else {
      setSettings(null);
    }
  }, [isLoaded]);

  const loadMetadata = async () => {
    try {
      const result = await invoke<ApiResponse<SettingMetadata[]>>('get_sandbox_metadata');
      if (result.success && result.data) {
        setMetadata(result.data);
      }
    } catch (e) {
      console.error('Failed to load metadata:', e);
    }
  };

  const loadSettings = async () => {
    try {
      const result = await invoke<ApiResponse<SandboxSettings>>('get_sandbox_settings');
      if (result.success && result.data) {
        setSettings(result.data);
        setHasChanges(false);
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  const updateSetting = async (key: string, value: SandboxValue) => {
    try {
      const result = await invoke<ApiResponse<void>>('update_sandbox_setting', { key, value });
      if (result.success) {
        setSettings(prev => {
          if (!prev) return prev;
          return {
            ...prev,
            settings: { ...prev.settings, [key]: value }
          };
        });
        setHasChanges(true);
      } else {
        onStatusChange(result.error || 'Failed to update setting', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const handleSave = async () => {
    try {
      onStatusChange('Saving sandbox file...');
      const result = await invoke<ApiResponse<string>>('save_sandbox', { path: null });
      if (result.success) {
        onStatusChange('Sandbox file saved successfully');
        setHasChanges(false);
        onRefresh();
      } else {
        onStatusChange(result.error || 'Failed to save', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const getMetadataForKey = (key: string): SettingMetadata | undefined => {
    return metadata.find(m => m.name === key);
  };

  const renderSettingInput = (key: string, value: SandboxValue) => {
    const meta = getMetadataForKey(key);

    if (value.type === 'Boolean') {
      return (
        <label className="toggle">
          <input
            type="checkbox"
            checked={value.value as boolean}
            onChange={(e) => updateSetting(key, { type: 'Boolean', value: e.target.checked })}
          />
          <span className="toggle-slider"></span>
        </label>
      );
    }

    if (meta?.enum_options) {
      return (
        <select
          value={value.value as number}
          onChange={(e) => updateSetting(key, { type: 'Enum', value: parseInt(e.target.value) })}
        >
          {meta.enum_options.map((opt, idx) => (
            <option key={idx} value={idx + 1}>{opt}</option>
          ))}
        </select>
      );
    }

    if (value.type === 'Float') {
      return (
        <input
          type="number"
          step="0.1"
          value={value.value as number}
          min={meta?.min_value ?? undefined}
          max={meta?.max_value ?? undefined}
          onChange={(e) => updateSetting(key, { type: 'Float', value: parseFloat(e.target.value) || 0 })}
        />
      );
    }

    if (value.type === 'Integer' || value.type === 'Enum') {
      return (
        <input
          type="number"
          value={value.value as number}
          min={meta?.min_value ?? undefined}
          max={meta?.max_value ?? undefined}
          onChange={(e) => updateSetting(key, { type: value.type, value: parseInt(e.target.value) || 0 })}
        />
      );
    }

    if (value.type === 'String') {
      return (
        <input
          type="text"
          value={value.value as string}
          onChange={(e) => updateSetting(key, { type: 'String', value: e.target.value })}
        />
      );
    }

    return <span>{JSON.stringify(value.value)}</span>;
  };

  const getFilteredSettings = () => {
    if (!settings) return [];

    let entries = Object.entries(settings.settings);

    // Filter by search term
    if (filter) {
      const lowerFilter = filter.toLowerCase();
      entries = entries.filter(([key]) => key.toLowerCase().includes(lowerFilter));
    }

    // Filter by category
    if (selectedCategory) {
      const categoryKeys = SETTING_GROUPS[selectedCategory as keyof typeof SETTING_GROUPS] || [];
      entries = entries.filter(([key]) => categoryKeys.some(k => key.includes(k)));
    }

    return entries.sort((a, b) => a[0].localeCompare(b[0]));
  };

  if (!isLoaded) {
    return (
      <div className="editor-placeholder">
        <div className="placeholder-icon">📁</div>
        <h3>No Sandbox File Loaded</h3>
        <p>Click "Load Sandbox" to open a map_sand.bin file</p>
        <p className="hint">Typically found in: Zomboid/Saves/Sandbox/[SaveName]/map_sand.bin</p>
      </div>
    );
  }

  const filteredSettings = getFilteredSettings();

  return (
    <div className="sandbox-editor">
      <div className="editor-header">
        <h2>Sandbox Settings</h2>
        <div className="editor-actions">
          {hasChanges && <span className="unsaved-badge">Unsaved Changes</span>}
          <button onClick={handleSave} className="btn btn-success" disabled={!hasChanges}>
            Save Changes
          </button>
        </div>
      </div>

      <div className="editor-filters">
        <input
          type="text"
          placeholder="Search settings..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="search-input"
        />

        <div className="category-filters">
          <button
            className={`category-btn ${!selectedCategory ? 'active' : ''}`}
            onClick={() => setSelectedCategory(null)}
          >
            All
          </button>
          {Object.keys(SETTING_GROUPS).map(cat => (
            <button
              key={cat}
              className={`category-btn ${selectedCategory === cat ? 'active' : ''}`}
              onClick={() => setSelectedCategory(cat)}
            >
              {cat}
            </button>
          ))}
        </div>
      </div>

      <div className="settings-list">
        {filteredSettings.length === 0 ? (
          <div className="no-results">No settings match your filter</div>
        ) : (
          filteredSettings.map(([key, value]) => {
            const meta = getMetadataForKey(key);
            return (
              <div key={key} className="setting-row">
                <div className="setting-info">
                  <span className="setting-name">{meta?.display_name || key}</span>
                  {meta?.description && (
                    <span className="setting-description">{meta.description}</span>
                  )}
                </div>
                <div className="setting-value">
                  {renderSettingInput(key, value)}
                </div>
              </div>
            );
          })
        )}
      </div>

      <div className="settings-count">
        Showing {filteredSettings.length} of {settings ? Object.keys(settings.settings).length : 0} settings
      </div>
    </div>
  );
}

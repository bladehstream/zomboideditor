import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PlayerData, PerkInfo, ApiResponse } from '../types';

interface Props {
  isLoaded: boolean;
  onStatusChange: (msg: string, isError?: boolean) => void;
  onRefresh: () => void;
}

export default function PlayerEditor({ isLoaded, onStatusChange, onRefresh }: Props) {
  const [playerData, setPlayerData] = useState<PlayerData | null>(null);
  const [perks, setPerks] = useState<PerkInfo[]>([]);
  const [activeSection, setActiveSection] = useState<'stats' | 'skills' | 'health' | 'position'>('stats');
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    loadPerks();
  }, []);

  useEffect(() => {
    if (isLoaded) {
      loadPlayerData();
    } else {
      setPlayerData(null);
    }
  }, [isLoaded]);

  const loadPerks = async () => {
    try {
      const result = await invoke<ApiResponse<PerkInfo[]>>('get_all_perks');
      if (result.success && result.data) {
        setPerks(result.data);
      }
    } catch (e) {
      console.error('Failed to load perks:', e);
    }
  };

  const loadPlayerData = async () => {
    try {
      const result = await invoke<ApiResponse<PlayerData>>('get_player_data');
      if (result.success && result.data) {
        setPlayerData(result.data);
        setHasChanges(false);
      }
    } catch (e) {
      console.error('Failed to load player data:', e);
    }
  };

  const updateSkill = async (perk: string, level: number) => {
    try {
      const result = await invoke<ApiResponse<void>>('update_player_skill', { perk, level });
      if (result.success) {
        setPlayerData(prev => {
          if (!prev) return prev;
          return {
            ...prev,
            skills: {
              ...prev.skills,
              [perk]: { ...prev.skills[perk], level }
            }
          };
        });
        setHasChanges(true);
      } else {
        onStatusChange(result.error || 'Failed to update skill', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const updatePosition = async (x: number, y: number, z: number) => {
    try {
      const result = await invoke<ApiResponse<void>>('update_player_position', { x, y, z });
      if (result.success) {
        setPlayerData(prev => {
          if (!prev) return prev;
          return { ...prev, position: { x, y, z } };
        });
        setHasChanges(true);
      } else {
        onStatusChange(result.error || 'Failed to update position', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const healPlayer = async () => {
    try {
      const result = await invoke<ApiResponse<void>>('heal_player');
      if (result.success) {
        onStatusChange('Player healed successfully');
        loadPlayerData();
      } else {
        onStatusChange(result.error || 'Failed to heal player', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const handleSave = async () => {
    try {
      onStatusChange('Saving player file...');
      const result = await invoke<ApiResponse<string>>('save_player', { path: null });
      if (result.success) {
        onStatusChange('Player file saved successfully');
        setHasChanges(false);
        onRefresh();
      } else {
        onStatusChange(result.error || 'Failed to save', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const groupPerksByCategory = () => {
    const groups: Record<string, PerkInfo[]> = {};
    perks.forEach(perk => {
      if (!groups[perk.category]) {
        groups[perk.category] = [];
      }
      groups[perk.category].push(perk);
    });
    return groups;
  };

  if (!isLoaded) {
    return (
      <div className="editor-placeholder">
        <div className="placeholder-icon">👤</div>
        <h3>No Player File Loaded</h3>
        <p>Click "Load Player" to open a map_p.bin or map_s.bin file</p>
        <p className="hint">Single player: map_p.bin | Multiplayer: map_s.bin</p>
        <p className="warning">Note: Player binary editing is experimental</p>
      </div>
    );
  }

  const perkGroups = groupPerksByCategory();

  return (
    <div className="player-editor">
      <div className="editor-header">
        <h2>Player Data</h2>
        <div className="editor-actions">
          <button onClick={healPlayer} className="btn btn-warning">
            Heal All
          </button>
          {hasChanges && <span className="unsaved-badge">Unsaved Changes</span>}
          <button onClick={handleSave} className="btn btn-success" disabled={!hasChanges}>
            Save Changes
          </button>
        </div>
      </div>

      {playerData && (
        <>
          <div className="player-summary">
            <div className="summary-item">
              <span className="label">Name:</span>
              <span className="value">{playerData.player_name}</span>
            </div>
            <div className="summary-item">
              <span className="label">Hours Survived:</span>
              <span className="value">{playerData.hours_survived.toFixed(1)}</span>
            </div>
            <div className="summary-item">
              <span className="label">Zombies Killed:</span>
              <span className="value">{playerData.zombies_killed}</span>
            </div>
            <div className="summary-item">
              <span className="label">Infected:</span>
              <span className={`value ${playerData.is_infected ? 'danger' : 'safe'}`}>
                {playerData.is_infected ? 'Yes' : 'No'}
              </span>
            </div>
          </div>

          <nav className="section-tabs">
            <button
              className={`section-tab ${activeSection === 'stats' ? 'active' : ''}`}
              onClick={() => setActiveSection('stats')}
            >
              Stats
            </button>
            <button
              className={`section-tab ${activeSection === 'skills' ? 'active' : ''}`}
              onClick={() => setActiveSection('skills')}
            >
              Skills
            </button>
            <button
              className={`section-tab ${activeSection === 'health' ? 'active' : ''}`}
              onClick={() => setActiveSection('health')}
            >
              Health
            </button>
            <button
              className={`section-tab ${activeSection === 'position' ? 'active' : ''}`}
              onClick={() => setActiveSection('position')}
            >
              Position
            </button>
          </nav>

          <div className="section-content">
            {activeSection === 'stats' && (
              <div className="stats-grid">
                {Object.entries(playerData.stats).map(([stat, value]) => (
                  <div key={stat} className="stat-item">
                    <span className="stat-name">{formatStatName(stat)}</span>
                    <div className="stat-bar">
                      <div
                        className={`stat-fill ${getStatClass(stat, value)}`}
                        style={{ width: `${Math.min(100, Math.abs(value) * 100)}%` }}
                      ></div>
                    </div>
                    <span className="stat-value">{(value * 100).toFixed(0)}%</span>
                  </div>
                ))}
              </div>
            )}

            {activeSection === 'skills' && (
              <div className="skills-section">
                {Object.entries(perkGroups).map(([category, categoryPerks]) => (
                  <div key={category} className="skill-category">
                    <h4>{category}</h4>
                    <div className="skills-grid">
                      {categoryPerks.map(perk => {
                        const skill = playerData.skills[perk.name];
                        const level = skill?.level || 0;
                        return (
                          <div key={perk.name} className="skill-item">
                            <span className="skill-name">{perk.display_name}</span>
                            <div className="skill-level">
                              <input
                                type="range"
                                min="0"
                                max={perk.max_level}
                                value={level}
                                onChange={(e) => updateSkill(perk.name, parseInt(e.target.value))}
                              />
                              <span className="level-value">{level}</span>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {activeSection === 'health' && (
              <div className="health-section">
                <div className="health-grid">
                  {Object.entries(playerData.health).map(([part, data]) => (
                    <div key={part} className={`health-item ${data.health < 100 ? 'injured' : ''}`}>
                      <span className="part-name">{formatBodyPart(data.name)}</span>
                      <div className="health-bar">
                        <div
                          className="health-fill"
                          style={{ width: `${data.health}%` }}
                        ></div>
                      </div>
                      <span className="health-value">{data.health.toFixed(0)}%</span>
                      <div className="conditions">
                        {data.is_bleeding && <span className="condition bleeding">Bleeding</span>}
                        {data.is_bitten && <span className="condition bitten">Bitten</span>}
                        {data.is_scratched && <span className="condition scratched">Scratched</span>}
                        {data.is_infected && <span className="condition infected">Infected</span>}
                        {data.is_fractured && <span className="condition fractured">Fractured</span>}
                        {data.is_burnt && <span className="condition burnt">Burnt</span>}
                        {data.is_bandaged && <span className="condition bandaged">Bandaged</span>}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {activeSection === 'position' && (
              <div className="position-section">
                <div className="position-inputs">
                  <div className="position-field">
                    <label>X Coordinate</label>
                    <input
                      type="number"
                      value={playerData.position.x.toFixed(0)}
                      onChange={(e) => updatePosition(
                        parseFloat(e.target.value) || 0,
                        playerData.position.y,
                        playerData.position.z
                      )}
                    />
                  </div>
                  <div className="position-field">
                    <label>Y Coordinate</label>
                    <input
                      type="number"
                      value={playerData.position.y.toFixed(0)}
                      onChange={(e) => updatePosition(
                        playerData.position.x,
                        parseFloat(e.target.value) || 0,
                        playerData.position.z
                      )}
                    />
                  </div>
                  <div className="position-field">
                    <label>Z (Floor Level)</label>
                    <input
                      type="number"
                      min="0"
                      max="8"
                      value={playerData.position.z}
                      onChange={(e) => updatePosition(
                        playerData.position.x,
                        playerData.position.y,
                        parseInt(e.target.value) || 0
                      )}
                    />
                  </div>
                </div>
                <p className="position-hint">
                  Coordinates are in game tiles. West Point spawn is approximately (10500, 9500).
                </p>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}

function formatStatName(stat: string): string {
  return stat
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/^./, s => s.toUpperCase());
}

function formatBodyPart(name: string): string {
  return name
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .replace(/_L$/, ' (Left)')
    .replace(/_R$/, ' (Right)');
}

function getStatClass(stat: string, value: number): string {
  // For stats where lower is better (hunger, thirst, etc.)
  const lowerIsBetter = ['hunger', 'thirst', 'fatigue', 'stress', 'boredom', 'unhappiness', 'pain', 'drunk'];

  if (lowerIsBetter.includes(stat.toLowerCase())) {
    if (value < 0.3) return 'good';
    if (value < 0.6) return 'warning';
    return 'danger';
  }

  // For stats where higher is better (endurance)
  if (value > 0.7) return 'good';
  if (value > 0.3) return 'warning';
  return 'danger';
}

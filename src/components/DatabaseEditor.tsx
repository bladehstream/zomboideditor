import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PlayerRecord, WhitelistEntry, ApiResponse } from '../types';

interface Props {
  isLoaded: boolean;
  onStatusChange: (msg: string, isError?: boolean) => void;
  onRefresh: () => void;
}

export default function DatabaseEditor({ isLoaded, onStatusChange, onRefresh }: Props) {
  const [players, setPlayers] = useState<PlayerRecord[]>([]);
  const [whitelist, setWhitelist] = useState<WhitelistEntry[]>([]);
  const [selectedPlayer, setSelectedPlayer] = useState<PlayerRecord | null>(null);
  const [activeView, setActiveView] = useState<'players' | 'whitelist' | 'schema'>('players');
  const [schema, setSchema] = useState<[string, string][]>([]);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    if (isLoaded) {
      loadPlayers();
      loadWhitelist();
      loadSchema();
    } else {
      setPlayers([]);
      setWhitelist([]);
      setSelectedPlayer(null);
    }
  }, [isLoaded]);

  const loadPlayers = async () => {
    try {
      const result = await invoke<ApiResponse<PlayerRecord[]>>('get_database_players');
      if (result.success && result.data) {
        setPlayers(result.data);
      }
    } catch (e) {
      console.error('Failed to load players:', e);
    }
  };

  const loadWhitelist = async () => {
    try {
      const result = await invoke<ApiResponse<WhitelistEntry[]>>('get_whitelist');
      if (result.success && result.data) {
        setWhitelist(result.data);
      }
    } catch (e) {
      console.error('Failed to load whitelist:', e);
    }
  };

  const loadSchema = async () => {
    try {
      const result = await invoke<ApiResponse<[string, string][]>>('get_database_schema');
      if (result.success && result.data) {
        setSchema(result.data);
      }
    } catch (e) {
      console.error('Failed to load schema:', e);
    }
  };

  const updateDisplayName = async (username: string, displayName: string) => {
    try {
      const result = await invoke<ApiResponse<void>>('update_player_display_name', {
        username,
        displayName
      });
      if (result.success) {
        onStatusChange('Display name updated');
        loadPlayers();
      } else {
        onStatusChange(result.error || 'Failed to update', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const updatePosition = async (username: string, x: number, y: number, z: number) => {
    try {
      const result = await invoke<ApiResponse<void>>('update_database_player_position', {
        username, x, y, z
      });
      if (result.success) {
        onStatusChange('Position updated');
        loadPlayers();
      } else {
        onStatusChange(result.error || 'Failed to update', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const toggleAdmin = async (username: string, isAdmin: boolean) => {
    try {
      const result = await invoke<ApiResponse<void>>('set_player_admin', {
        username,
        isAdmin
      });
      if (result.success) {
        onStatusChange(`Admin status ${isAdmin ? 'granted' : 'revoked'}`);
        loadPlayers();
      } else {
        onStatusChange(result.error || 'Failed to update', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const toggleBan = async (username: string, isBanned: boolean) => {
    try {
      const result = await invoke<ApiResponse<void>>('set_player_banned', {
        username,
        isBanned
      });
      if (result.success) {
        onStatusChange(`Player ${isBanned ? 'banned' : 'unbanned'}`);
        loadPlayers();
      } else {
        onStatusChange(result.error || 'Failed to update', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const deletePlayer = async (username: string) => {
    if (!confirm(`Are you sure you want to delete player "${username}"? This cannot be undone.`)) {
      return;
    }

    try {
      const result = await invoke<ApiResponse<void>>('delete_database_player', { username });
      if (result.success) {
        onStatusChange('Player deleted');
        setSelectedPlayer(null);
        loadPlayers();
        onRefresh();
      } else {
        onStatusChange(result.error || 'Failed to delete', true);
      }
    } catch (e) {
      onStatusChange(`Error: ${e}`, true);
    }
  };

  const filteredPlayers = players.filter(p =>
    p.username.toLowerCase().includes(filter.toLowerCase()) ||
    p.display_name.toLowerCase().includes(filter.toLowerCase())
  );

  if (!isLoaded) {
    return (
      <div className="editor-placeholder">
        <div className="placeholder-icon">🗄️</div>
        <h3>No Database Loaded</h3>
        <p>Click "Load Database" to open a players.db file</p>
        <p className="hint">Typically found in: Zomboid/db/players.db</p>
        <p className="info">Used for multiplayer server administration</p>
      </div>
    );
  }

  return (
    <div className="database-editor">
      <div className="editor-header">
        <h2>Multiplayer Database</h2>
        <div className="editor-actions">
          <button onClick={loadPlayers} className="btn btn-secondary">
            Refresh
          </button>
        </div>
      </div>

      <nav className="section-tabs">
        <button
          className={`section-tab ${activeView === 'players' ? 'active' : ''}`}
          onClick={() => setActiveView('players')}
        >
          Players ({players.length})
        </button>
        <button
          className={`section-tab ${activeView === 'whitelist' ? 'active' : ''}`}
          onClick={() => setActiveView('whitelist')}
        >
          Whitelist ({whitelist.length})
        </button>
        <button
          className={`section-tab ${activeView === 'schema' ? 'active' : ''}`}
          onClick={() => setActiveView('schema')}
        >
          Schema
        </button>
      </nav>

      <div className="section-content">
        {activeView === 'players' && (
          <div className="players-view">
            <input
              type="text"
              placeholder="Search players..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className="search-input"
            />

            <div className="players-layout">
              <div className="players-list">
                {filteredPlayers.length === 0 ? (
                  <div className="no-results">No players found</div>
                ) : (
                  filteredPlayers.map(player => (
                    <div
                      key={player.id}
                      className={`player-item ${selectedPlayer?.id === player.id ? 'selected' : ''} ${player.is_banned ? 'banned' : ''}`}
                      onClick={() => setSelectedPlayer(player)}
                    >
                      <div className="player-name">
                        {player.display_name || player.username}
                        {player.is_admin && <span className="badge admin">Admin</span>}
                        {player.is_banned && <span className="badge banned">Banned</span>}
                      </div>
                      <div className="player-username">@{player.username}</div>
                    </div>
                  ))
                )}
              </div>

              {selectedPlayer && (
                <div className="player-details">
                  <h3>{selectedPlayer.display_name || selectedPlayer.username}</h3>

                  <div className="detail-section">
                    <h4>Account</h4>
                    <div className="detail-row">
                      <span className="label">Username:</span>
                      <span className="value">{selectedPlayer.username}</span>
                    </div>
                    <div className="detail-row">
                      <span className="label">Display Name:</span>
                      <input
                        type="text"
                        defaultValue={selectedPlayer.display_name}
                        onBlur={(e) => {
                          if (e.target.value !== selectedPlayer.display_name) {
                            updateDisplayName(selectedPlayer.username, e.target.value);
                          }
                        }}
                      />
                    </div>
                    <div className="detail-row">
                      <span className="label">Steam ID:</span>
                      <span className="value">{selectedPlayer.steam_id || 'N/A'}</span>
                    </div>
                    <div className="detail-row">
                      <span className="label">Hours Played:</span>
                      <span className="value">{selectedPlayer.hours_played.toFixed(1)}</span>
                    </div>
                    <div className="detail-row">
                      <span className="label">Last Connection:</span>
                      <span className="value">{selectedPlayer.last_connection || 'N/A'}</span>
                    </div>
                  </div>

                  <div className="detail-section">
                    <h4>Position</h4>
                    <div className="position-row">
                      <div className="position-field">
                        <label>X</label>
                        <input
                          type="number"
                          defaultValue={selectedPlayer.x.toFixed(0)}
                          onBlur={(e) => updatePosition(
                            selectedPlayer.username,
                            parseFloat(e.target.value) || 0,
                            selectedPlayer.y,
                            selectedPlayer.z
                          )}
                        />
                      </div>
                      <div className="position-field">
                        <label>Y</label>
                        <input
                          type="number"
                          defaultValue={selectedPlayer.y.toFixed(0)}
                          onBlur={(e) => updatePosition(
                            selectedPlayer.username,
                            selectedPlayer.x,
                            parseFloat(e.target.value) || 0,
                            selectedPlayer.z
                          )}
                        />
                      </div>
                      <div className="position-field">
                        <label>Z</label>
                        <input
                          type="number"
                          defaultValue={selectedPlayer.z}
                          onBlur={(e) => updatePosition(
                            selectedPlayer.username,
                            selectedPlayer.x,
                            selectedPlayer.y,
                            parseInt(e.target.value) || 0
                          )}
                        />
                      </div>
                    </div>
                  </div>

                  <div className="detail-section">
                    <h4>Actions</h4>
                    <div className="action-buttons">
                      <button
                        className={`btn ${selectedPlayer.is_admin ? 'btn-warning' : 'btn-success'}`}
                        onClick={() => toggleAdmin(selectedPlayer.username, !selectedPlayer.is_admin)}
                      >
                        {selectedPlayer.is_admin ? 'Remove Admin' : 'Make Admin'}
                      </button>
                      <button
                        className={`btn ${selectedPlayer.is_banned ? 'btn-success' : 'btn-warning'}`}
                        onClick={() => toggleBan(selectedPlayer.username, !selectedPlayer.is_banned)}
                      >
                        {selectedPlayer.is_banned ? 'Unban' : 'Ban'}
                      </button>
                      <button
                        className="btn btn-danger"
                        onClick={() => deletePlayer(selectedPlayer.username)}
                      >
                        Delete Player
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeView === 'whitelist' && (
          <div className="whitelist-view">
            {whitelist.length === 0 ? (
              <div className="no-results">No whitelist entries found</div>
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Username</th>
                    <th>Admin</th>
                    <th>Steam ID</th>
                  </tr>
                </thead>
                <tbody>
                  {whitelist.map(entry => (
                    <tr key={entry.id}>
                      <td>{entry.id}</td>
                      <td>{entry.username}</td>
                      <td>{entry.is_admin ? 'Yes' : 'No'}</td>
                      <td>{entry.steam_id || 'N/A'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {activeView === 'schema' && (
          <div className="schema-view">
            <p className="schema-info">Database schema information for advanced users</p>
            {schema.map(([name, sql]) => (
              <div key={name} className="schema-table">
                <h4>{name}</h4>
                <pre>{sql}</pre>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import type { TabType, SaveInfo, ApiResponse } from './types';
import SandboxEditor from './components/SandboxEditor';
import PlayerEditor from './components/PlayerEditor';
import DatabaseEditor from './components/DatabaseEditor';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('sandbox');
  const [saveInfo, setSaveInfo] = useState<SaveInfo | null>(null);
  const [status, setStatus] = useState<string>('Ready');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    refreshSaveInfo();
  }, []);

  const refreshSaveInfo = async () => {
    try {
      const info = await invoke<SaveInfo>('get_save_info');
      setSaveInfo(info);
    } catch (e) {
      console.error('Failed to get save info:', e);
    }
  };

  const showStatus = (msg: string, isError = false) => {
    setStatus(msg);
    if (isError) {
      setError(msg);
      setTimeout(() => setError(null), 5000);
    }
  };

  const handleLoadSandbox = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: 'Sandbox Settings', extensions: ['bin'] }],
        title: 'Select map_sand.bin'
      });

      if (selected) {
        showStatus('Loading sandbox file...');
        const result = await invoke<ApiResponse<unknown>>('load_sandbox', { path: selected });

        if (result.success) {
          showStatus('Sandbox file loaded successfully');
          refreshSaveInfo();
        } else {
          showStatus(result.error || 'Failed to load sandbox file', true);
        }
      }
    } catch (e) {
      showStatus(`Error: ${e}`, true);
    }
  };

  const handleLoadPlayer = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: 'Player Data', extensions: ['bin'] }],
        title: 'Select map_p.bin or map_s.bin'
      });

      if (selected) {
        showStatus('Loading player file...');
        const result = await invoke<ApiResponse<unknown>>('load_player', { path: selected });

        if (result.success) {
          showStatus('Player file loaded successfully');
          refreshSaveInfo();
        } else {
          showStatus(result.error || 'Failed to load player file', true);
        }
      }
    } catch (e) {
      showStatus(`Error: ${e}`, true);
    }
  };

  const handleLoadDatabase = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: 'Player Database', extensions: ['db'] }],
        title: 'Select players.db'
      });

      if (selected) {
        showStatus('Loading database...');
        const result = await invoke<ApiResponse<unknown>>('load_database', { path: selected });

        if (result.success) {
          showStatus('Database loaded successfully');
          refreshSaveInfo();
        } else {
          showStatus(result.error || 'Failed to load database', true);
        }
      }
    } catch (e) {
      showStatus(`Error: ${e}`, true);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>Project Zomboid Save Editor</h1>
        <p className="subtitle">Build 41 Compatible</p>
      </header>

      <div className="toolbar">
        <button onClick={handleLoadSandbox} className="btn btn-primary">
          Load Sandbox (map_sand.bin)
        </button>
        <button onClick={handleLoadPlayer} className="btn btn-primary">
          Load Player (map_p.bin)
        </button>
        <button onClick={handleLoadDatabase} className="btn btn-primary">
          Load Database (players.db)
        </button>
      </div>

      {saveInfo && (
        <div className="save-info">
          <div className={`info-item ${saveInfo.sandbox_loaded ? 'loaded' : ''}`}>
            <span className="label">Sandbox:</span>
            <span className="value">
              {saveInfo.sandbox_loaded
                ? `${saveInfo.sandbox_settings_count} settings`
                : 'Not loaded'}
            </span>
          </div>
          <div className={`info-item ${saveInfo.player_loaded ? 'loaded' : ''}`}>
            <span className="label">Player:</span>
            <span className="value">
              {saveInfo.player_loaded
                ? saveInfo.player_name || 'Loaded'
                : 'Not loaded'}
            </span>
          </div>
          <div className={`info-item ${saveInfo.database_loaded ? 'loaded' : ''}`}>
            <span className="label">Database:</span>
            <span className="value">
              {saveInfo.database_loaded
                ? `${saveInfo.player_count} players`
                : 'Not loaded'}
            </span>
          </div>
        </div>
      )}

      <nav className="tabs">
        <button
          className={`tab ${activeTab === 'sandbox' ? 'active' : ''}`}
          onClick={() => setActiveTab('sandbox')}
        >
          Sandbox Settings
        </button>
        <button
          className={`tab ${activeTab === 'player' ? 'active' : ''}`}
          onClick={() => setActiveTab('player')}
        >
          Player Data
        </button>
        <button
          className={`tab ${activeTab === 'database' ? 'active' : ''}`}
          onClick={() => setActiveTab('database')}
        >
          Database (MP)
        </button>
        <button
          className={`tab ${activeTab === 'about' ? 'active' : ''}`}
          onClick={() => setActiveTab('about')}
        >
          About
        </button>
      </nav>

      <main className="content">
        {activeTab === 'sandbox' && (
          <SandboxEditor
            isLoaded={saveInfo?.sandbox_loaded || false}
            onStatusChange={showStatus}
            onRefresh={refreshSaveInfo}
          />
        )}
        {activeTab === 'player' && (
          <PlayerEditor
            isLoaded={saveInfo?.player_loaded || false}
            onStatusChange={showStatus}
            onRefresh={refreshSaveInfo}
          />
        )}
        {activeTab === 'database' && (
          <DatabaseEditor
            isLoaded={saveInfo?.database_loaded || false}
            onStatusChange={showStatus}
            onRefresh={refreshSaveInfo}
          />
        )}
        {activeTab === 'about' && (
          <div className="about-section">
            <h2>Project Zomboid Save Editor</h2>
            <p>A graphical editor for Project Zomboid Build 41 save files.</p>

            <h3>Features</h3>
            <ul>
              <li>Edit sandbox settings (map_sand.bin)</li>
              <li>View and modify player data (map_p.bin)</li>
              <li>Manage multiplayer database (players.db)</li>
            </ul>

            <h3>File Locations</h3>
            <div className="file-locations">
              <p><strong>Single Player:</strong></p>
              <code>C:\Users\[Username]\Zomboid\Saves\Sandbox\[SaveName]\</code>

              <p><strong>Multiplayer:</strong></p>
              <code>C:\Users\[Username]\Zomboid\Saves\Multiplayer\[ServerName]\</code>

              <p><strong>Database:</strong></p>
              <code>C:\Users\[Username]\Zomboid\db\players.db</code>
            </div>

            <h3>Important Notes</h3>
            <ul className="notes">
              <li>Always backup your save files before editing!</li>
              <li>Close Project Zomboid before editing saves</li>
              <li>Some binary formats may vary between game versions</li>
              <li>Player binary editing is experimental</li>
            </ul>

            <p className="version">Version 0.1.0</p>
          </div>
        )}
      </main>

      <footer className="status-bar">
        <span className={error ? 'error' : ''}>{status}</span>
      </footer>
    </div>
  );
}

export default App;

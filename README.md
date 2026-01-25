# Project Zomboid Save Editor

A graphical editor for Project Zomboid Build 41 save files. Built with Tauri (Rust + React).

## Features

- **Sandbox Settings Editor** - Edit game world settings from `map_sand.bin`
  - Zombie lore (speed, strength, toughness, transmission, etc.)
  - Population settings
  - Loot rarity
  - Day/night cycle
  - Utility shutoff timing
  - XP multipliers and more

- **Player Data Viewer** - View and modify player data from `map_p.bin`
  - Skills and XP levels
  - Character stats (hunger, thirst, fatigue, etc.)
  - Health status per body part
  - Position coordinates
  - Heal all injuries

- **Multiplayer Database Manager** - Manage `players.db` SQLite database
  - View all players
  - Edit display names and positions
  - Grant/revoke admin status
  - Ban/unban players
  - Delete player records
  - View whitelist

## File Locations

### Single Player
```
C:\Users\[Username]\Zomboid\Saves\Sandbox\[SaveName]\
├── map_sand.bin    # Sandbox settings
├── map_p.bin       # Player data
├── map_ver.bin     # World version
└── map_*.bin       # World chunks
```

### Multiplayer
```
C:\Users\[Username]\Zomboid\Saves\Multiplayer\[ServerName]\
├── map_sand.bin    # Server sandbox settings
└── map_s.bin       # Player data (server-side)

C:\Users\[Username]\Zomboid\db\
└── players.db      # Player database
```

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)
- Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Windows: [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build for Production

```bash
# Build the application
npm run tauri build
```

The built executable will be in `src-tauri/target/release/`.

## Technical Notes

### Binary Format

Project Zomboid uses Java's ByteBuffer serialization with big-endian byte order. The save file formats are:

- **map_sand.bin**: Contains sandbox settings as a series of string keys followed by typed values (boolean, integer, float, string)
- **map_p.bin / map_s.bin**: Serialized IsoPlayer/IsoGameCharacter objects containing player state
- **players.db**: SQLite3 database with player account information

### Limitations

- Player binary editing is experimental - the exact format varies between game versions
- Always backup your save files before editing
- Close Project Zomboid before editing saves
- Some settings may not be recognized if the game version differs

## License

MIT

## Acknowledgments

- [Project Zomboid](https://projectzomboid.com/) by The Indie Stone
- [PZwiki](https://pzwiki.net/) for file format documentation
- Community reverse engineering efforts

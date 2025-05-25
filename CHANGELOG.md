# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-05-25

### Added
- **Real-time Cursor AI usage monitoring** - Track your GPT-4 usage with live updates
- **Cross-platform system tray support** - Native integration with macOS, Windows, and Linux system trays
- **Bilingual interface** - Full support for English and Chinese languages
- **Configurable refresh intervals** - Choose from 1 minute to 1 hour update frequencies
- **Color-coded status indicators** - Visual feedback based on usage levels:
  - 🟢 Green: < 50% usage
  - 🟡 Yellow: 50-70% usage  
  - 🟠 Orange: 70-90% usage
  - 🔴 Red: > 90% usage
- **Interactive menu system** - Quick access to:
  - Current usage statistics
  - Account information
  - Language settings
  - Refresh interval configuration
  - Direct link to Cursor settings
- **Persistent settings** - Automatically saves user preferences locally
- **Robust error handling** - Graceful degradation on network issues
- **Performance optimizations** - Built with Rust for minimal resource usage (~5MB RAM)

### Technical Features
- **Secure token extraction** - Safely reads Cursor authentication tokens from local storage
- **HTTP retry mechanism** - Automatic retry with exponential backoff for network requests
- **Efficient memory management** - Uses parking_lot for high-performance synchronization
- **Non-blocking UI updates** - Asynchronous data fetching prevents UI freezing
- **Graceful shutdown** - Proper cleanup of background threads and resources
- **Cross-platform compatibility** - Supports macOS, Windows, and Linux

### Security
- **Local-only data storage** - No cloud dependencies or external data transmission
- **Secure authentication** - Uses existing Cursor session tokens
- **Privacy-focused** - Only accesses necessary Cursor API endpoints

### Performance
- **Minimal resource usage** - Optimized for low CPU and memory consumption
- **Smart caching** - Reduces unnecessary API calls
- **Efficient networking** - Connection reuse and timeout management
- **Background processing** - Non-blocking data updates

### Developer Experience
- **Comprehensive error handling** - Detailed error messages and recovery mechanisms
- **Modular architecture** - Clean separation of concerns across modules
- **Type safety** - Leverages Rust's type system for reliability
- **Extensive documentation** - Complete API documentation and usage examples

### Known Limitations
- Windows and Linux platforms require testing (marked as ⚠️)
- Requires Cursor to be installed and logged in
- Network connectivity required for usage data updates

### Dependencies
- `ureq` - HTTP client with JSON support
- `tray-icon` - Cross-platform system tray integration
- `tao` - Cross-platform window management
- `parking_lot` - High-performance synchronization primitives
- `retry` - Robust retry mechanisms
- `serde` - Serialization framework
- `anyhow` - Error handling utilities
- `chrono` - Date and time handling
- `rusqlite` - SQLite database access
- `base64` - Base64 encoding/decoding
- `dirs` - Platform-specific directory paths
- `open` - Cross-platform file/URL opening
- `image` - Image processing for tray icons

[0.1.0]: https://github.com/atopx/CursorBar/releases/tag/v0.1.0

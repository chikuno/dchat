# Music API, Database Persistence, and File Upload Integration

This document describes the newly integrated features for music API support, database persistence for profiles/statuses, and file upload system.

## 1. Music API Integration (`music_api.rs`)

### Supported Providers
- **Spotify** - Full track metadata and 30-second previews
- **Apple Music** - Track information with 30-second previews

### Features

#### Spotify Integration
```rust
use dchat_bots::MusicApiClient;

// Create client
let mut client = MusicApiClient::new();

// Authenticate with client credentials
let token = MusicApiClient::authenticate_spotify(
    "your_client_id",
    "your_client_secret"
).await?;

client.set_spotify_token(token);

// Search for tracks
let tracks = client.search_spotify("bohemian rhapsody", 10).await?;

// Get specific track
let track = client.get_spotify_track("spotify:track:4u7EnebtmKWzUH433cf5Qv").await?;
```

#### Apple Music Integration
```rust
// Set Apple Music developer token
client.set_apple_music_token("your_developer_token".to_string());

// Search for tracks (country code required)
let tracks = client.search_apple_music("hello adele", 10, "us").await?;

// Get specific track
let track = client.get_apple_music_track("1234567890", "us").await?;
```

#### Track Metadata
```rust
pub struct MusicApiTrack {
    pub provider: MusicProvider,      // Spotify, AppleMusic, etc.
    pub track_id: String,              // Provider-specific ID
    pub track_name: String,            // Song name
    pub artist_name: String,           // Primary artist
    pub album_name: Option<String>,    // Album name
    pub album_art_url: Option<String>, // Cover art (600x600)
    pub preview_url: Option<String>,   // 30-second preview MP3/M4A
}
```

### API Setup

#### Spotify API
1. Create app at https://developer.spotify.com/dashboard
2. Get Client ID and Client Secret
3. Use client credentials flow for authentication
4. Token expires after 1 hour - refresh as needed

#### Apple Music API
1. Enroll in Apple Developer Program
2. Create MusicKit identifier
3. Generate JWT developer token
4. Token valid for 6 months

### Rate Limits
- **Spotify**: ~1 request per second for search
- **Apple Music**: 1 request per second per token

## 2. Database Persistence (`profile_storage.rs`)

### Features
- User profile storage (username, bio, pictures, metadata)
- Privacy settings with granular controls
- User status storage with automatic expiration
- Search capabilities

### Database Schema

#### user_profiles
```sql
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    bio TEXT,
    profile_picture_file_id TEXT,
    profile_picture_unique_id TEXT,
    profile_picture_small TEXT,
    profile_picture_large TEXT,
    profile_picture_uploaded_at TEXT,
    online_status TEXT NOT NULL DEFAULT 'Offline',
    last_seen TEXT,
    created_at TEXT NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    metadata TEXT
);
```

#### profile_privacy
```sql
CREATE TABLE profile_privacy (
    user_id TEXT PRIMARY KEY,
    profile_picture_visibility TEXT NOT NULL DEFAULT 'Everyone',
    profile_picture_allowed TEXT,
    profile_picture_blocked TEXT,
    status_visibility TEXT NOT NULL DEFAULT 'Everyone',
    status_allowed TEXT,
    status_blocked TEXT,
    last_seen_visibility TEXT NOT NULL DEFAULT 'Everyone',
    last_seen_allowed TEXT,
    last_seen_blocked TEXT,
    bio_visibility TEXT NOT NULL DEFAULT 'Everyone',
    bio_allowed TEXT,
    bio_blocked TEXT,
    message_visibility TEXT NOT NULL DEFAULT 'Everyone',
    message_allowed TEXT,
    message_blocked TEXT
);
```

#### user_statuses
```sql
CREATE TABLE user_statuses (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    status_type TEXT NOT NULL,  -- 'Text', 'Image', 'Video', 'Audio'
    status_data TEXT NOT NULL,   -- JSON with type-specific data
    caption TEXT,
    background_color TEXT,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    view_count INTEGER NOT NULL DEFAULT 0,
    viewers TEXT                 -- JSON array of user IDs
);
```

### Usage

#### Initialize Storage
```rust
use dchat_bots::ProfileStorage;
use sqlx::SqlitePool;

let pool = SqlitePool::connect("sqlite:dchat_bots.db").await?;
let storage = ProfileStorage::new(pool);
storage.init_schema().await?;
```

#### Save Profile
```rust
let profile = UserProfile {
    user_id: user_id.clone(),
    username: "johndoe".to_string(),
    display_name: "John Doe".to_string(),
    bio: Some("Software developer".to_string()),
    profile_picture: Some(profile_pic),
    online_status: OnlineStatus::Online,
    privacy: PrivacySettings::default(),
    is_verified: false,
    // ... other fields
};

storage.save_profile(&profile).await?;
```

#### Get Profile
```rust
// By user ID
let profile = storage.get_profile(&user_id).await?;

// By username
let profile = storage.get_profile_by_username("johndoe").await?;
```

#### Save Status
```rust
let status = UserStatus {
    id: Uuid::new_v4(),
    status_type: StatusType::Audio {
        audio_file_id: "audio_123".to_string(),
        background_image_id: Some("bg_456".to_string()),
        duration: 243,
        title: Some("Bohemian Rhapsody".to_string()),
        artist: Some("Queen".to_string()),
        music_api_track_id: Some(spotify_track),
    },
    caption: Some("🎵 Classic rock!".to_string()),
    expires_at: Utc::now() + Duration::hours(24),
    // ... other fields
};

storage.save_status(&user_id, &status).await?;
```

#### Get Active Statuses
```rust
let statuses = storage.get_active_statuses(&user_id).await?;
```

#### Cleanup Expired Statuses
```rust
// Run periodically (e.g., every hour)
let deleted_count = storage.cleanup_expired_statuses().await?;
```

#### Search Profiles
```rust
let profiles = storage.search_profiles("john", 20).await?;
```

## 3. File Upload System (`file_upload.rs`)

### Features
- Multi-format support (photos, videos, audio, documents, stickers)
- Automatic file validation (size, MIME type)
- SHA-256 checksum for integrity
- Organized storage by media type
- Thumbnail generation (placeholder implemented)

### Supported Media Types
```rust
pub enum MediaFileType {
    Photo,
    Video,
    Audio,
    Voice,
    Document,
    Sticker,
    Animation,
    VideoNote,
}
```

### Upload Configuration
```rust
pub struct UploadConfig {
    pub storage_path: PathBuf,           // Default: ./data/uploads
    pub max_photo_size: u64,             // Default: 10 MB
    pub max_video_size: u64,             // Default: 100 MB
    pub max_audio_size: u64,             // Default: 50 MB
    pub max_document_size: u64,          // Default: 100 MB
    pub allowed_photo_types: Vec<String>,
    pub allowed_video_types: Vec<String>,
    pub allowed_audio_types: Vec<String>,
    pub allowed_document_types: Vec<String>,
}
```

### Usage

#### Initialize Upload Manager
```rust
use dchat_bots::{FileUploadManager, UploadConfig};

// With defaults
let manager = FileUploadManager::with_defaults();

// With custom config
let config = UploadConfig {
    storage_path: PathBuf::from("/var/dchat/uploads"),
    max_photo_size: 20 * 1024 * 1024,  // 20 MB
    ..Default::default()
};
let manager = FileUploadManager::new(config);

// Initialize storage directories
manager.init_storage().await?;
```

#### Upload File
```rust
use dchat_bots::MediaFileType;

let file_data = std::fs::read("photo.jpg")?;

let uploaded = manager.upload_file(
    MediaFileType::Photo,
    file_data,
    Some("image/jpeg".to_string()),
    Some(1920),  // width
    Some(1080),  // height
    None,        // duration
).await?;

println!("File ID: {}", uploaded.file_id);
println!("Checksum: {}", uploaded.checksum);
```

#### Get File
```rust
let file_data = manager.get_file(&uploaded.file_id).await?;
```

#### Delete File
```rust
manager.delete_file(&uploaded.file_id).await?;
```

#### Generate Thumbnail
```rust
let thumbnail = manager.generate_thumbnail(
    &uploaded.file_id,
    320,  // max width
    320,  // max height
).await?;
```

#### Storage Statistics
```rust
let stats = manager.get_storage_stats().await?;
println!("Total files: {}", stats.total_files);
println!("Total size: {:.2} GB", stats.size_gb());
```

### Storage Structure
```
./data/uploads/
├── photos/
│   ├── uuid1.jpg
│   └── uuid2.png
├── videos/
│   └── uuid3.mp4
├── audio/
│   └── uuid4.mp3
├── voice/
│   └── uuid5.ogg
├── documents/
│   └── uuid6.pdf
├── stickers/
│   └── uuid7.webp
├── animations/
│   └── uuid8.gif
└── thumbnails/
    └── thumb_uuid9.jpg
```

## Integration Example: Complete Audio Status Flow

```rust
use dchat_bots::*;
use sqlx::SqlitePool;

async fn create_audio_status_with_music(
    user_id: &dchat_core::types::UserId,
) -> dchat_core::Result<()> {
    // 1. Initialize components
    let pool = SqlitePool::connect("sqlite:dchat_bots.db").await?;
    let profile_storage = ProfileStorage::new(pool);
    let mut music_client = MusicApiClient::new();
    let file_manager = FileUploadManager::with_defaults();
    
    file_manager.init_storage().await?;
    profile_storage.init_schema().await?;
    
    // 2. Authenticate with Spotify
    let spotify_token = MusicApiClient::authenticate_spotify(
        "client_id",
        "client_secret"
    ).await?;
    music_client.set_spotify_token(spotify_token);
    
    // 3. Search for track
    let tracks = music_client.search_spotify("bohemian rhapsody queen", 1).await?;
    let track = tracks.into_iter().next()
        .ok_or_else(|| dchat_core::Error::not_found("Track not found"))?;
    
    // 4. Upload background image
    let bg_image_data = std::fs::read("background.jpg")?;
    let bg_image = file_manager.upload_file(
        MediaFileType::Photo,
        bg_image_data,
        Some("image/jpeg".to_string()),
        Some(1920),
        Some(1080),
        None,
    ).await?;
    
    // 5. Upload audio file (if you have the actual audio)
    let audio_data = std::fs::read("preview.mp3")?;
    let audio_file = file_manager.upload_file(
        MediaFileType::Audio,
        audio_data,
        Some("audio/mpeg".to_string()),
        None,
        None,
        Some(30),  // 30 seconds
    ).await?;
    
    // 6. Create status
    let status = UserStatus {
        id: uuid::Uuid::new_v4(),
        status_type: StatusType::Audio {
            audio_file_id: audio_file.file_id,
            background_image_id: Some(bg_image.file_id),
            duration: 243,
            title: Some(track.track_name.clone()),
            artist: Some(track.artist_name.clone()),
            music_api_track_id: Some(track),
        },
        caption: Some("🎵 Listening to Queen!".to_string()),
        background_color: Some("#1DB954".to_string()),  // Spotify green
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        view_count: 0,
        viewers: Vec::new(),
    };
    
    // 7. Save to database
    profile_storage.save_status(user_id, &status).await?;
    
    println!("Audio status created with Spotify integration!");
    Ok(())
}
```

## Running Tests

All modules include comprehensive tests:

```bash
# Test music API client
cargo test -p dchat-bots music_api::tests

# Test profile storage
cargo test -p dchat-bots profile_storage::tests

# Test file upload
cargo test -p dchat-bots file_upload::tests

# Run all bot tests
cargo test -p dchat-bots
```

## Deployment Considerations

### Database
- Use SQLite for development/small deployments
- For production, consider PostgreSQL with proper connection pooling
- Run `cleanup_expired_statuses()` periodically (cron job every hour)
- Back up database regularly

### File Storage
- Local filesystem works for small deployments
- For production, consider:
  - S3-compatible storage (AWS S3, MinIO, DigitalOcean Spaces)
  - CDN for media delivery (CloudFlare, Fastly)
  - Separate storage service with API
- Implement cleanup for orphaned files

### Music APIs
- Store API credentials securely (environment variables, secrets manager)
- Implement token refresh for Spotify (1-hour expiry)
- Cache track metadata to reduce API calls
- Respect rate limits (use token bucket or similar)

### Performance
- Index frequently queried fields (username, expires_at)
- Use connection pooling for database
- Implement caching layer (Redis) for profiles
- Compress media files before upload
- Generate thumbnails asynchronously

## Next Steps

1. **Thumbnail Generation**: Integrate `image` crate for actual thumbnail generation
2. **CDN Integration**: Add S3/CDN upload support
3. **Music Cache**: Cache track metadata in database
4. **Token Refresh**: Implement automatic Spotify token refresh
5. **Batch Operations**: Add bulk upload/delete operations
6. **Streaming**: Implement chunked upload for large files
7. **Real-time**: WebSocket notifications for status views
8. **Analytics**: Track popular tracks and usage statistics

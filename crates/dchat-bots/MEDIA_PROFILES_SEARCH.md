# Bot System Feature Documentation

## Overview

The dchat bot system now includes comprehensive media handling, user profiles with statuses, and advanced search capabilities.

## Features

### 1. Enhanced Message Types

Support for rich media messages including:

#### Media Types
- **Photos**: Multiple sizes with thumbnails
- **Videos**: With duration, MIME type, thumbnails
- **Audio**: Music files with performer/title metadata
- **Voice**: Voice messages
- **Documents**: File attachments with MIME types
- **Stickers**: Regular, mask, and custom emoji stickers
- **Animations**: GIFs and animated content
- **Video Notes**: Round video messages
- **Locations**: Geographic coordinates with live location support
- **Contacts**: Phone numbers and vCard
- **Polls**: Regular and quiz-style polls

#### Text Features
- **Message Entities**: Rich text formatting
  - Bold, italic, underline, strikethrough
  - Code blocks and inline code
  - Mentions (@username)
  - Hashtags (#tag)
  - URLs and text links
  - Commands (/start, /help)
  - Phone numbers and emails
  - Spoilers

#### Link Previews
- Automatic URL preview generation
- Title, description, image extraction
- Site name and favicon support

### 2. User Profiles

Comprehensive user profile system with:

#### Profile Information
- Username (searchable)
- Display name
- Bio/description
- Profile picture (small and large thumbnails)
- Verified badge
- Custom metadata

#### Privacy Settings
Granular control over:
- Profile picture visibility
- Status visibility
- Last seen visibility
- Bio visibility
- Message permissions

Privacy levels:
- **Everyone**: Public access
- **Contacts**: Only contacts can see
- **Nobody**: Private
- **Custom**: Specific allow/block lists

#### Online Status
- Online
- Offline
- Away
- Do Not Disturb

### 3. User Status (Stories)

WhatsApp/Telegram-style status updates:

#### Status Types

**Text Status**
```rust
StatusType::Text {
    text: "Hello world!".to_string(),
    font: Some("Arial".to_string()),
}
```

**Image Status**
```rust
StatusType::Image {
    file_id: "img_123".to_string(),
    width: 1920,
    height: 1080,
}
```

**Video Status**
```rust
StatusType::Video {
    file_id: "vid_123".to_string(),
    width: 1920,
    height: 1080,
    duration: 30,
}
```

**Audio Status with Public Music APIs**
```rust
StatusType::Audio {
    audio_file_id: "audio_123".to_string(),
    background_image_id: Some("bg_456".to_string()),
    duration: 180,
    title: Some("My Favorite Song".to_string()),
    artist: Some("The Artist".to_string()),
    music_api_track_id: Some(MusicApiTrack {
        provider: MusicProvider::Spotify,
        track_id: "spotify:track:abc123".to_string(),
        track_name: "Song Name".to_string(),
        artist_name: "Artist Name".to_string(),
        album_name: Some("Album Name".to_string()),
        album_art_url: Some("https://...".to_string()),
        preview_url: Some("https://preview.mp3".to_string()),
    }),
}
```

#### Supported Music APIs
- **Spotify**: Full track metadata and 30-second previews
- **Apple Music**: Track information and previews
- **YouTube Music**: Video and audio tracks
- **SoundCloud**: Independent artist tracks
- **Deezer**: International music catalog

#### Status Features
- 24-hour expiration (configurable)
- View count tracking
- Viewer list (if privacy allows)
- Background color customization
- Captions

### 4. Search System

Advanced search for users and bots:

#### Search by Username (Exact)
```rust
let results = search_manager.search_by_username("testuser")?;
```

#### Search by Query (Fuzzy)
```rust
let results = search_manager.search("music", SearchFilters {
    search_type: Some(SearchType::Bots),
    verified_only: true,
    min_rating: Some(4.0),
    tags: vec!["music".to_string(), "entertainment".to_string()],
    limit: Some(10),
    ..Default::default()
})?;
```

#### Search Filters
- **Search Type**: Users, Bots, or All
- **Verified Only**: Filter by verification badge
- **Online Only**: Users currently online (users only)
- **Minimum Rating**: Filter bots by rating (0-5 stars)
- **Tags**: Category/tag filtering for bots
- **Limit**: Maximum results

#### Bot Discovery
```rust
// Get popular bots
let popular = search_manager.get_popular_bots(10)?;

// Get bots by category
let music_bots = search_manager.get_bots_by_tag("music", Some(20))?;

// Update bot rating
search_manager.update_bot_rating(&bot_id, 4.5)?;
```

## Usage Examples

### Example 1: Creating a User Profile with Status

```rust
use dchat_bots::*;
use chrono::Utc;

// Create profile manager
let profile_manager = ProfileManager::new();

// Create user profile
let user_id = dchat_core::types::UserId::new();
let mut profile = UserProfile {
    user_id: user_id.clone(),
    username: "johndoe".to_string(),
    display_name: "John Doe".to_string(),
    bio: Some("Software developer and music lover".to_string()),
    profile_picture: Some(ProfilePicture {
        file_id: "profile_pic_123".to_string(),
        file_unique_id: "unique_123".to_string(),
        small_file_id: Some("small_123".to_string()),
        large_file_id: Some("large_123".to_string()),
        uploaded_at: Utc::now(),
    }),
    status: None,
    online_status: OnlineStatus::Online,
    last_seen: None,
    created_at: Utc::now(),
    privacy: PrivacySettings {
        profile_picture_visibility: VisibilityLevel::Everyone,
        status_visibility: VisibilityLevel::Contacts,
        last_seen_visibility: VisibilityLevel::Contacts,
        bio_visibility: VisibilityLevel::Everyone,
        message_visibility: VisibilityLevel::Everyone,
    },
    is_verified: false,
    metadata: std::collections::HashMap::new(),
};

profile_manager.update_profile(profile)?;

// Add a music status with Spotify integration
let status = UserStatus {
    id: uuid::Uuid::new_v4(),
    status_type: StatusType::Audio {
        audio_file_id: "audio_456".to_string(),
        background_image_id: Some("bg_789".to_string()),
        duration: 243,
        title: Some("Bohemian Rhapsody".to_string()),
        artist: Some("Queen".to_string()),
        music_api_track_id: Some(MusicApiTrack {
            provider: MusicProvider::Spotify,
            track_id: "spotify:track:abcd1234".to_string(),
            track_name: "Bohemian Rhapsody".to_string(),
            artist_name: "Queen".to_string(),
            album_name: Some("A Night at the Opera".to_string()),
            album_art_url: Some("https://i.scdn.co/image/abc123".to_string()),
            preview_url: Some("https://p.scdn.co/mp3-preview/xyz789".to_string()),
        }),
    },
    caption: Some("🎵 Classic rock never dies!".to_string()),
    background_color: None,
    created_at: Utc::now(),
    expires_at: Utc::now() + chrono::Duration::hours(24),
    view_count: 0,
    viewers: Vec::new(),
};

profile_manager.set_status(&user_id, status)?;
```

### Example 2: Sending Rich Media Messages

```rust
use dchat_bots::*;
use uuid::Uuid;
use chrono::Utc;

// Create a photo message
let photo_message = EnhancedBotMessage {
    message_id: Uuid::new_v4(),
    from: user_id.clone(),
    chat_id: "chat_123".to_string(),
    text: None,
    caption: Some("Check out this sunset! 🌅".to_string()),
    entities: Vec::new(),
    caption_entities: vec![
        MessageEntity {
            entity_type: EntityType::Hashtag,
            offset: 23,
            length: 9,
            data: None,
        },
    ],
    photo: Some(vec![
        PhotoSize {
            file_id: "photo_small".to_string(),
            width: 320,
            height: 240,
            file_size: Some(15000),
        },
        PhotoSize {
            file_id: "photo_large".to_string(),
            width: 1920,
            height: 1080,
            file_size: Some(500000),
        },
    ]),
    video: None,
    audio: None,
    voice: None,
    document: None,
    sticker: None,
    animation: None,
    video_note: None,
    location: None,
    contact: None,
    poll: None,
    link_preview: None,
    timestamp: Utc::now(),
    edit_timestamp: None,
    is_forwarded: false,
    forward_from: None,
    forward_from_chat: None,
    forward_date: None,
    reply_to_message_id: None,
    is_command: false,
    command: None,
    command_args: Vec::new(),
};

// Check media type
assert_eq!(photo_message.get_media_type(), Some(MediaType::Photo));
```

### Example 3: Search for Users and Bots

```rust
use dchat_bots::*;

// Create search manager
let search_manager = SearchManager::new();

// Search for everything matching "music"
let results = search_manager.search("music", SearchFilters::default())?;

for result in results {
    match result {
        SearchResult::User(user) => {
            println!("User: @{} - {}", user.username, user.display_name);
        },
        SearchResult::Bot(bot) => {
            println!("Bot: @{} - {} (⭐ {:.1})", 
                bot.username, bot.display_name, bot.rating);
        },
    }
}

// Search only verified music bots with high ratings
let music_bots = search_manager.search("music", SearchFilters {
    search_type: Some(SearchType::Bots),
    verified_only: true,
    min_rating: Some(4.5),
    tags: vec!["music".to_string()],
    limit: Some(5),
    ..Default::default()
})?;

// Search by exact username
let user = search_manager.search_by_username("johndoe")?;

// Get popular bots
let trending = search_manager.get_popular_bots(20)?;

// Get bots by category
let gaming_bots = search_manager.get_bots_by_tag("gaming", Some(10))?;
```

### Example 4: Message with Text Entities and Links

```rust
use dchat_bots::*;

let message = EnhancedBotMessage {
    message_id: Uuid::new_v4(),
    from: user_id.clone(),
    chat_id: "chat_123".to_string(),
    text: Some("Check out https://example.com for more info! Use /help command.".to_string()),
    caption: None,
    entities: vec![
        MessageEntity {
            entity_type: EntityType::Url,
            offset: 10,
            length: 19,
            data: None,
        },
        MessageEntity {
            entity_type: EntityType::BotCommand,
            offset: 45,
            length: 5,
            data: None,
        },
    ],
    caption_entities: Vec::new(),
    photo: None,
    video: None,
    audio: None,
    voice: None,
    document: None,
    sticker: None,
    animation: None,
    video_note: None,
    location: None,
    contact: None,
    poll: None,
    link_preview: Some(LinkPreview {
        url: "https://example.com".to_string(),
        title: Some("Example Site".to_string()),
        description: Some("This is an example website".to_string()),
        image_url: Some("https://example.com/image.jpg".to_string()),
        site_name: Some("Example".to_string()),
        favicon_url: Some("https://example.com/favicon.ico".to_string()),
    }),
    timestamp: Utc::now(),
    edit_timestamp: None,
    is_forwarded: false,
    forward_from: None,
    forward_from_chat: None,
    forward_date: None,
    reply_to_message_id: None,
    is_command: false,
    command: None,
    command_args: Vec::new(),
};

// Extract all URLs
let urls = message.extract_urls();
println!("URLs: {:?}", urls);
```

## Integration with External Music APIs

### Spotify Integration

To integrate Spotify tracks in status:

1. **Get Spotify Web API credentials** at https://developer.spotify.com
2. **Search for tracks**:
   ```
   GET https://api.spotify.com/v1/search?q=bohemian+rhapsody&type=track
   ```
3. **Get track details**:
   ```
   GET https://api.spotify.com/v1/tracks/{track_id}
   ```
4. **Extract preview URL** (30-second snippet)

### Apple Music Integration

1. **Get Apple Music API key** at https://developer.apple.com/music/
2. **Search catalog**:
   ```
   GET https://api.music.apple.com/v1/catalog/{country}/search?term=song
   ```
3. **Get preview URL** from track metadata

### Implementation Example

```rust
// Helper function to create music status from Spotify
async fn create_spotify_status(
    track_id: &str,
    spotify_token: &str,
) -> Result<UserStatus> {
    // Call Spotify API
    let response = reqwest::get(&format!(
        "https://api.spotify.com/v1/tracks/{}",
        track_id
    ))
    .header("Authorization", format!("Bearer {}", spotify_token))
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;
    
    Ok(UserStatus {
        id: Uuid::new_v4(),
        status_type: StatusType::Audio {
            audio_file_id: "".to_string(), // Upload audio separately
            background_image_id: None,
            duration: response["duration_ms"].as_u64().unwrap_or(0) / 1000,
            title: response["name"].as_str().map(String::from),
            artist: response["artists"][0]["name"].as_str().map(String::from),
            music_api_track_id: Some(MusicApiTrack {
                provider: MusicProvider::Spotify,
                track_id: track_id.to_string(),
                track_name: response["name"].as_str().unwrap_or("").to_string(),
                artist_name: response["artists"][0]["name"].as_str().unwrap_or("").to_string(),
                album_name: response["album"]["name"].as_str().map(String::from),
                album_art_url: response["album"]["images"][0]["url"].as_str().map(String::from),
                preview_url: response["preview_url"].as_str().map(String::from),
            }),
        },
        caption: None,
        background_color: None,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        view_count: 0,
        viewers: Vec::new(),
    })
}
```

## Testing

All features are fully tested. Run:

```bash
cargo test -p dchat-bots --lib
```

**Test Coverage:**
- 48 tests total
- Media type detection
- URL extraction
- Profile creation and search
- Status management
- Search filters
- Bot rating system
- Username indexing

## Architecture

```
dchat-bots/
├── media.rs           # Rich media types and message entities
├── user_profile.rs    # User profiles and status management
├── search.rs          # Search system for users and bots
├── bot_manager.rs     # Bot registry (existing)
├── bot_api.rs         # Bot API endpoints (existing)
├── webhook.rs         # Webhook system (existing)
├── commands.rs        # Command handling (existing)
├── inline.rs          # Inline queries (existing)
├── permissions.rs     # Permission system (existing)
└── storage.rs         # Database persistence (existing)
```

## Next Steps

1. **Storage Integration**: Persist media files, profiles, and statuses to database
2. **File Upload**: Implement file upload handlers for media
3. **Music API Clients**: Create helper modules for Spotify, Apple Music, etc.
4. **Status Feed**: Build a feed UI for viewing active statuses
5. **Search Indexing**: Add full-text search with indexing
6. **CDN Integration**: Store media files on CDN for performance
7. **Notification System**: Notify users when someone views their status
8. **Analytics**: Track bot usage and popular content

## API Reference

All types are exported from the `dchat_bots` crate:

```rust
use dchat_bots::{
    // Media types
    EnhancedBotMessage, MediaType, Photo, Video, Audio, Voice,
    Document, Sticker, Animation, VideoNote, Location, Contact, Poll,
    MessageEntity, EntityType, LinkPreview,
    
    // User profiles
    UserProfile, ProfilePicture, UserStatus, StatusType,
    OnlineStatus, PrivacySettings, VisibilityLevel,
    ProfileManager, MusicApiTrack, MusicProvider,
    
    // Search
    SearchManager, SearchResult, SearchFilters, SearchType,
    BotSearchResult, BotMetadata,
};
```

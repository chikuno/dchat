# dchat Marketplace: Expanded Features & On-Chain Storage

## Overview

The dchat marketplace now supports comprehensive digital asset trading with on-chain storage for ownership verification and escrow protection. Users can buy, sell, and transfer:

- **Bots** (full ownership transfer)
- **Channels** (complete ownership transfer)
- **Emoji Packs** (custom animated/static emojis)
- **Images/Artwork** (with licensing)
- **NFTs** (collectible tokens)
- **Channel Memberships** (time-based access passes)
- **Sticker Packs** (content packs)
- **Themes** (UI customization)

---

## On-Chain Storage Architecture

### Storage Types

```rust
pub enum OnChainStorageType {
    /// Stored on chat chain (ownership, metadata)
    ChatChain,
    /// Stored on currency chain (financial data)
    CurrencyChain,
    /// Stored on IPFS (content files)
    Ipfs,
    /// Hybrid (metadata on-chain, content on IPFS)
    Hybrid,
}
```

### Storage Matrix

| Asset Type | Storage Type | On-Chain Address | Content Location |
|-----------|--------------|------------------|------------------|
| Bot | Hybrid | `0xbot{uuid}` | Code on IPFS, ownership on chain |
| Channel | Hybrid | `0xchan{uuid}` | Messages distributed, ownership on chain |
| EmojiPack | Ipfs | None | All content on IPFS |
| Image | Hybrid | `0ximg{uuid}` | Image on IPFS, license on chain |
| NFT | ChatChain | `0xnft{token_id}` | Metadata + image on chain |
| Membership | ChatChain | `0xmem{uuid}` | Access rights on chain |
| StickerPack | Ipfs | None | All content on IPFS |
| Theme | Ipfs | None | All files on IPFS |

---

## Bot Trading

### How It Works

1. **Registration**: Bot owner registers bot for trading
2. **Listing**: Creates marketplace listing with price
3. **Escrow**: Buyer purchases, tokens locked in escrow
4. **Transfer**: Bot ownership transferred on-chain
5. **Release**: Seller gets tokens after transfer confirmed

### Bot Ownership Structure

```rust
pub struct BotOwnership {
    pub bot_id: Uuid,
    pub bot_username: String,
    pub current_owner: UserId,
    pub previous_owners: Vec<(UserId, DateTime<Utc>)>,
    pub on_chain_address: String,
    pub transfer_count: u32,
}
```

### Example: Selling a Bot

```rust
// 1. Register bot ownership
let bot_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let on_chain_address = marketplace.register_bot_ownership(
    bot_id,
    "my_auto_responder_bot".to_string(),
    seller_id,
)?;

// 2. Create marketplace listing
let listing_id = marketplace.create_listing(
    seller_id,
    "Advanced Auto-Responder Bot".to_string(),
    "AI-powered bot with 50+ commands".to_string(),
    DigitalGoodType::Bot,
    PricingModel::OneTime { price: 5000 },
    "QmBotCodeHash".to_string(),
    OnChainStorageType::Hybrid,
    None, // nft_token_id
    Some(bot_id),
    None, // channel_id
    None, // membership_duration_days
)?;

// 3. Buyer purchases (escrow auto-created)
let purchase_id = marketplace.purchase(
    buyer_id,
    listing_id,
    5000,
    "0x123...transaction_hash".to_string(),
)?;

// 4. After buyer confirms, complete transfer
marketplace.complete_purchase_transfer(purchase_id, escrow_id)?;

// Bot ownership now transferred to buyer!
```

### CLI Commands

```bash
# Register bot for trading
dchat marketplace register-bot \
  --bot-id <uuid> \
  --username my_bot \
  --owner <your_user_id>

# List bot for sale
dchat marketplace create-listing \
  --creator-id <your_id> \
  --title "My Amazing Bot" \
  --description "Does X, Y, Z" \
  --item-type bot \
  --price 5000 \
  --bot-id <bot_uuid>

# Buy bot
dchat marketplace buy \
  --buyer-id <your_id> \
  --listing-id <listing_uuid>

# Check bot ownership
dchat marketplace bot-ownership --bot-id <uuid>

# List my bots
dchat marketplace my-bots --user-id <your_id>
```

---

## Channel Ownership Trading

### How It Works

1. **Registration**: Channel owner registers channel for trading
2. **Listing**: Lists channel with current member count
3. **Escrow**: Buyer purchases, tokens locked
4. **Transfer**: Channel ownership transferred on chat chain
5. **Release**: Seller receives payment, buyer becomes new owner

### Channel Ownership Structure

```rust
pub struct ChannelOwnership {
    pub channel_id: Uuid,
    pub channel_name: String,
    pub current_owner: UserId,
    pub previous_owners: Vec<(UserId, DateTime<Utc>)>,
    pub on_chain_address: String,
    pub member_count: u64,
    pub transfer_count: u32,
}
```

### Example: Selling a Channel

```rust
// 1. Register channel ownership
let channel_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001")?;
let on_chain_address = marketplace.register_channel_ownership(
    channel_id,
    "Crypto Trading Signals".to_string(),
    seller_id,
    5000, // 5000 members
)?;

// 2. List channel for sale
let listing_id = marketplace.create_listing(
    seller_id,
    "Crypto Trading Channel - 5K Members".to_string(),
    "Active community with daily signals".to_string(),
    DigitalGoodType::Channel,
    PricingModel::OneTime { price: 50000 },
    "QmChannelMetadata".to_string(),
    OnChainStorageType::Hybrid,
    None,
    None,
    Some(channel_id),
    None,
)?;

// 3. Purchase and transfer
let purchase_id = marketplace.purchase(buyer_id, listing_id, 50000, "tx_hash")?;
marketplace.complete_purchase_transfer(purchase_id, escrow_id)?;

// Buyer now owns the channel!
```

### CLI Commands

```bash
# Register channel for trading
dchat marketplace register-channel \
  --channel-id <uuid> \
  --name "My Channel" \
  --owner <your_id> \
  --member-count 5000

# List channel for sale
dchat marketplace create-listing \
  --creator-id <your_id> \
  --title "Active Trading Channel - 5K Members" \
  --item-type channel \
  --price 50000 \
  --channel-id <channel_uuid>

# Buy channel
dchat marketplace buy \
  --buyer-id <your_id> \
  --listing-id <listing_uuid>

# Check channel ownership history
dchat marketplace channel-ownership --channel-id <uuid>

# List my channels
dchat marketplace my-channels --user-id <your_id>
```

---

## Emoji Pack Trading

### Emoji Pack Structure

```rust
pub struct EmojiPack {
    pub pack_id: Uuid,
    pub name: String,
    pub description: String,
    pub emoji_count: u32,
    pub creator: UserId,
    pub content_hash: String,
    pub preview_emojis: Vec<String>,
    pub is_animated: bool,
}
```

### Example: Selling Emoji Pack

```rust
// 1. Register emoji pack
let pack_id = marketplace.register_emoji_pack(
    "Cute Animals Pack".to_string(),
    "50 adorable animal emojis".to_string(),
    50, // emoji count
    creator_id,
    "QmEmojiPackHash".to_string(),
    vec!["🐶".to_string(), "🐱".to_string(), "🐰".to_string()],
    true, // is_animated
)?;

// 2. List for sale
let listing_id = marketplace.create_listing(
    creator_id,
    "Cute Animals Emoji Pack".to_string(),
    "50 animated animal emojis".to_string(),
    DigitalGoodType::EmojiPack,
    PricingModel::OneTime { price: 300 },
    "QmEmojiPackHash".to_string(),
    OnChainStorageType::Ipfs,
    None,
    None,
    None,
    None,
)?;

// 3. Buy
marketplace.purchase(buyer_id, listing_id, 300, "tx_hash")?;
```

### CLI Commands

```bash
# Create emoji pack
dchat marketplace create-emoji-pack \
  --name "Cute Animals" \
  --description "50 animated emojis" \
  --emoji-count 50 \
  --creator-id <your_id> \
  --content-hash QmHash \
  --animated true

# List emoji pack
dchat marketplace create-listing \
  --title "Cute Animals Pack" \
  --item-type emoji-pack \
  --price 300

# Browse emoji packs
dchat marketplace list --item-type emoji-pack

# Buy emoji pack
dchat marketplace buy --buyer-id <your_id> --listing-id <uuid>
```

---

## Image/Artwork Trading

### Image Structure with Licensing

```rust
pub struct ImageArtwork {
    pub image_id: Uuid,
    pub title: String,
    pub description: String,
    pub creator: UserId,
    pub content_hash: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub license_type: LicenseType,
}

pub enum LicenseType {
    AllRightsReserved,
    CcBy,               // Creative Commons Attribution
    CcBySa,             // CC Attribution-ShareAlike
    CcByNd,             // CC Attribution-NoDerivs
    CcByNc,             // CC Attribution-NonCommercial
    PublicDomain,
}
```

### Example: Selling Artwork

```rust
// 1. Register image
let image_id = marketplace.register_image(
    "Cyberpunk Cityscape".to_string(),
    "4K digital art".to_string(),
    artist_id,
    "QmImageHash".to_string(),
    3840, // width
    2160, // height
    "png".to_string(),
    LicenseType::CcBy,
)?;

// 2. List for sale
let listing_id = marketplace.create_listing(
    artist_id,
    "Cyberpunk Cityscape - 4K".to_string(),
    "Original digital artwork, CC-BY license".to_string(),
    DigitalGoodType::Image,
    PricingModel::OneTime { price: 2000 },
    "QmImageHash".to_string(),
    OnChainStorageType::Hybrid,
    None,
    None,
    None,
    None,
)?;
```

### CLI Commands

```bash
# Register artwork
dchat marketplace register-image \
  --title "My Artwork" \
  --creator-id <your_id> \
  --content-hash QmHash \
  --width 3840 \
  --height 2160 \
  --format png \
  --license cc-by

# List artwork
dchat marketplace create-listing \
  --title "Amazing Artwork" \
  --item-type image \
  --price 2000

# Browse images
dchat marketplace list --item-type image

# Filter by license
dchat marketplace list-images --license cc-by
```

---

## Channel Membership Trading

### Membership Structure

```rust
pub struct ChannelMembership {
    pub membership_id: Uuid,
    pub channel_id: Uuid,
    pub holder: UserId,
    pub purchased_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_transferable: bool,
    pub access_level: MembershipAccessLevel,
}

pub enum MembershipAccessLevel {
    Basic,        // Read access
    Contributor,  // Can post messages
    Moderator,    // Can moderate
    Vip,          // Special perks
}
```

### How Membership Trading Works

1. **Create**: Channel owner lists membership for sale
2. **Duration**: Specify membership duration (days)
3. **Purchase**: Buyer gets time-based access pass
4. **Transfer**: Memberships can be resold (if transferable)
5. **Expiration**: Auto-expires after duration

### Example: Selling Channel Membership

```rust
// 1. List membership for sale
let listing_id = marketplace.create_listing(
    channel_owner_id,
    "VIP Membership - Premium Trading Channel".to_string(),
    "30-day VIP access with exclusive signals".to_string(),
    DigitalGoodType::Membership,
    PricingModel::Subscription { price_per_month: 1000 },
    "QmMembershipMetadata".to_string(),
    OnChainStorageType::ChatChain,
    None,
    None,
    Some(channel_id),
    Some(30), // 30 days duration
)?;

// 2. Buyer purchases
let purchase_id = marketplace.purchase(buyer_id, listing_id, 1000, "tx_hash")?;

// 3. Complete transfer (grants membership)
marketplace.complete_purchase_transfer(purchase_id, escrow_id)?;

// Buyer now has 30-day VIP membership!

// 4. Check membership
let is_member = marketplace.has_active_membership(channel_id, &buyer_id);

// 5. Resell membership (if transferable)
marketplace.transfer_membership(membership_id, new_buyer_id)?;
```

### CLI Commands

```bash
# List membership for sale
dchat marketplace create-listing \
  --creator-id <channel_owner_id> \
  --title "30-Day VIP Membership" \
  --item-type membership \
  --price 1000 \
  --channel-id <channel_uuid> \
  --membership-duration 30

# Buy membership
dchat marketplace buy \
  --buyer-id <your_id> \
  --listing-id <listing_uuid>

# Check my memberships
dchat marketplace my-memberships --user-id <your_id>

# Check channel membership status
dchat marketplace check-membership \
  --channel-id <uuid> \
  --user-id <your_id>

# Resell membership (if transferable)
dchat marketplace transfer-membership \
  --membership-id <uuid> \
  --new-holder <buyer_id>

# List all members of channel
dchat marketplace channel-members --channel-id <uuid>
```

---

## Escrow Integration

### Automatic Escrow Creation

**Every purchase automatically creates escrow** for buyer protection:

```rust
// When buyer purchases:
marketplace.purchase(buyer_id, listing_id, amount, tx_hash)?;

// Behind the scenes:
// 1. Tokens locked in escrow (30 days)
// 2. Listing marked as "in_escrow"
// 3. Seller cannot relist until escrow resolved
// 4. Buyer can confirm or dispute
```

### Escrow Flow

```
Purchase Initiated
        ↓
Tokens Locked in Escrow (30 days)
        ↓
Seller Delivers Asset
        ↓
Buyer Confirms Receipt
        ↓
Asset Ownership Transferred (on-chain)
        ↓
Tokens Released to Seller
```

### Handling Disputes

```bash
# If item not as described
dchat marketplace dispute \
  --escrow-id <uuid> \
  --reason item-not-as-described \
  --description "Bot doesn't work as advertised"

# Admin reviews and resolves
dchat marketplace resolve-dispute \
  --escrow-id <uuid> \
  --resolution refund-full
```

---

## On-Chain Verification

### Ownership Verification

All asset ownership is verifiable on-chain:

```bash
# Verify bot ownership
dchat chain verify-ownership \
  --asset-type bot \
  --asset-id <bot_uuid> \
  --owner <user_id>

# Verify channel ownership
dchat chain verify-ownership \
  --asset-type channel \
  --asset-id <channel_uuid> \
  --owner <user_id>

# Verify membership
dchat chain verify-membership \
  --channel-id <uuid> \
  --user-id <user_id>
```

### Ownership History

View complete ownership transfer history:

```bash
# Bot ownership history
dchat marketplace bot-ownership-history --bot-id <uuid>
```

**Output**:
```
Bot: my_auto_responder_bot (0xbot550e8400...)
Current Owner: alice@dchat
Transfer Count: 3

Ownership History:
1. 2025-01-15: Created by bob@dchat
2. 2025-02-01: Transferred to charlie@dchat (Sale: 5,000 tokens)
3. 2025-03-10: Transferred to dana@dchat (Sale: 8,000 tokens)
4. 2025-04-20: Transferred to alice@dchat (Sale: 12,000 tokens)
```

---

## Complete Trading Examples

### Example 1: Bot Trading

```bash
# Seller: Bob wants to sell his bot
dchat marketplace register-bot \
  --bot-id 550e8400-e29b-41d4-a716-446655440000 \
  --username bobs_helper_bot \
  --owner bob_user_id

dchat marketplace create-listing \
  --creator-id bob_user_id \
  --title "Advanced Helper Bot" \
  --description "AI-powered automation with 100+ commands" \
  --item-type bot \
  --price 10000 \
  --bot-id 550e8400-e29b-41d4-a716-446655440000

# Buyer: Alice buys the bot
dchat marketplace buy \
  --buyer-id alice_user_id \
  --listing-id <listing_uuid>

# Escrow created automatically, tokens locked

# Alice tests bot, confirms it works
dchat marketplace release-escrow --escrow-id <escrow_uuid>

# Ownership transferred, Bob gets 9,900 tokens (10,000 - 1% fee)
```

### Example 2: Channel Trading

```bash
# Seller: Channel owner lists 5K-member channel
dchat marketplace register-channel \
  --channel-id 660e8400-e29b-41d4-a716-446655440001 \
  --name "Crypto Trading Signals" \
  --owner seller_id \
  --member-count 5000

dchat marketplace create-listing \
  --creator-id seller_id \
  --title "Active Trading Channel - 5K Members" \
  --description "Daily signals, 80% win rate" \
  --item-type channel \
  --price 100000 \
  --channel-id 660e8400-e29b-41d4-a716-446655440001

# Buyer: Investor buys channel
dchat marketplace buy \
  --buyer-id investor_id \
  --listing-id <listing_uuid>

# Escrow holds 100,000 tokens

# Investor confirms ownership transfer
dchat marketplace release-escrow --escrow-id <escrow_uuid>

# Channel ownership transferred on chat chain
# Seller receives 99,000 tokens (100,000 - 1% fee)
```

### Example 3: Membership Trading

```bash
# Channel owner lists VIP membership
dchat marketplace create-listing \
  --creator-id channel_owner_id \
  --title "30-Day VIP Membership" \
  --description "Exclusive trading signals + private chat" \
  --item-type membership \
  --price 2000 \
  --channel-id <channel_uuid> \
  --membership-duration 30

# User buys membership
dchat marketplace buy \
  --buyer-id user_id \
  --listing-id <listing_uuid>

# User gets 30-day access pass
# After 15 days, user wants to resell membership
dchat marketplace transfer-membership \
  --membership-id <membership_uuid> \
  --new-holder new_user_id

# New user gets remaining 15 days of membership
```

---

## Economics & Token Flow

### Transaction Fees

All marketplace sales incur **1% transaction fee** (burned):

```
Sale Price: 10,000 tokens
Burned Fee: 100 tokens (1%)
Seller Receives: 9,900 tokens
```

### Price Discovery by Asset Type

**Typical Price Ranges**:

| Asset Type | Low | Medium | High | Premium |
|-----------|-----|--------|------|---------|
| Bot | 1,000 | 5,000 | 20,000 | 100,000+ |
| Channel | 10,000 | 50,000 | 200,000 | 1,000,000+ |
| EmojiPack | 100 | 500 | 2,000 | 10,000 |
| Image | 500 | 2,000 | 10,000 | 50,000+ |
| NFT | 1,000 | 10,000 | 100,000 | 10,000,000+ |
| Membership | 500/month | 2,000/month | 10,000/month | 50,000/month |

### Valuation Factors

**Bots**:
- Number of commands
- Complexity/AI features
- User base
- Revenue generation
- Update frequency

**Channels**:
- Member count
- Engagement rate
- Revenue/monetization
- Content quality
- Brand value

**Memberships**:
- Exclusive content quality
- Community activity
- Duration
- Access level perks

---

## API Reference

### MarketplaceManager Methods

#### Bot Methods

```rust
// Register bot for trading
pub fn register_bot_ownership(
    &mut self,
    bot_id: Uuid,
    bot_username: String,
    owner: UserId,
) -> Result<String>

// Transfer bot ownership
pub fn transfer_bot_ownership(
    &mut self,
    bot_id: Uuid,
    new_owner: UserId,
) -> Result<()>

// Get bot ownership info
pub fn get_bot_ownership(
    &self,
    bot_id: Uuid,
) -> Option<&BotOwnership>

// Get all bots owned by user
pub fn get_bots_by_owner(
    &self,
    owner: &UserId,
) -> Vec<&BotOwnership>
```

#### Channel Methods

```rust
// Register channel for trading
pub fn register_channel_ownership(
    &mut self,
    channel_id: Uuid,
    channel_name: String,
    owner: UserId,
    member_count: u64,
) -> Result<String>

// Transfer channel ownership
pub fn transfer_channel_ownership(
    &mut self,
    channel_id: Uuid,
    new_owner: UserId,
) -> Result<()>

// Get channel ownership info
pub fn get_channel_ownership(
    &self,
    channel_id: Uuid,
) -> Option<&ChannelOwnership>

// Get all channels owned by user
pub fn get_channels_by_owner(
    &self,
    owner: &UserId,
) -> Vec<&ChannelOwnership>
```

#### Membership Methods

```rust
// Grant channel membership
pub fn grant_membership(
    &mut self,
    channel_id: Uuid,
    holder: UserId,
    duration_days: u32,
) -> Result<Uuid>

// Transfer membership
pub fn transfer_membership(
    &mut self,
    membership_id: Uuid,
    new_holder: UserId,
) -> Result<()>

// Check active membership
pub fn has_active_membership(
    &self,
    channel_id: Uuid,
    holder: &UserId,
) -> bool

// Get user's memberships
pub fn get_memberships_by_holder(
    &self,
    holder: &UserId,
) -> Vec<&ChannelMembership>
```

---

## Security Considerations

### On-Chain Verification

- All ownership transfers recorded on chat chain
- Immutable transfer history
- Cryptographic proof of ownership
- Cannot forge ownership records

### Escrow Protection

- Automatic escrow on every purchase
- 30-day dispute window
- Multi-signature release
- Refund protection

### Asset Validation

```rust
// Before transfer, validate asset exists
if let Some(bot) = marketplace.get_bot_ownership(bot_id) {
    if bot.current_owner != seller_id {
        return Err(Error::validation("Seller doesn't own bot"));
    }
}

// Prevent double-spending
if listing.in_escrow {
    return Err(Error::validation("Item already in escrow"));
}
```

### Content Verification

- IPFS content hashes verified
- Cannot modify content after listing
- Immutable content addressing
- Tamper-proof storage

---

## Future Enhancements

### Planned Features

1. **Fractional Ownership**
   - Split channel ownership among multiple users
   - Proportional revenue sharing
   - DAO-style governance

2. **Subscription Management**
   - Auto-renewing memberships
   - Bulk discounts
   - Family/team plans

3. **Royalties System**
   - Creators get % on resales
   - Configurable royalty rates
   - Automatic distribution

4. **Auction System**
   - Timed auctions for rare assets
   - Bid increments
   - Reserve prices

5. **Bundle Deals**
   - Package multiple assets
   - Discount pricing
   - Theme + emoji + sticker bundles

6. **Reputation Integration**
   - Verified seller badges
   - Trust scores
   - Purchase history

---

## Summary

The expanded dchat marketplace provides:

✅ **Bot Trading** - Full ownership transfer with on-chain verification  
✅ **Channel Trading** - Complete channel ownership transfer  
✅ **Emoji Pack Trading** - Custom animated/static emoji packs  
✅ **Image Trading** - Artwork with licensing options  
✅ **NFT Trading** - Collectible tokens  
✅ **Membership Trading** - Time-based channel access passes  
✅ **Automatic Escrow** - Every purchase protected  
✅ **On-Chain Storage** - Ownership verification and history  
✅ **Transfer History** - Complete provenance tracking  
✅ **Dispute Resolution** - Fair resolution system  

**All assets can be bought, sold, and transferred with full on-chain verification and escrow protection.**

---

## Getting Started

```bash
# 1. Register your asset (bot, channel, etc.)
dchat marketplace register-bot --bot-id <uuid> --username my_bot

# 2. Create listing
dchat marketplace create-listing \
  --item-type bot \
  --price 5000 \
  --bot-id <uuid>

# 3. Buyer purchases (escrow auto-created)
dchat marketplace buy --listing-id <uuid>

# 4. Complete transfer (after escrow release)
# Ownership automatically transferred on-chain ✅
```

**Start trading today!** 🚀

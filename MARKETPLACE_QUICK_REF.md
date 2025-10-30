# Marketplace Quick Reference

## Asset Types & Storage

| Type | Storage | On-Chain | CLI Flag |
|------|---------|----------|----------|
| Bot | Hybrid | Yes | `--item-type bot` |
| Channel | ChatChain | Yes | `--item-type channel` |
| Emoji Pack | Ipfs | No | `--item-type emoji-pack` |
| Image | Hybrid | Yes | `--item-type image` |
| Membership | ChatChain | Yes | `--item-type membership` |
| NFT | Hybrid | Yes | `--item-type nft` |

## Quick Commands

### Register Assets
```bash
# Bot
cargo run -- marketplace register-bot --bot-id <uuid> --username <name> --owner <user_id>

# Channel
cargo run -- marketplace register-channel --channel-id <uuid> --name <name> --owner <user_id> --member-count <count>

# Emoji Pack
cargo run -- marketplace create-emoji-pack --name <name> --description <desc> --emoji-count <count> --creator-id <user_id> --content-hash <hash> --animated <true/false>

# Image
cargo run -- marketplace register-image --title <title> --description <desc> --creator-id <user_id> --content-hash <hash> --width <px> --height <px> --format <format> --license <type>
```

### Create Listings
```bash
# Bot
cargo run -- marketplace create-listing --creator-id <user_id> --title <title> --description <desc> --item-type bot --price <amount> --content-hash <hash> --bot-id <uuid>

# Channel
cargo run -- marketplace create-listing --creator-id <user_id> --title <title> --description <desc> --item-type channel --price <amount> --content-hash <hash> --channel-id <uuid>

# Membership (30 days)
cargo run -- marketplace create-listing --creator-id <user_id> --title <title> --description <desc> --item-type membership --price <amount> --content-hash <hash> --channel-id <uuid> --membership-duration 30

# Emoji Pack
cargo run -- marketplace create-listing --creator-id <user_id> --title <title> --description <desc> --item-type emoji-pack --price <amount> --content-hash <hash>

# Image
cargo run -- marketplace create-listing --creator-id <user_id> --title <title> --description <desc> --item-type image --price <amount> --content-hash <hash>
```

### Query Ownership
```bash
# Check bot ownership (shows transfer history)
cargo run -- marketplace bot-ownership --bot-id <uuid>

# Check channel ownership
cargo run -- marketplace channel-ownership --channel-id <uuid>

# My assets
cargo run -- marketplace my-bots --user-id <user_id>
cargo run -- marketplace my-channels --user-id <user_id>
```

### Membership Management
```bash
# Check membership status
cargo run -- marketplace check-membership --channel-id <uuid> --user-id <user_id>

# List my memberships
cargo run -- marketplace my-memberships --user-id <user_id>

# Transfer membership
cargo run -- marketplace transfer-membership --membership-id <uuid> --new-holder <user_id>

# List channel members
cargo run -- marketplace channel-members --channel-id <uuid>
```

### Trading
```bash
# Buy item (creates escrow automatically)
cargo run -- marketplace buy --buyer-id <user_id> --listing-id <uuid>

# Check creator stats
cargo run -- marketplace creator-stats --creator-id <user_id>
```

## License Types (for images)

| CLI Flag | Meaning |
|----------|---------|
| `all-rights-reserved` | Full copyright, no reuse |
| `cc-by` | Attribution required |
| `cc-by-sa` | Attribution + ShareAlike |
| `cc-by-nd` | Attribution + NoDerivs |
| `cc-by-nc` | Attribution + NonCommercial |
| `public-domain` | No restrictions |

## Pricing Guidelines

| Asset | Typical Range | Based On |
|-------|--------------|----------|
| Bot | 1K - 100K+ | Features, ratings |
| Channel | 10K - 1M+ | Members, activity |
| Emoji Pack | 500 - 5K | Count, quality |
| Image | 500 - 50K | Resolution, license |
| Membership | 500 - 50K/mo | Access level |

## API Methods (for developers)

### Bot Methods
```rust
marketplace.register_bot_ownership(bot_id, username, owner) -> String
marketplace.transfer_bot_ownership(bot_id, new_owner) -> Result<()>
marketplace.get_bot_ownership(bot_id) -> Option<&BotOwnership>
marketplace.get_bots_by_owner(owner) -> Vec<&BotOwnership>
```

### Channel Methods
```rust
marketplace.register_channel_ownership(channel_id, name, owner, member_count) -> String
marketplace.transfer_channel_ownership(channel_id, new_owner) -> Result<()>
marketplace.get_channel_ownership(channel_id) -> Option<&ChannelOwnership>
marketplace.get_channels_by_owner(owner) -> Vec<&ChannelOwnership>
```

### Emoji Pack Methods
```rust
marketplace.register_emoji_pack(name, desc, count, creator, hash, preview, animated) -> Uuid
marketplace.get_emoji_pack(pack_id) -> Option<&EmojiPack>
marketplace.get_emoji_packs_by_creator(creator) -> Vec<&EmojiPack>
```

### Image Methods
```rust
marketplace.register_image(title, desc, creator, hash, width, height, format, license) -> Uuid
marketplace.get_image(image_id) -> Option<&ImageArtwork>
marketplace.get_images_by_creator(creator) -> Vec<&ImageArtwork>
```

### Membership Methods
```rust
marketplace.grant_membership(channel_id, holder, duration_days) -> Uuid
marketplace.transfer_membership(membership_id, new_holder) -> Result<()>
marketplace.has_active_membership(channel_id, holder) -> bool
marketplace.get_memberships_by_holder(holder) -> Vec<&ChannelMembership>
marketplace.get_memberships_by_channel(channel_id) -> Vec<&ChannelMembership>
```

### Trading Methods
```rust
marketplace.create_listing(
    creator, title, description, good_type, pricing, content_hash,
    storage_type, nft_token_id, bot_id, channel_id, membership_duration
) -> Result<Uuid>

marketplace.purchase(buyer, listing_id, amount, tx_hash) -> Result<Uuid>
marketplace.complete_purchase_transfer(purchase_id, escrow_id) -> Result<()>
```

## Testing

```bash
# Run all marketplace tests (22 tests)
cargo test --package dchat-marketplace

# Run specific test
cargo test --package dchat-marketplace test_purchase_listing

# Run with output
cargo test --package dchat-marketplace -- --nocapture
```

## Common Workflows

### Bot Sale Workflow
1. Register bot: `register-bot`
2. Create listing: `create-listing --item-type bot --bot-id <uuid>`
3. Buyer purchases: `buy`
4. Escrow created automatically (30 days)
5. After inspection: Release escrow
6. Ownership transferred: `complete_purchase_transfer`
7. Verify: `bot-ownership` shows new owner

### Channel Sale Workflow
1. Register channel: `register-channel`
2. Create listing: `create-listing --item-type channel --channel-id <uuid>`
3. Buyer purchases: `buy`
4. Escrow period (30 days)
5. Complete transfer: Buyer gets full admin rights
6. Verify: `channel-ownership` shows new owner + transfer history

### Membership Sale Workflow
1. Create listing: `create-listing --item-type membership --membership-duration 30`
2. Buyer purchases: `buy`
3. Membership granted automatically with expiration
4. Check: `check-membership` returns ✅
5. View details: `my-memberships` shows expiration date
6. Optional transfer: `transfer-membership` (if transferable)

## Status Checks

All ✅ complete:
- 22/22 tests passing
- Clean compilation
- Full CLI implementation
- Automatic escrow integration
- Complete documentation

## Next Steps

- [ ] Integrate with CurrencyChainClient
- [ ] Integrate with ChatChainClient
- [ ] End-to-end testing
- [ ] Performance testing
- [ ] Security audit

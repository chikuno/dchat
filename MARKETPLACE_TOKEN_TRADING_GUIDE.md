# Marketplace Token Trading Guide

## Overview

The dchat marketplace allows users to:
1. **Sell digital goods** (sticker packs, themes, bots, NFTs) for tokens
2. **Buy digital goods** using tokens from their wallet
3. **Trade NFTs** peer-to-peer
4. **Create listings** with various pricing models
5. **Escrow protection** for secure transactions

---

## How Token Trading Works

### Architecture

```
┌─────────────┐         ┌──────────────┐         ┌─────────────────┐
│   Seller    │ creates │  Marketplace │ lists   │ Liquidity Pool  │
│   (Creator) │────────>│   Listing    │────────>│ (Token Reserve) │
└─────────────┘         └──────────────┘         └─────────────────┘
                               │                           │
                               │ purchase                  │
                               ▼                           ▼
                        ┌──────────────┐         ┌─────────────────┐
                        │    Escrow    │◄────────│  Token Transfer │
                        │   (Safety)   │         │  (1% Fee Burn)  │
                        └──────────────┘         └─────────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
            ┌──────────────┐      ┌──────────────┐
            │ Buyer Gets   │      │ Seller Gets  │
            │ Digital Good │      │    Tokens    │
            └──────────────┘      └──────────────┘
```

---

## Selling Digital Goods (Earning Tokens)

### Step 1: Create a Listing

**CLI Command**:
```bash
dchat marketplace create-listing \
  --title "Awesome Sticker Pack" \
  --description "50+ animated stickers for chat" \
  --type sticker-pack \
  --price 1000 \
  --content-hash "QmStickerPackIPFSHash"
```

**Rust API**:
```rust
use dchat::marketplace::{MarketplaceManager, DigitalGoodType, PricingModel};

let mut marketplace = MarketplaceManager::new();

// Create a listing
let listing_id = marketplace.create_listing(
    creator_id,                              // Your user ID
    "Awesome Sticker Pack".to_string(),     // Title
    "50+ animated stickers".to_string(),    // Description
    DigitalGoodType::StickerPack,           // Type
    PricingModel::OneTime { price: 1000 },  // 1000 tokens
    "QmStickerPackIPFSHash".to_string(),    // IPFS content hash
    None,                                    // No NFT
)?;

println!("Listing created: {}", listing_id);
```

### Pricing Models

**1. One-Time Purchase**:
```rust
PricingModel::OneTime { price: 1000 }  // 1000 tokens
```

**2. Subscription**:
```rust
PricingModel::Subscription { price_per_month: 500 }  // 500 tokens/month
```

**3. Free (Tips Accepted)**:
```rust
PricingModel::Free
```

**4. Pay-What-You-Want**:
```rust
PricingModel::PayWhatYouWant { minimum: 100 }  // Min 100 tokens
```

### Step 2: List Your Item

Once created, your listing appears in the marketplace:
- Searchable by type (sticker-pack, theme, bot, nft)
- Discoverable by users
- Tracked for downloads and ratings

### Step 3: Receive Payment

When a buyer purchases your item:
1. **Escrow Created**: Tokens locked in escrow (safety)
2. **Delivery**: Buyer downloads your content
3. **Release**: Buyer confirms → tokens released to you
4. **Fee**: 1% transaction fee automatically burned (deflationary)

**Example**:
- Sale price: 1000 tokens
- Transaction fee: 10 tokens (burned)
- You receive: 990 tokens

---

## Buying Digital Goods (Spending Tokens)

### Step 1: Browse Listings

**CLI Command**:
```bash
# List all sticker packs
dchat marketplace list-by-type --type sticker-pack

# Search by creator
dchat marketplace list-by-creator --creator-id <uuid>
```

**Rust API**:
```rust
// Get all sticker packs
let sticker_packs = marketplace.get_listings_by_type(&DigitalGoodType::StickerPack);

for listing in sticker_packs {
    println!("Title: {}", listing.title);
    println!("Price: {:?}", listing.pricing);
    println!("Rating: {:.1}/5.0", listing.rating);
    println!("Downloads: {}", listing.downloads);
    println!("---");
}
```

### Step 2: Check Your Balance

```bash
dchat token balance --user-id <your_uuid>
```

**Output**:
```
💰 Wallet Balance
============================================================
User ID: 550e8400-e29b-41d4-a716-446655440000
Balance:                 10,000 tokens
Staked:                   2,000 tokens
Pending Rewards:            500 tokens
Total Assets:            12,500 tokens
```

### Step 3: Purchase the Item

**CLI Command**:
```bash
dchat marketplace purchase \
  --listing-id <listing_uuid> \
  --amount 1000
```

**Rust API**:
```rust
use dchat::blockchain::CurrencyChainClient;

// Step 1: Get listing details
let listing = marketplace.get_listing(listing_id)
    .ok_or_else(|| Error::validation("Listing not found"))?;

// Step 2: Verify price
let price = match listing.pricing {
    PricingModel::OneTime { price } => price,
    _ => return Err(Error::validation("Unsupported pricing model")),
};

// Step 3: Transfer tokens (with 1% burn)
let tx_hash = currency_client.transfer(&buyer_id, &seller_id, price)?;

// Step 4: Create purchase record
let purchase_id = marketplace.purchase(
    buyer_id,
    listing_id,
    price,
    tx_hash.to_string(),
)?;

println!("Purchase complete! ID: {}", purchase_id);
```

### Step 4: Download Your Content

After purchase, you can download the digital good:
```bash
dchat marketplace download --purchase-id <purchase_uuid>
```

The content is retrieved from IPFS using the `content_hash` from the listing.

---

## Escrow Protection

All purchases are protected by an escrow system:

### How Escrow Works

**1. Funds Locked**:
```rust
// Create escrow when purchase happens
let escrow_id = marketplace.escrow.create_two_party_escrow(
    listing_id,
    &buyer_id,
    &seller_id,
    amount,
    86400,  // 24 hours lock
)?;
```

**2. Seller Delivers**:
```rust
// Seller marks as complete
marketplace.escrow.mark_awaiting_release(escrow_id)?;
```

**3. Buyer Confirms**:
```bash
# Release funds to seller
dchat marketplace release-escrow --escrow-id <uuid>
```

```rust
// Release funds
marketplace.escrow.release_funds(escrow_id, &buyer_id)?;
```

**4. Dispute Resolution** (if needed):
```bash
# Buyer raises dispute
dchat marketplace dispute \
  --escrow-id <uuid> \
  --reason item-not-received \
  --description "Never got the stickers"
```

```rust
// Raise dispute
marketplace.escrow.raise_dispute(
    escrow_id,
    &buyer_id,
    DisputeReason::ItemNotReceived,
    "Never received the content".to_string(),
)?;

// Admin resolves (full refund, partial refund, or release)
marketplace.escrow.resolve_dispute(
    escrow_id,
    DisputeResolution::RefundFull,  // or PartialRefund(500) or ReleaseFunds
)?;
```

### Escrow States

```
Locked → AwaitingRelease → Released ✅
   ↓                          
Disputed → Resolved → Refunded 🔄 or Released ✅
   ↓
Expired → Auto-Refunded 🔄
```

---

## NFT Trading

### Creating an NFT

**CLI Command**:
```bash
dchat marketplace create-nft \
  --token-id "rare_badge_001" \
  --name "Legendary Founder Badge" \
  --description "Limited edition founder NFT" \
  --image-hash "QmNFTImageHash" \
  --trait-type "Rarity" \
  --trait-value "Legendary"
```

**Rust API**:
```rust
use dchat::marketplace::NftAttribute;

marketplace.register_nft(
    "rare_badge_001".to_string(),
    "Legendary Founder Badge".to_string(),
    "Limited edition".to_string(),
    "QmNFTImageHash".to_string(),
    vec![
        NftAttribute {
            trait_type: "Rarity".to_string(),
            value: "Legendary".to_string(),
        },
        NftAttribute {
            trait_type: "Edition".to_string(),
            value: "1/100".to_string(),
        },
    ],
    creator_id,
    owner_id,  // Initially owned by creator
)?;
```

### Listing NFT for Sale

```rust
// Create listing for NFT
let listing_id = marketplace.create_listing(
    seller_id,
    "Legendary Founder Badge".to_string(),
    "Rare collectible NFT".to_string(),
    DigitalGoodType::Nft,
    PricingModel::OneTime { price: 50000 },  // 50,000 tokens
    "QmNFTImageHash".to_string(),
    Some("rare_badge_001".to_string()),  // NFT token ID
)?;
```

### Buying an NFT

```rust
// Purchase NFT listing
let purchase_id = marketplace.purchase(
    buyer_id,
    listing_id,
    50000,
    tx_hash.to_string(),
)?;

// Transfer NFT ownership
marketplace.transfer_nft("rare_badge_001", buyer_id)?;
```

### Viewing Your NFTs

**CLI Command**:
```bash
dchat marketplace my-nfts --user-id <uuid>
```

**Rust API**:
```rust
let my_nfts = marketplace.get_nfts_by_owner(&user_id);

for nft in my_nfts {
    println!("Token ID: {}", nft.token_id);
    println!("Name: {}", nft.name);
    println!("Creator: {}", nft.creator.0);
    println!("Attributes:");
    for attr in &nft.attributes {
        println!("  - {}: {}", attr.trait_type, attr.value);
    }
}
```

---

## Multi-Party Revenue Split

For collaborative creations, revenue can be split automatically:

```rust
// Create multi-party escrow (70% creator, 20% platform, 10% affiliate)
let escrow_id = marketplace.escrow.create_multi_party_escrow(
    listing_id,
    &buyer_id,
    vec![
        (creator_id, 700),    // 70%
        (platform_id, 200),   // 20%
        (affiliate_id, 100),  // 10%
    ],
    86400,  // 24 hour lock
)?;
```

When buyer releases escrow:
- Creator receives: 700 tokens
- Platform receives: 200 tokens
- Affiliate receives: 100 tokens
- Transaction fee: 10 tokens burned (1%)

---

## Tokenomics Integration

### Liquidity Pools for Marketplace

Liquidity pools provide token reserves for marketplace operations:

**Create Pool**:
```bash
dchat token create-pool \
  --name "Marketplace - Digital Goods" \
  --initial-amount 100000000  # 100M tokens
```

**Pool Usage Flow**:
```rust
// When user lists item for sale
let pool_id = get_pool_for_category(&listing.good_type);

// Allocate tokens from pool (reserve for sale)
let allocation_id = tokenomics.allocate_from_pool(pool_id, listing_price)?;

// When purchase completes
tokenomics.release_allocation(pool_id, allocation_id)?;

// Tokens flow: Pool → Escrow → Seller (minus 1% burn)
```

**Replenishment**:
```bash
# Manual
dchat token replenish-pool --pool-id <uuid> --amount 50000000

# Automated (distribution schedule)
dchat token create-schedule \
  --recipient-type marketplace \
  --amount 10000000 \
  --interval-blocks 1000
```

### Transaction Fee Economics

Every purchase burns 1% of tokens:
- **Sale**: 1000 tokens
- **Burned**: 10 tokens (deflationary)
- **Seller receives**: 990 tokens

This creates **deflationary pressure** as marketplace activity increases.

---

## Creator Economy

### Earning Tokens as Creator

**1. Create Quality Goods**:
- High-quality sticker packs
- Beautiful themes
- Useful bots
- Rare NFTs

**2. Build Reputation**:
```bash
# Check your stats
dchat marketplace creator-stats --creator-id <your_uuid>
```

**Output**:
```
📊 Creator Statistics
============================================================
Creator ID: 550e8400-e29b-41d4-a716-446655440000
Total Sales: 150
Total Earnings: 150,000 tokens
Active Listings: 12
Total Downloads: 500
Average Rating: 4.7/5.0
```

**3. Verified Badge**:
```bash
# Get verified (after meeting criteria)
dchat marketplace verify-creator --creator-id <uuid>
```

Verified creators:
- Get "✓" badge on listings
- Higher visibility in search
- Increased buyer trust

**4. Pricing Strategy**:
- Start with **Free** or **Pay-What-You-Want** to build audience
- Add **One-Time** pricing once established
- Offer **Subscription** for premium content packs
- Create **Limited Edition NFTs** for collectors

---

## Complete Purchase Flow Example

### Scenario: Buyer purchases sticker pack

**1. Seller Creates Listing**:
```rust
let listing_id = marketplace.create_listing(
    seller_id,
    "Cute Animals Pack".to_string(),
    "30 adorable animal stickers".to_string(),
    DigitalGoodType::StickerPack,
    PricingModel::OneTime { price: 500 },
    "QmStickerHash".to_string(),
    None,
)?;
```

**2. Buyer Browses and Finds Listing**:
```rust
let sticker_packs = marketplace.get_listings_by_type(&DigitalGoodType::StickerPack);
let chosen = sticker_packs.iter().find(|l| l.id == listing_id).unwrap();
```

**3. Buyer Checks Balance**:
```rust
let balance = currency_client.get_balance(&buyer_id)?;
assert!(balance >= 500, "Insufficient balance");
```

**4. Create Escrow**:
```rust
let escrow_id = marketplace.escrow.create_two_party_escrow(
    listing_id,
    &buyer_id,
    &seller_id,
    500,
    86400,  // 24 hours
)?;
```

**5. Transfer Tokens**:
```rust
// 500 tokens transferred, 5 tokens burned (1%)
let tx_hash = currency_client.transfer(&buyer_id, &escrow_wallet, 500)?;
```

**6. Record Purchase**:
```rust
let purchase_id = marketplace.purchase(
    buyer_id,
    listing_id,
    500,
    tx_hash.to_string(),
)?;
```

**7. Seller Delivers Content**:
```rust
// Upload to IPFS, mark as delivered
marketplace.escrow.mark_awaiting_release(escrow_id)?;
```

**8. Buyer Downloads and Confirms**:
```rust
// Download from IPFS
let content = ipfs_client.get(chosen.content_hash)?;

// Release funds
marketplace.escrow.release_funds(escrow_id, &buyer_id)?;
```

**9. Final Balances**:
```
Buyer: -500 tokens
Seller: +495 tokens (500 - 5 burned)
Burned: +5 tokens (deflationary)
```

---

## Marketplace Commands (CLI Reference)

### Listings
```bash
# Create listing
dchat marketplace create-listing --title "..." --price 1000

# List by type
dchat marketplace list-by-type --type sticker-pack

# List by creator
dchat marketplace list-by-creator --creator-id <uuid>

# Get listing details
dchat marketplace get-listing --listing-id <uuid>

# Verify listing
dchat marketplace verify-listing --listing-id <uuid>
```

### Purchases
```bash
# Purchase item
dchat marketplace purchase --listing-id <uuid> --amount 1000

# View purchase history
dchat marketplace my-purchases --user-id <uuid>

# Download purchased content
dchat marketplace download --purchase-id <uuid>
```

### Escrow
```bash
# Release escrow (buyer confirms)
dchat marketplace release-escrow --escrow-id <uuid>

# Raise dispute
dchat marketplace dispute --escrow-id <uuid> --reason item-not-received

# Resolve dispute (admin)
dchat marketplace resolve-dispute --escrow-id <uuid> --resolution refund-full

# Check escrow status
dchat marketplace escrow-status --escrow-id <uuid>
```

### NFTs
```bash
# Create NFT
dchat marketplace create-nft --token-id "nft_001" --name "Badge"

# Transfer NFT
dchat marketplace transfer-nft --token-id "nft_001" --new-owner <uuid>

# View my NFTs
dchat marketplace my-nfts --user-id <uuid>

# Get NFT details
dchat marketplace get-nft --token-id "nft_001"
```

### Creator Stats
```bash
# View creator statistics
dchat marketplace creator-stats --creator-id <uuid>

# Update listing rating
dchat marketplace rate-listing --listing-id <uuid> --rating 4.5
```

---

## Best Practices

### For Sellers
1. **Price Competitively**: Research similar items
2. **Quality Content**: High-quality stickers, themes, bots
3. **Clear Descriptions**: Detailed item descriptions
4. **Fast Delivery**: Upload to IPFS immediately
5. **Build Reputation**: Respond to buyers, maintain high ratings
6. **Diversify**: Offer multiple items at different price points

### For Buyers
1. **Check Ratings**: Look for 4+ star ratings
2. **Verified Sellers**: Prefer verified creators
3. **Read Descriptions**: Understand what you're buying
4. **Use Escrow**: Don't release funds until satisfied
5. **Report Issues**: Raise disputes for legitimate problems
6. **Leave Ratings**: Help other buyers

### For Platform
1. **Monitor Disputes**: Quick resolution builds trust
2. **Verify Creators**: Maintain quality standards
3. **Replenish Pools**: Keep liquidity high
4. **Burn Tracking**: Monitor deflationary effects
5. **Creator Support**: Help sellers succeed

---

## Economics Summary

### Token Flow
```
Users → Buy Tokens → Marketplace Purchases → Sellers
                ↓
        1% Burned (Deflationary)
                ↓
       Reduced Circulating Supply
```

### Liquidity Management
- **Pools**: Reserve tokens for marketplace operations
- **Allocation**: Reserve for pending sales
- **Release**: Complete transactions
- **Replenishment**: Automated from inflation

### Incentives
- **Sellers**: Earn tokens from sales
- **Buyers**: Access to exclusive content
- **Platform**: Transaction fees burned (reduces supply)
- **Network**: Deflation increases token value

---

## Troubleshooting

### "Insufficient Balance"
**Problem**: Not enough tokens to purchase
**Solution**: 
```bash
# Check balance
dchat token balance --user-id <uuid>

# Acquire tokens (receive from someone, earn as creator, etc.)
```

### "Escrow Expired"
**Problem**: Seller didn't deliver within 24 hours
**Solution**: Funds automatically refunded to buyer

### "Item Not Received"
**Problem**: Paid but didn't get content
**Solution**:
```bash
# Raise dispute
dchat marketplace dispute --escrow-id <uuid> --reason item-not-received
```

### "Can't Release Escrow"
**Problem**: Escrow still in "Locked" state
**Solution**: Wait for seller to mark as "AwaitingRelease"

---

## Future Enhancements

### Planned Features
1. **Auction System**: Bid on rare NFTs
2. **Bundle Deals**: Multi-item discounts
3. **Loyalty Rewards**: Frequent buyer bonuses
4. **Creator Tiers**: Platinum/Gold/Silver badges
5. **Referral System**: Earn tokens for referrals
6. **Rental Model**: Temporary access to content
7. **Fractional NFTs**: Shared ownership of expensive NFTs
8. **Automated Royalties**: Original creator gets % on resales

---

## Conclusion

The dchat marketplace provides a **complete token economy** for buying and selling digital goods:

✅ **Secure**: Escrow protection on all transactions
✅ **Fair**: 1% fee burned (no platform profit)
✅ **Decentralized**: Peer-to-peer trading
✅ **Creator-Friendly**: High earnings (99% after burn)
✅ **Buyer-Friendly**: Dispute resolution and refunds
✅ **Transparent**: All transactions on-chain
✅ **Deflationary**: Reduces supply with each trade

**Start trading today!**
```bash
# Sellers
dchat marketplace create-listing ...

# Buyers
dchat marketplace list-by-type --type sticker-pack
```

---

**Status**: ✅ Fully functional marketplace + tokenomics integration
**Documentation**: Complete trading guide
**Support**: See escrow.rs and lib.rs for implementation details

# How Users Buy and Sell Tokens in dchat Marketplace

## Quick Answer

Users can **buy** and **sell** tokens through the marketplace in three ways:

1. **Direct Token Transfer** - Send tokens between users
2. **Marketplace Purchase** - Buy digital goods using tokens
3. **NFT Trading** - Trade NFTs for tokens

---

## Method 1: Direct Token Transfer (Peer-to-Peer)

### Selling Tokens (User A → User B)

**User A wants to sell 1000 tokens to User B for $10 USD (off-chain)**

```bash
# User A transfers tokens to User B
dchat token transfer \
  --from <user_a_uuid> \
  --to <user_b_uuid> \
  --amount 1000
```

**Result**:
- User A: -1000 tokens
- User B: +990 tokens (1000 - 10 fee)
- Network: +10 tokens burned (1% fee)

**Payment**: External (PayPal, bank transfer, cash, etc.)

---

## Method 2: Marketplace Digital Goods Trading

### Selling Digital Goods for Tokens

**You are a creator selling sticker packs**

#### Step 1: Create Your Listing

```bash
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Cute Cat Stickers" \
  --description "50 adorable cat stickers" \
  --item-type sticker-pack \
  --price 500 \
  --content-hash "QmYourIPFSHash"
```

#### Step 2: Wait for Buyers

Your listing appears in marketplace:
```bash
# Buyers can find it with:
dchat marketplace list --item-type sticker-pack
```

#### Step 3: Buyer Purchases

**Buyer executes**:
```bash
dchat marketplace buy \
  --buyer-id <buyer_uuid> \
  --listing-id <your_listing_uuid>
```

**Behind the scenes**:
1. Buyer's tokens locked in escrow
2. Buyer downloads your content
3. Buyer confirms receipt
4. You receive tokens!

#### Step 4: Receive Payment

```
Sale Price: 500 tokens
Transaction Fee: 5 tokens (1% burned)
You Receive: 495 tokens ✅
```

### Buying Digital Goods with Tokens

**You want to buy a theme**

#### Step 1: Browse Listings

```bash
# Find themes for sale
dchat marketplace list --item-type theme
```

**Output**:
```
🏪 Marketplace Listings:
1. "Dark Mode Pro" - 300 tokens - ⭐4.8
2. "Neon Vibes" - 500 tokens - ⭐4.6
3. "Minimal Clean" - 200 tokens - ⭐4.9
```

#### Step 2: Check Your Balance

```bash
dchat token balance --user-id <your_uuid>
```

**Output**:
```
Balance: 1,000 tokens ✅ (Enough to buy!)
```

#### Step 3: Purchase

```bash
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <theme_listing_uuid>
```

#### Step 4: Download Content

After purchase, you can download:
```bash
dchat marketplace download --purchase-id <purchase_uuid>
```

**Cost**:
```
Theme Price: 300 tokens
Your New Balance: 700 tokens
```

---

## Method 3: NFT Trading

### Selling an NFT for Tokens

**You own "Legendary Founder Badge" NFT, want to sell for 10,000 tokens**

#### Step 1: Create NFT Listing

```bash
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Legendary Founder Badge - RARE!" \
  --description "Limited edition founder NFT - Only 100 exist" \
  --item-type nft \
  --price 10000 \
  --content-hash "QmNFTImageHash"
```

#### Step 2: Buyer Purchases

When someone buys:
1. Their 10,000 tokens locked in escrow
2. NFT ownership transferred to buyer
3. You receive 9,900 tokens (10,000 - 100 fee)

### Buying an NFT with Tokens

**You want to buy a rare NFT**

```bash
# Find NFTs for sale
dchat marketplace list --item-type nft

# Buy the NFT
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <nft_listing_uuid>
```

**Result**:
- You own the NFT ✅
- NFT shows in your collection
- Previous owner got tokens

---

## Marketplace CLI Commands Reference

### Creating Listings (Selling)

```bash
# Sell sticker pack
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Emoji Pack Vol 1" \
  --description "100 custom emojis" \
  --item-type sticker-pack \
  --price 250 \
  --content-hash "QmHash123"

# Sell theme
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Cyberpunk Theme" \
  --description "Futuristic UI theme" \
  --item-type theme \
  --price 400 \
  --content-hash "QmHash456"

# Sell bot
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Auto-Responder Bot" \
  --description "Smart reply automation" \
  --item-type bot \
  --price 1000 \
  --content-hash "QmHash789"

# Sell NFT
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Rare Achievement Badge" \
  --description "Special achievement NFT" \
  --item-type nft \
  --price 5000 \
  --content-hash "QmHashABC"

# Free item (accepts tips)
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Basic Stickers" \
  --description "Free starter pack" \
  --item-type sticker-pack \
  --price 0 \
  --content-hash "QmHashDEF"
```

### Browsing Listings (Buying)

```bash
# List all sticker packs
dchat marketplace list --item-type sticker-pack

# List all themes
dchat marketplace list --item-type theme

# List all bots
dchat marketplace list --item-type bot

# List all NFTs
dchat marketplace list --item-type nft

# List all subscriptions
dchat marketplace list --item-type subscription

# List all badges
dchat marketplace list --item-type badge

# List everything (no filter)
dchat marketplace list
```

### Purchasing (Buying)

```bash
# Buy any item
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <item_listing_uuid>

# The system automatically:
# 1. Checks your balance
# 2. Creates escrow
# 3. Transfers tokens (with 1% burn)
# 4. Gives you access to content
```

### Checking Stats (Sellers)

```bash
# View your creator statistics
dchat marketplace creator-stats --creator-id <your_uuid>
```

**Output Example**:
```
📊 Creator Statistics:
Creator: 550e8400-e29b-41d4-a716-446655440000
Total Sales: 42
Total Earnings: 21,000 tokens
Active Listings: 8
Total Downloads: 156
Average Rating: 4.7⭐
```

---

## Escrow System (Transaction Safety)

### Why Escrow?

Protects both buyer and seller:
- **Buyer**: Don't pay until you get the content
- **Seller**: Get paid when buyer confirms receipt

### How Escrow Works

```
1. Buyer initiates purchase
   ↓
2. Tokens locked in escrow (safe)
   ↓
3. Seller delivers digital content
   ↓
4. Buyer confirms receipt
   ↓
5. Tokens released to seller ✅
```

### Escrow Commands

```bash
# Create escrow manually (advanced)
dchat marketplace create-escrow \
  --buyer <buyer_uuid> \
  --seller <seller_uuid> \
  --amount 1000

# Release escrow (buyer confirms)
dchat marketplace release-escrow --escrow-id <escrow_uuid>

# Dispute (if problem occurs)
dchat marketplace dispute \
  --escrow-id <escrow_uuid> \
  --reason item-not-received \
  --description "Content never delivered"

# Check escrow status
dchat marketplace escrow-status --escrow-id <escrow_uuid>
```

### Escrow Protection Period

- **Default**: 30 days
- **Auto-refund**: If seller doesn't deliver within 30 days
- **Dispute resolution**: Admin can intervene

---

## Real-World Scenarios

### Scenario 1: Beginner Creator

**You create your first sticker pack**

```bash
# 1. Create listing (free to attract users)
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "My First Sticker Pack" \
  --description "10 hand-drawn stickers" \
  --item-type sticker-pack \
  --price 0 \
  --content-hash "QmMyFirstPack"

# 2. Users download (build reputation)
# 3. Get good ratings
# 4. Create premium pack with price:

dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "Premium Sticker Pack Vol 2" \
  --description "50 professional stickers" \
  --item-type sticker-pack \
  --price 500 \
  --content-hash "QmPremiumPack"

# 5. Earn tokens! 🎉
```

### Scenario 2: Power User Buyer

**You want to customize dchat with themes and bots**

```bash
# 1. Check balance
dchat token balance --user-id <your_uuid>
# Output: Balance: 5,000 tokens

# 2. Buy dark theme (300 tokens)
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <dark_theme_uuid>

# 3. Buy automation bot (1,000 tokens)
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <bot_uuid>

# 4. Buy sticker pack (200 tokens)
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <stickers_uuid>

# Total spent: 1,500 tokens
# Remaining: 3,500 tokens
```

### Scenario 3: NFT Collector

**You collect rare NFTs**

```bash
# 1. Browse NFTs
dchat marketplace list --item-type nft

# 2. Found rare "OG Member" badge - 20,000 tokens
dchat marketplace buy \
  --buyer-id <your_uuid> \
  --listing-id <og_badge_uuid>

# 3. Own the NFT!
dchat marketplace my-nfts --user-id <your_uuid>

# 4. Later sell for profit (30,000 tokens)
dchat marketplace create-listing \
  --creator-id <your_uuid> \
  --title "OG Member Badge #42" \
  --description "Early adopter NFT - Rare!" \
  --item-type nft \
  --price 30000 \
  --content-hash "QmOGBadge"

# Profit: 10,000 tokens! 📈
```

### Scenario 4: Professional Creator

**You're a full-time digital artist**

```bash
# Create portfolio of listings
# 1. Free starter pack (marketing)
dchat marketplace create-listing \
  --title "Artist Sample Pack" \
  --price 0 ...

# 2. Budget tier (200 tokens)
dchat marketplace create-listing \
  --title "Basic Sticker Set" \
  --price 200 ...

# 3. Premium tier (800 tokens)
dchat marketplace create-listing \
  --title "Professional Theme Bundle" \
  --price 800 ...

# 4. Exclusive tier (3,000 tokens)
dchat marketplace create-listing \
  --title "Custom Bot + Premium Support" \
  --price 3000 ...

# 5. Check earnings monthly
dchat marketplace creator-stats --creator-id <your_uuid>

# Example earnings:
# Month 1: 50,000 tokens
# Month 2: 75,000 tokens
# Month 3: 100,000 tokens
```

---

## Token Economics for Marketplace

### Transaction Fees

**Every purchase burns 1% of tokens**:
```
Sale: 1,000 tokens
Burned: 10 tokens (1%)
Seller receives: 990 tokens
```

**Why burn tokens?**
- Reduces circulating supply
- Makes remaining tokens more valuable
- Rewards long-term holders

### Price Discovery

**Sellers set prices based on**:
- Content quality
- Rarity (NFTs)
- Market demand
- Competition

**Example price ranges**:
- Basic sticker pack: 100-500 tokens
- Premium theme: 300-1,000 tokens
- Automation bot: 500-2,000 tokens
- Rare NFT: 1,000-100,000 tokens

### Earning Potential

**Monthly earnings for creators**:
- **Hobbyist**: 1,000-10,000 tokens/month
- **Semi-pro**: 10,000-50,000 tokens/month
- **Professional**: 50,000-200,000 tokens/month
- **Top 1%**: 200,000+ tokens/month

---

## Getting Started Checklist

### For Sellers (Creators)

- [ ] Create high-quality content (stickers, themes, bots)
- [ ] Upload content to IPFS
- [ ] Create marketplace listing
- [ ] Set competitive price
- [ ] Promote your listings
- [ ] Build reputation (ratings)
- [ ] Earn tokens! 💰

### For Buyers (Users)

- [ ] Get tokens (earn, receive, or buy from others)
- [ ] Check balance
- [ ] Browse marketplace listings
- [ ] Purchase desired items
- [ ] Download content
- [ ] Leave ratings (help others)
- [ ] Enjoy your purchases! 🎉

---

## FAQ

### Q: How do I get my first tokens?
**A**: 
1. Receive tokens from friend (transfer)
2. Create and sell free content (build audience)
3. Participate in network activities (rewards)
4. Buy from someone outside marketplace

### Q: What's the minimum price I can charge?
**A**: 0 tokens (free with optional tips)

### Q: Can I change my listing price after creating it?
**A**: Currently no, but this feature is planned

### Q: What happens if buyer doesn't confirm receipt?
**A**: After 30 days, escrow auto-releases to seller

### Q: Can I get a refund if I'm unhappy?
**A**: Yes, raise a dispute within the escrow period

### Q: Are there listing fees?
**A**: No listing fees! Only 1% transaction fee on sales

### Q: Can I sell physical goods?
**A**: Marketplace is designed for digital goods only

### Q: How do I cash out tokens to USD?
**A**: Peer-to-peer exchange (outside the dchat system)

### Q: Can I buy tokens directly in the app?
**A**: Currently no, tokens are earned or transferred peer-to-peer

### Q: What file types can I sell?
**A**: Any digital content (images, videos, scripts, etc.)
Content is stored on IPFS

---

## Advanced: Revenue Splits

**For collaborative projects**:

```bash
# Create multi-party escrow (70% creator, 20% platform, 10% affiliate)
dchat marketplace create-multi-party-escrow \
  --buyer <buyer_uuid> \
  --recipients <creator_uuid>:700,<platform_uuid>:200,<affiliate_uuid>:100 \
  --listing-id <listing_uuid>
```

**When sale completes**:
- Creator: 700 tokens
- Platform: 200 tokens
- Affiliate: 100 tokens
- Burned: 10 tokens (1% fee)

---

## Summary

### Buying Tokens = Buying Digital Goods

You don't "buy tokens from marketplace" directly. Instead:

1. **Get tokens** (transfer, earn, or external exchange)
2. **Spend tokens** on digital goods in marketplace
3. **Sellers receive tokens** (minus 1% burn)

### Selling Tokens = Selling Digital Goods

You don't "sell tokens to marketplace" directly. Instead:

1. **Create digital content** (stickers, themes, bots, NFTs)
2. **List in marketplace** with price in tokens
3. **Buyers purchase** using their tokens
4. **You receive tokens** (98.5-99% after fees)

### Complete Flow

```
User A (Seller) → Creates Listing → 500 tokens
                        ↓
                   Marketplace
                        ↓
User B (Buyer) → Purchases → Pays 500 tokens
                        ↓
                    Escrow (Safety)
                        ↓
        User A receives 495 tokens ✅
        Network burns 5 tokens 🔥
        User B gets digital good ✅
```

---

**Ready to start trading?**

```bash
# Sellers
dchat marketplace create-listing --help

# Buyers
dchat marketplace list --help
dchat marketplace buy --help
```

**Status**: ✅ Fully functional marketplace
**Documentation**: Complete trading guide
**Support**: See MARKETPLACE_TOKEN_TRADING_GUIDE.md for full details

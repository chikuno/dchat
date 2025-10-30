# dchat Python SDK

Official Python SDK for dchat - a decentralized chat application with blockchain-enforced message ordering and end-to-end encryption.

## Features

- ✅ **Blockchain Integration**: All operations create on-chain transactions
- ✅ **User Management**: Create users with Ed25519 keypairs
- ✅ **Direct Messaging**: Send encrypted direct messages with SHA-256 content hashing
- ✅ **Channel Operations**: Create and post to channels
- ✅ **Transaction Confirmation**: Wait for blockchain confirmations with configurable blocks
- ✅ **Async/Await Support**: Full async support using asyncio
- ✅ **Type Hints**: Complete type annotations for better IDE support

## Installation

```bash
pip install dchat-sdk
```

Or from source:

```bash
cd sdk/python
pip install -e .
```

## Quick Start

```python
import asyncio
from dchat import BlockchainClient, UserManager

async def main():
    # Initialize blockchain client
    blockchain = BlockchainClient.local()
    
    # Create user manager
    user_manager = UserManager(
        blockchain=blockchain,
        base_url="http://localhost:8080"
    )
    
    # Create user
    user = await user_manager.create_user("alice")
    print(f"User created: {user.user_id}")
    print(f"On-chain confirmed: {user.on_chain_confirmed}")
    print(f"Transaction ID: {user.tx_id}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Documentation

### BlockchainClient

```python
from dchat import BlockchainClient, BlockchainConfig

# Local development
blockchain = BlockchainClient.local()

# Custom configuration
config = BlockchainConfig(
    rpc_url="https://rpc.dchat.io",
    ws_url="wss://ws.dchat.io",
    confirmation_blocks=6,
    confirmation_timeout=300,
    max_retries=3
)
blockchain = BlockchainClient(config)

# Register user
tx_id = await blockchain.register_user(
    user_id="user-id",
    username="alice",
    public_key="ed25519-public-key-hex"
)

# Wait for confirmation
receipt = await blockchain.wait_for_confirmation(tx_id)
print(f"Confirmed: {receipt.success}")
```

### UserManager

```python
from dchat import UserManager

manager = UserManager(
    blockchain=blockchain,
    base_url="http://localhost:8080"
)

# Create user
user = await manager.create_user("alice")

# Send direct message
message = await manager.send_direct_message(
    sender_id=user.user_id,
    recipient_id="recipient-id",
    content="Hello!",
    relay_node_id="relay-1"  # Optional
)

# Create channel
channel = await manager.create_channel(
    creator_id=user.user_id,
    channel_name="General",
    description="General discussion"
)

# Post to channel
post = await manager.post_to_channel(
    sender_id=user.user_id,
    channel_id=channel.channel_id,
    content="Welcome!"
)
```

### Cryptographic Utilities

```python
from dchat.crypto import KeyPair, hash_content

# Generate key pair
keypair = KeyPair.generate()
print(f"Public key: {keypair.public_key_hex}")

# Sign message
signature = keypair.sign(b"message")

# Verify signature
is_valid = keypair.verify(b"message", signature)

# Hash content
content_hash = hash_content("message content")
```

## Requirements

- Python 3.8+
- aiohttp
- cryptography
- ed25519

## Examples

See [examples/](examples/) directory for complete examples.

## Development

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run type checking
mypy dchat

# Format code
black dchat tests
```

## License

See [LICENSE](../../LICENSE) in the main repository.

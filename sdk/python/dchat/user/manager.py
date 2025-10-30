"""
User management for creating users and handling profiles
"""

import uuid
from datetime import datetime, timezone

from ..blockchain.client import BlockchainClient
from ..crypto.keypair import KeyPair, hash_content
from .models import (
    CreateUserResponse,
    DirectMessageResponse,
    CreateChannelResponse,
)


class UserManager:
    """User manager for user operations"""

    def __init__(self, blockchain: BlockchainClient, base_url: str):
        self.blockchain = blockchain
        self.base_url = base_url

    async def create_user(self, username: str) -> CreateUserResponse:
        """Create a new user with blockchain registration"""
        # Generate unique user ID
        user_id = str(uuid.uuid4())

        # Generate Ed25519 key pair
        keypair = KeyPair.generate()

        # Submit blockchain transaction
        tx_id = await self.blockchain.register_user(
            user_id=user_id,
            username=username,
            public_key=keypair.public_key_hex,
        )

        # Wait for blockchain confirmation
        receipt = await self.blockchain.wait_for_confirmation(tx_id)
        on_chain_confirmed = receipt.success

        # Return response with actual blockchain status
        return CreateUserResponse(
            user_id=user_id,
            username=username,
            public_key=keypair.public_key_hex,
            private_key=keypair.private_key_hex,
            created_at=datetime.now(timezone.utc).isoformat(),
            on_chain_confirmed=on_chain_confirmed,
            tx_id=tx_id,
        )

    async def send_direct_message(
        self,
        sender_id: str,
        recipient_id: str,
        content: str,
        relay_node_id: str = None,
    ) -> DirectMessageResponse:
        """Send a direct message"""
        # Generate message ID
        message_id = str(uuid.uuid4())

        # Hash the content
        content_hash = hash_content(content)

        # Submit blockchain transaction
        tx_id = await self.blockchain.send_direct_message(
            message_id=message_id,
            sender_id=sender_id,
            recipient_id=recipient_id,
            content_hash=content_hash,
            payload_size=len(content),
            relay_node_id=relay_node_id,
        )

        # Wait for confirmation
        receipt = await self.blockchain.wait_for_confirmation(tx_id)
        on_chain_confirmed = receipt.success

        return DirectMessageResponse(
            message_id=message_id,
            sender_id=sender_id,
            recipient_id=recipient_id,
            content_hash=content_hash,
            created_at=datetime.now(timezone.utc).isoformat(),
            on_chain_confirmed=on_chain_confirmed,
            tx_id=tx_id,
        )

    async def create_channel(
        self,
        creator_id: str,
        channel_name: str,
        description: str = None,
    ) -> CreateChannelResponse:
        """Create a new channel"""
        # Generate channel ID
        channel_id = str(uuid.uuid4())

        # Submit blockchain transaction
        tx_id = await self.blockchain.create_channel(
            channel_id=channel_id,
            name=channel_name,
            description=description or "",
            creator_id=creator_id,
        )

        # Wait for confirmation
        receipt = await self.blockchain.wait_for_confirmation(tx_id)
        on_chain_confirmed = receipt.success

        return CreateChannelResponse(
            channel_id=channel_id,
            name=channel_name,
            description=description,
            creator_id=creator_id,
            created_at=datetime.now(timezone.utc).isoformat(),
            on_chain_confirmed=on_chain_confirmed,
            tx_id=tx_id,
        )

    async def post_to_channel(
        self,
        sender_id: str,
        channel_id: str,
        content: str,
    ) -> DirectMessageResponse:
        """Post a message to a channel"""
        # Generate message ID
        message_id = str(uuid.uuid4())

        # Hash the content
        content_hash = hash_content(content)

        # Submit blockchain transaction
        tx_id = await self.blockchain.post_to_channel(
            message_id=message_id,
            channel_id=channel_id,
            sender_id=sender_id,
            content_hash=content_hash,
            payload_size=len(content),
        )

        # Wait for confirmation
        receipt = await self.blockchain.wait_for_confirmation(tx_id)
        on_chain_confirmed = receipt.success

        return DirectMessageResponse(
            message_id=message_id,
            sender_id=sender_id,
            recipient_id=channel_id,  # Using channel_id as recipient
            content_hash=content_hash,
            created_at=datetime.now(timezone.utc).isoformat(),
            on_chain_confirmed=on_chain_confirmed,
            tx_id=tx_id,
        )

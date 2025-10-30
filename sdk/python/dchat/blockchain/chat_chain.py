"""Chat Chain client for Python SDK"""

import asyncio
import httpx
from enum import Enum
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
import time


class ChatChainTxType(Enum):
    """Chat chain transaction types"""
    REGISTER_USER = "register_user"
    SEND_DIRECT_MESSAGE = "send_direct_message"
    CREATE_CHANNEL = "create_channel"
    POST_TO_CHANNEL = "post_to_channel"
    UPDATE_REPUTATION = "update_reputation"
    GOVERNANCE = "governance"


@dataclass
class ChatChainTransaction:
    """Chat chain transaction"""
    id: str
    tx_type: str
    sender: str
    data: Dict[str, Any]
    status: str
    confirmations: int
    block_height: int
    created_at: int


class ChatChainClient:
    """Chat Chain client for identity and messaging operations"""

    def __init__(self, rpc_url: str = "http://localhost:8545"):
        self.rpc_url = rpc_url
        self.client = httpx.Client()

    async def register_user(self, user_id: str, public_key: bytes) -> ChatChainTransaction:
        """Register user identity on chat chain"""
        import base64
        response = await self.client.post(
            f"{self.rpc_url}/chat/register_user",
            json={
                "user_id": user_id,
                "public_key": base64.b64encode(public_key).decode(),
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        return ChatChainTransaction(**data)

    async def send_direct_message(
        self, sender: str, recipient: str, message_id: str
    ) -> ChatChainTransaction:
        """Send direct message transaction"""
        response = await self.client.post(
            f"{self.rpc_url}/chat/send_message",
            json={
                "sender": sender,
                "recipient": recipient,
                "message_id": message_id,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        return ChatChainTransaction(**data)

    async def create_channel(
        self, owner: str, channel_id: str, name: str
    ) -> ChatChainTransaction:
        """Create channel on chat chain"""
        response = await self.client.post(
            f"{self.rpc_url}/chat/create_channel",
            json={
                "owner": owner,
                "channel_id": channel_id,
                "name": name,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        return ChatChainTransaction(**data)

    async def post_to_channel(
        self, sender: str, channel_id: str, message_id: str
    ) -> ChatChainTransaction:
        """Post message to channel on chat chain"""
        response = await self.client.post(
            f"{self.rpc_url}/chat/post_message",
            json={
                "sender": sender,
                "channel_id": channel_id,
                "message_id": message_id,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        return ChatChainTransaction(**data)

    async def get_reputation(self, user_id: str) -> int:
        """Get user reputation"""
        response = await self.client.get(f"{self.rpc_url}/chat/reputation/{user_id}")
        response.raise_for_status()
        data = response.json()
        return data["reputation"]

    async def get_user_transactions(self, user_id: str) -> List[ChatChainTransaction]:
        """Get user transaction history"""
        response = await self.client.get(f"{self.rpc_url}/chat/transactions/{user_id}")
        response.raise_for_status()
        data = response.json()
        return [ChatChainTransaction(**item) for item in data]

    async def get_transaction(self, tx_id: str) -> Optional[ChatChainTransaction]:
        """Get transaction by ID"""
        try:
            response = await self.client.get(f"{self.rpc_url}/chat/transaction/{tx_id}")
            if response.status_code == 404:
                return None
            response.raise_for_status()
            data = response.json()
            return ChatChainTransaction(**data)
        except httpx.HTTPError:
            return None

    async def wait_for_confirmation(
        self, tx_id: str, confirmations: int = 6, max_wait_ms: int = 30000
    ) -> ChatChainTransaction:
        """Wait for transaction to be confirmed"""
        start_time = time.time()
        while (time.time() - start_time) * 1000 < max_wait_ms:
            tx = await self.get_transaction(tx_id)
            if tx and tx.confirmations >= confirmations:
                return tx
            await asyncio.sleep(1)
        raise TimeoutError(f"Transaction {tx_id} did not confirm within {max_wait_ms}ms")

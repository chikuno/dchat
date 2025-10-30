"""Cross-chain bridge for Python SDK"""

import httpx
from enum import Enum
from typing import Optional, List
from dataclasses import dataclass
import time
import asyncio
from .chat_chain import ChatChainClient
from .currency_chain import CurrencyChainClient


class CrossChainStatus(Enum):
    """Cross-chain transaction status"""
    PENDING = "pending"
    CHAT_CHAIN_CONFIRMED = "chat_chain_confirmed"
    CURRENCY_CHAIN_CONFIRMED = "currency_chain_confirmed"
    ATOMIC_SUCCESS = "atomic_success"
    ROLLED_BACK = "rolled_back"
    FAILED = "failed"


@dataclass
class CrossChainTransaction:
    """Cross-chain transaction"""
    id: str
    operation: str
    user_id: str
    chat_chain_tx: Optional[str] = None
    currency_chain_tx: Optional[str] = None
    status: str = "pending"
    created_at: int = 0
    finalized_at: Optional[int] = None


class CrossChainBridge:
    """Bridge for coordinating atomic transactions between chains"""

    def __init__(
        self,
        chat_chain: ChatChainClient,
        currency_chain: CurrencyChainClient,
        bridge_url: str = "http://localhost:8548",
    ):
        self.chat_chain = chat_chain
        self.currency_chain = currency_chain
        self.bridge_url = bridge_url
        self.client = httpx.Client()

    async def register_user_with_stake(
        self, user_id: str, public_key: bytes, stake_amount: int
    ) -> CrossChainTransaction:
        """Register user with initial stake (atomic operation)"""
        import base64
        response = await self.client.post(
            f"{self.bridge_url}/register_user_with_stake",
            json={
                "user_id": user_id,
                "public_key": base64.b64encode(public_key).decode(),
                "stake_amount": stake_amount,
            },
        )
        response.raise_for_status()
        data = response.json()
        return CrossChainTransaction(**data)

    async def create_channel_with_fee(
        self, owner: str, channel_name: str, creation_fee: int
    ) -> CrossChainTransaction:
        """Create channel with fee (atomic operation)"""
        response = await self.client.post(
            f"{self.bridge_url}/create_channel_with_fee",
            json={
                "owner": owner,
                "channel_name": channel_name,
                "creation_fee": creation_fee,
            },
        )
        response.raise_for_status()
        data = response.json()
        return CrossChainTransaction(**data)

    async def get_status(self, bridge_tx_id: str) -> Optional[CrossChainTransaction]:
        """Get cross-chain transaction status"""
        try:
            response = await self.client.get(f"{self.bridge_url}/status/{bridge_tx_id}")
            if response.status_code == 404:
                return None
            response.raise_for_status()
            data = response.json()
            return CrossChainTransaction(**data)
        except httpx.HTTPError:
            return None

    async def get_user_transactions(self, user_id: str) -> List[CrossChainTransaction]:
        """Get all cross-chain transactions for user"""
        response = await self.client.get(f"{self.bridge_url}/user_transactions/{user_id}")
        response.raise_for_status()
        data = response.json()
        return [CrossChainTransaction(**item) for item in data]

    async def wait_for_atomic_completion(
        self, bridge_tx_id: str, max_wait_ms: int = 60000
    ) -> CrossChainTransaction:
        """Wait for atomic cross-chain transaction to complete"""
        start_time = time.time()
        while (time.time() - start_time) * 1000 < max_wait_ms:
            tx = await self.get_status(bridge_tx_id)
            if tx and tx.status == CrossChainStatus.ATOMIC_SUCCESS.value:
                return tx
            if tx and tx.status in [CrossChainStatus.FAILED.value, CrossChainStatus.ROLLED_BACK.value]:
                raise RuntimeError(f"Cross-chain transaction {bridge_tx_id} failed with status: {tx.status}")
            await asyncio.sleep(1)
        raise TimeoutError(f"Cross-chain transaction {bridge_tx_id} did not complete within {max_wait_ms}ms")

"""
Blockchain client for transaction submission and confirmation
"""

import asyncio
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Optional, Dict

import aiohttp

from .transaction import (
    RegisterUserTx,
    SendDirectMessageTx,
    CreateChannelTx,
    PostToChannelTx,
    TransactionReceipt,
    ChannelVisibility,
)


@dataclass
class BlockchainConfig:
    """Configuration for blockchain client"""
    rpc_url: str
    ws_url: Optional[str] = None
    confirmation_blocks: int = 6
    confirmation_timeout: int = 300  # seconds
    max_retries: int = 3


class BlockchainClient:
    """Blockchain client for transaction submission and confirmation"""

    def __init__(self, config: BlockchainConfig):
        self.config = config
        self._transaction_cache: Dict[str, TransactionReceipt] = {}
        self._session: Optional[aiohttp.ClientSession] = None

    @classmethod
    def local(cls) -> "BlockchainClient":
        """Create a client configured for local development"""
        return cls(
            BlockchainConfig(
                rpc_url="http://localhost:8545",
                ws_url="ws://localhost:8546",
                confirmation_blocks=6,
                confirmation_timeout=300,
            )
        )

    async def __aenter__(self):
        self._session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session:
            await self._session.close()

    async def register_user(
        self, user_id: str, username: str, public_key: str
    ) -> str:
        """Register a new user on-chain"""
        tx = RegisterUserTx(
            user_id=user_id,
            username=username,
            public_key=public_key,
            timestamp=datetime.now(timezone.utc),
        )
        return await self._submit_transaction("register_user", tx.to_dict())

    async def send_direct_message(
        self,
        message_id: str,
        sender_id: str,
        recipient_id: str,
        content_hash: str,
        payload_size: int,
        relay_node_id: Optional[str] = None,
    ) -> str:
        """Send a direct message on-chain"""
        tx = SendDirectMessageTx(
            message_id=message_id,
            sender_id=sender_id,
            recipient_id=recipient_id,
            content_hash=content_hash,
            payload_size=payload_size,
            timestamp=datetime.now(timezone.utc),
            relay_node_id=relay_node_id,
        )
        return await self._submit_transaction("send_direct_message", tx.to_dict())

    async def create_channel(
        self,
        channel_id: str,
        name: str,
        description: str,
        creator_id: str,
        visibility: ChannelVisibility = ChannelVisibility.PUBLIC,
        token_requirement: Optional[str] = None,
    ) -> str:
        """Create a new channel on-chain"""
        tx = CreateChannelTx(
            channel_id=channel_id,
            name=name,
            description=description,
            creator_id=creator_id,
            visibility=visibility,
            timestamp=datetime.now(timezone.utc),
            token_requirement=token_requirement,
        )
        return await self._submit_transaction("create_channel", tx.to_dict())

    async def post_to_channel(
        self,
        message_id: str,
        channel_id: str,
        sender_id: str,
        content_hash: str,
        payload_size: int,
    ) -> str:
        """Post a message to a channel on-chain"""
        tx = PostToChannelTx(
            message_id=message_id,
            channel_id=channel_id,
            sender_id=sender_id,
            content_hash=content_hash,
            payload_size=payload_size,
            timestamp=datetime.now(timezone.utc),
        )
        return await self._submit_transaction("post_to_channel", tx.to_dict())

    async def wait_for_confirmation(self, tx_id: str) -> TransactionReceipt:
        """Wait for transaction confirmation"""
        # Check cache first
        if tx_id in self._transaction_cache:
            cached = self._transaction_cache[tx_id]
            if cached.success or cached.error:
                return cached

        deadline = asyncio.get_event_loop().time() + self.config.confirmation_timeout

        while asyncio.get_event_loop().time() < deadline:
            receipt = await self.get_transaction_receipt(tx_id)
            if receipt:
                self._transaction_cache[tx_id] = receipt

                if receipt.success:
                    return receipt
                elif receipt.error:
                    raise Exception(f"Transaction failed: {receipt.error}")

            # Poll every 2 seconds
            await asyncio.sleep(2)

        raise TimeoutError(
            f"Transaction confirmation timed out after {self.config.confirmation_timeout}s"
        )

    async def is_transaction_confirmed(self, tx_id: str) -> bool:
        """Check if a transaction is confirmed"""
        try:
            receipt = await self.get_transaction_receipt(tx_id)
            return receipt.success if receipt else False
        except Exception:
            return False

    async def get_transaction_receipt(
        self, tx_id: str
    ) -> Optional[TransactionReceipt]:
        """Get transaction receipt"""
        session = self._session or aiohttp.ClientSession()
        try:
            async with session.post(
                self.config.rpc_url,
                json={
                    "jsonrpc": "2.0",
                    "method": "eth_getTransactionReceipt",
                    "params": [tx_id],
                    "id": 1,
                },
            ) as response:
                if response.status != 200:
                    return None

                data = await response.json()
                if "result" in data and data["result"]:
                    result = data["result"]
                    return TransactionReceipt(
                        tx_id=result["tx_id"],
                        tx_hash=result["tx_hash"],
                        success=result["success"],
                        block_height=result.get("block_height"),
                        block_hash=result.get("block_hash"),
                        timestamp=datetime.fromisoformat(result["timestamp"])
                        if "timestamp" in result
                        else None,
                        error=result.get("error"),
                    )
                return None
        except Exception:
            return None
        finally:
            if not self._session:
                await session.close()

    async def get_block_number(self) -> int:
        """Get current block number"""
        session = self._session or aiohttp.ClientSession()
        try:
            async with session.post(
                self.config.rpc_url,
                json={
                    "jsonrpc": "2.0",
                    "method": "eth_blockNumber",
                    "params": [],
                    "id": 1,
                },
            ) as response:
                if response.status != 200:
                    raise Exception(f"Failed to get block number: {response.status}")

                data = await response.json()
                return int(data["result"], 16)
        finally:
            if not self._session:
                await session.close()

    async def _submit_transaction(self, method: str, params: dict) -> str:
        """Submit a transaction to the blockchain"""
        session = self._session or aiohttp.ClientSession()
        try:
            async with session.post(
                self.config.rpc_url,
                json={
                    "jsonrpc": "2.0",
                    "method": f"dchat_{method}",
                    "params": [params],
                    "id": 1,
                },
            ) as response:
                if response.status != 200:
                    raise Exception(f"Failed to submit transaction: {response.status}")

                data = await response.json()
                if "error" in data:
                    raise Exception(f"RPC error: {data['error']['message']}")

                return data["result"]["tx_id"]
        finally:
            if not self._session:
                await session.close()

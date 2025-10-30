"""Currency Chain client for Python SDK"""

import httpx
from enum import Enum
from typing import Optional, List, Dict, Any
from dataclasses import dataclass
import time
import asyncio


class CurrencyChainTxType(Enum):
    """Currency chain transaction types"""
    PAYMENT = "payment"
    STAKE = "stake"
    UNSTAKE = "unstake"
    REWARD = "reward"
    SLASH = "slash"
    SWAP = "swap"


@dataclass
class Wallet:
    """Wallet information"""
    user_id: str
    balance: int
    staked: int
    rewards_pending: int


@dataclass
class CurrencyChainTransaction:
    """Currency chain transaction"""
    id: str
    tx_type: str
    from_user: str
    to_user: Optional[str]
    amount: int
    status: str
    confirmations: int
    block_height: int
    created_at: int


class CurrencyChainClient:
    """Currency Chain client for economics operations"""

    def __init__(self, rpc_url: str = "http://localhost:8546"):
        self.rpc_url = rpc_url
        self.client = httpx.Client()

    async def create_wallet(self, user_id: str, initial_balance: int) -> Wallet:
        """Create wallet for user"""
        response = await self.client.post(
            f"{self.rpc_url}/currency/create_wallet",
            json={
                "user_id": user_id,
                "initial_balance": initial_balance,
            },
        )
        response.raise_for_status()
        data = response.json()
        wallet_data = data["wallet"]
        return Wallet(**wallet_data)

    async def get_wallet(self, user_id: str) -> Wallet:
        """Get wallet balance"""
        response = await self.client.get(f"{self.rpc_url}/currency/wallet/{user_id}")
        response.raise_for_status()
        data = response.json()
        return Wallet(**data)

    async def get_balance(self, user_id: str) -> int:
        """Get user balance"""
        wallet = await self.get_wallet(user_id)
        return wallet.balance

    async def transfer(self, from_user: str, to_user: str, amount: int) -> CurrencyChainTransaction:
        """Transfer tokens between users"""
        response = await self.client.post(
            f"{self.rpc_url}/currency/transfer",
            json={
                "from": from_user,
                "to": to_user,
                "amount": amount,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        data["from_user"] = data.pop("from")
        data["to_user"] = data.pop("to", None)
        return CurrencyChainTransaction(**data)

    async def stake(self, user_id: str, amount: int, lock_duration_seconds: int) -> CurrencyChainTransaction:
        """Stake tokens for rewards"""
        response = await self.client.post(
            f"{self.rpc_url}/currency/stake",
            json={
                "user_id": user_id,
                "amount": amount,
                "lock_duration_seconds": lock_duration_seconds,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        data["from_user"] = data.get("from", user_id)
        data["to_user"] = data.get("to", None)
        return CurrencyChainTransaction(**data)

    async def claim_rewards(self, user_id: str) -> CurrencyChainTransaction:
        """Claim rewards"""
        response = await self.client.post(
            f"{self.rpc_url}/currency/claim_rewards",
            json={
                "user_id": user_id,
                "timestamp": int(time.time()),
            },
        )
        response.raise_for_status()
        data = response.json()
        data["from_user"] = data.get("from", user_id)
        data["to_user"] = data.get("to", None)
        return CurrencyChainTransaction(**data)

    async def get_user_transactions(self, user_id: str) -> List[CurrencyChainTransaction]:
        """Get user transaction history"""
        response = await self.client.get(f"{self.rpc_url}/currency/transactions/{user_id}")
        response.raise_for_status()
        data = response.json()
        transactions = []
        for item in data:
            item["from_user"] = item.pop("from", None)
            item["to_user"] = item.pop("to", None)
            transactions.append(CurrencyChainTransaction(**item))
        return transactions

    async def get_transaction(self, tx_id: str) -> Optional[CurrencyChainTransaction]:
        """Get transaction by ID"""
        try:
            response = await self.client.get(f"{self.rpc_url}/currency/transaction/{tx_id}")
            if response.status_code == 404:
                return None
            response.raise_for_status()
            data = response.json()
            data["from_user"] = data.pop("from", None)
            data["to_user"] = data.pop("to", None)
            return CurrencyChainTransaction(**data)
        except httpx.HTTPError:
            return None

    async def wait_for_confirmation(
        self, tx_id: str, confirmations: int = 6, max_wait_ms: int = 30000
    ) -> CurrencyChainTransaction:
        """Wait for transaction to be confirmed"""
        start_time = time.time()
        while (time.time() - start_time) * 1000 < max_wait_ms:
            tx = await self.get_transaction(tx_id)
            if tx and tx.confirmations >= confirmations:
                return tx
            await asyncio.sleep(1)
        raise TimeoutError(f"Transaction {tx_id} did not confirm within {max_wait_ms}ms")

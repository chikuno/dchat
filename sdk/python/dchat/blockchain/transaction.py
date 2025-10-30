"""
Transaction types for blockchain operations
"""

from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Optional


class TransactionStatus(Enum):
    """Transaction status enum"""
    PENDING = "Pending"
    CONFIRMED = "Confirmed"
    FAILED = "Failed"
    TIMED_OUT = "TimedOut"


class ChannelVisibility(Enum):
    """Channel visibility enum"""
    PUBLIC = "Public"
    PRIVATE = "Private"
    TOKEN_GATED = "TokenGated"


@dataclass
class TransactionReceipt:
    """Transaction receipt with confirmation details"""
    tx_id: str
    tx_hash: str
    success: bool
    block_height: Optional[int] = None
    block_hash: Optional[str] = None
    timestamp: Optional[datetime] = None
    error: Optional[str] = None


@dataclass
class RegisterUserTx:
    """Register user transaction"""
    user_id: str
    username: str
    public_key: str
    timestamp: datetime
    initial_reputation: int = 100

    def to_dict(self) -> dict:
        return {
            "user_id": self.user_id,
            "username": self.username,
            "public_key": self.public_key,
            "timestamp": self.timestamp.isoformat(),
            "initial_reputation": self.initial_reputation,
        }


@dataclass
class SendDirectMessageTx:
    """Send direct message transaction"""
    message_id: str
    sender_id: str
    recipient_id: str
    content_hash: str
    payload_size: int
    timestamp: datetime
    relay_node_id: Optional[str] = None

    def to_dict(self) -> dict:
        data = {
            "message_id": self.message_id,
            "sender_id": self.sender_id,
            "recipient_id": self.recipient_id,
            "content_hash": self.content_hash,
            "payload_size": self.payload_size,
            "timestamp": self.timestamp.isoformat(),
        }
        if self.relay_node_id:
            data["relay_node_id"] = self.relay_node_id
        return data


@dataclass
class CreateChannelTx:
    """Create channel transaction"""
    channel_id: str
    name: str
    description: str
    creator_id: str
    visibility: ChannelVisibility
    timestamp: datetime
    token_requirement: Optional[str] = None

    def to_dict(self) -> dict:
        data = {
            "channel_id": self.channel_id,
            "name": self.name,
            "description": self.description,
            "creator_id": self.creator_id,
            "visibility": self.visibility.value,
            "timestamp": self.timestamp.isoformat(),
        }
        if self.token_requirement:
            data["token_requirement"] = self.token_requirement
        return data


@dataclass
class PostToChannelTx:
    """Post to channel transaction"""
    message_id: str
    channel_id: str
    sender_id: str
    content_hash: str
    payload_size: int
    timestamp: datetime

    def to_dict(self) -> dict:
        return {
            "message_id": self.message_id,
            "channel_id": self.channel_id,
            "sender_id": self.sender_id,
            "content_hash": self.content_hash,
            "payload_size": self.payload_size,
            "timestamp": self.timestamp.isoformat(),
        }

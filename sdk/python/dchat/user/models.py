"""
User management models and responses
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class CreateUserResponse:
    """Response for user creation"""
    user_id: str
    username: str
    public_key: str
    private_key: str
    created_at: str
    on_chain_confirmed: bool
    tx_id: Optional[str] = None


@dataclass
class UserProfile:
    """User profile information"""
    user_id: str
    username: str
    public_key: str
    created_at: str
    reputation: int
    on_chain_confirmed: bool


@dataclass
class DirectMessageResponse:
    """Direct message response"""
    message_id: str
    sender_id: str
    recipient_id: str
    content_hash: str
    created_at: str
    on_chain_confirmed: bool
    tx_id: Optional[str] = None


@dataclass
class CreateChannelResponse:
    """Channel creation response"""
    channel_id: str
    name: str
    creator_id: str
    created_at: str
    on_chain_confirmed: bool
    description: Optional[str] = None
    tx_id: Optional[str] = None


@dataclass
class ChannelMessage:
    """Channel message"""
    message_id: str
    channel_id: str
    sender_id: str
    content: str
    content_hash: str
    created_at: str
    on_chain_confirmed: bool


@dataclass
class DirectMessage:
    """Direct message with decrypted content"""
    message_id: str
    sender_id: str
    recipient_id: str
    content: str
    content_hash: str
    created_at: str
    on_chain_confirmed: bool

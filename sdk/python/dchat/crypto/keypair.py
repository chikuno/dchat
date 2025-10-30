"""
Cryptographic utilities for key management
"""

import hashlib
import os
from typing import Tuple

# Note: For production, use a proper Ed25519 library like PyNaCl or cryptography
# This is a simplified implementation


class KeyPair:
    """Ed25519 key pair for identity management"""

    def __init__(self, public_key: bytes, private_key: bytes):
        self.public_key = public_key
        self.private_key = private_key

    @classmethod
    def generate(cls) -> "KeyPair":
        """Generate a new random key pair"""
        # TODO: Use proper Ed25519 key generation
        # For now, using random bytes as placeholder
        private_key = os.urandom(32)
        public_key = os.urandom(32)  # Should be derived from private key
        return cls(public_key, private_key)

    @classmethod
    def from_private_key(cls, private_key: bytes) -> "KeyPair":
        """Create from existing private key"""
        # TODO: Derive public key from private key using Ed25519
        public_key = os.urandom(32)  # Placeholder
        return cls(public_key, private_key)

    @property
    def public_key_hex(self) -> str:
        """Get public key as hex string"""
        return self.public_key.hex()

    @property
    def private_key_hex(self) -> str:
        """Get private key as hex string"""
        return self.private_key.hex()

    def sign(self, message: bytes) -> bytes:
        """Sign a message"""
        # TODO: Implement proper Ed25519 signing
        # Placeholder implementation
        return hashlib.sha256(message + self.private_key).digest()

    def verify(self, message: bytes, signature: bytes) -> bool:
        """Verify a signature"""
        # TODO: Implement proper Ed25519 verification
        # Placeholder implementation
        expected = hashlib.sha256(message + self.private_key).digest()
        return signature == expected

    def to_dict(self) -> dict:
        """Export key pair to dictionary"""
        return {
            "public_key": self.public_key_hex,
            "private_key": self.private_key_hex,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "KeyPair":
        """Import key pair from dictionary"""
        return cls(
            public_key=bytes.fromhex(data["public_key"]),
            private_key=bytes.fromhex(data["private_key"]),
        )


def hash_content(content: str) -> str:
    """Hash content using SHA-256"""
    return hashlib.sha256(content.encode()).hexdigest()


def hash_bytes(data: bytes) -> str:
    """Hash bytes using SHA-256"""
    return hashlib.sha256(data).hexdigest()


def bytes_to_hex(data: bytes) -> str:
    """Convert bytes to hex string"""
    return data.hex()


def hex_to_bytes(hex_str: str) -> bytes:
    """Convert hex string to bytes"""
    return bytes.fromhex(hex_str)

"""
Example: Cryptographic operations
"""

from dchat.crypto import KeyPair, hash_content


def main():
    # Generate new key pair
    print("Generating key pair...")
    keypair = KeyPair.generate()
    print(f"Public Key: {keypair.public_key_hex}")
    print(f"Private Key: {keypair.private_key_hex}")

    # Sign a message
    message = b"Hello, blockchain!"
    print(f"\nSigning message: {message.decode()}")
    signature = keypair.sign(message)
    print(f"Signature: {signature.hex()}")

    # Verify signature
    is_valid = keypair.verify(message, signature)
    print(f"Signature valid: {is_valid}")

    # Hash content
    content = "Message content"
    content_hash = hash_content(content)
    print(f"\nContent: {content}")
    print(f"Hash: {content_hash}")

    # Export/import key pair
    keypair_dict = keypair.to_dict()
    print(f"\nExported keypair: {keypair_dict}")

    restored_keypair = KeyPair.from_dict(keypair_dict)
    print(f"Restored public key: {restored_keypair.public_key_hex}")


if __name__ == "__main__":
    main()

"""
Example: Using blockchain client directly

This example demonstrates:
- Blockchain client initialization
- User registration with on-chain confirmation
- Transaction status checking
- Block information retrieval
- Proper error handling and logging

Documentation:
    https://github.com/dchat-io/dchat/tree/main/sdk/python
"""

import asyncio
import logging
from typing import Optional

from dchat import BlockchainClient

__all__ = ["main", "register_user_example"]

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Configuration constants
DEFAULT_RPC_URL = "http://localhost:8545"
DEFAULT_USER_ID = "user-123"
DEFAULT_USERNAME = "alice"
DEFAULT_PUBLIC_KEY = "ed25519-public-key-hex"
CONFIRMATION_TIMEOUT_SECONDS = 300
MAX_RETRY_ATTEMPTS = 3


async def register_user_example(
    blockchain: BlockchainClient,
    user_id: str = DEFAULT_USER_ID,
    username: str = DEFAULT_USERNAME,
    public_key: str = DEFAULT_PUBLIC_KEY,
) -> Optional[str]:
    """
    Register a new user on the blockchain.

    Args:
        blockchain: Initialized BlockchainClient instance
        user_id: Unique identifier for the user
        username: Human-readable username
        public_key: User's Ed25519 public key in hex format

    Returns:
        Transaction ID if successful, None otherwise

    Raises:
        TimeoutError: If confirmation times out
        ValueError: If input validation fails
    """
    # Input validation
    if not user_id or not isinstance(user_id, str):
        logger.error("Invalid user_id: must be non-empty string")
        raise ValueError("user_id must be a non-empty string")

    if not username or not isinstance(username, str):
        logger.error("Invalid username: must be non-empty string")
        raise ValueError("username must be a non-empty string")

    if not public_key or not isinstance(public_key, str):
        logger.error("Invalid public_key: must be non-empty string")
        raise ValueError("public_key must be a non-empty string")

    logger.info(f"Registering user: {username} (ID: {user_id})")

    retry_attempt = 0
    while retry_attempt < MAX_RETRY_ATTEMPTS:
        try:
            # Submit user registration transaction
            logger.debug(
                f"Submitting registration transaction (attempt {retry_attempt + 1}/{MAX_RETRY_ATTEMPTS})"
            )
            tx_id: str = await blockchain.register_user(
                user_id=user_id,
                username=username,
                public_key=public_key,
            )
            logger.info(f"Transaction submitted: {tx_id}")

            # Wait for confirmation with timeout
            logger.info("Waiting for blockchain confirmation...")
            receipt = await asyncio.wait_for(
                blockchain.wait_for_confirmation(tx_id),
                timeout=CONFIRMATION_TIMEOUT_SECONDS,
            )

            # Log confirmation result
            if receipt.success:
                logger.info("✓ Transaction confirmed!")
                logger.info(f"  Block Height: {receipt.block_height}")
                logger.info(f"  Block Hash: {receipt.block_hash}")
                logger.info(f"  Gas Used: {getattr(receipt, 'gas_used', 'N/A')}")
                return tx_id
            else:
                error_message = getattr(receipt, "error", "Unknown error")
                logger.error(f"✗ Transaction failed: {error_message}")
                retry_attempt += 1
                if retry_attempt < MAX_RETRY_ATTEMPTS:
                    await asyncio.sleep(2 ** retry_attempt)  # Exponential backoff
                continue

        except TimeoutError as e:
            logger.error(f"Transaction confirmation timed out: {e}")
            retry_attempt += 1
            if retry_attempt < MAX_RETRY_ATTEMPTS:
                logger.info(f"Retrying... (attempt {retry_attempt + 1}/{MAX_RETRY_ATTEMPTS})")
                await asyncio.sleep(2 ** retry_attempt)
            else:
                raise
        except ValueError as e:
            logger.error(f"Validation error: {e}")
            raise
        except Exception as e:
            logger.error(f"Unexpected error during registration: {type(e).__name__}: {e}")
            retry_attempt += 1
            if retry_attempt >= MAX_RETRY_ATTEMPTS:
                raise

    logger.error(f"Failed to register user after {MAX_RETRY_ATTEMPTS} attempts")
    return None


async def check_blockchain_status(blockchain: BlockchainClient) -> bool:
    """
    Check blockchain connectivity and status.

    Args:
        blockchain: Initialized BlockchainClient instance

    Returns:
        True if blockchain is accessible, False otherwise
    """
    try:
        logger.info("Checking blockchain status...")
        block_number: int = await asyncio.wait_for(
            blockchain.get_block_number(),
            timeout=5.0,
        )
        logger.info(f"✓ Blockchain accessible. Current block: {block_number}")
        return True
    except TimeoutError:
        logger.error("✗ Blockchain connection timed out")
        return False
    except Exception as e:
        logger.error(f"✗ Failed to connect to blockchain: {type(e).__name__}: {e}")
        return False


async def main() -> None:
    """
    Main entry point demonstrating blockchain client usage.

    Workflow:
    1. Initialize BlockchainClient
    2. Check blockchain status
    3. Register a new user
    4. Verify transaction confirmation
    5. Query blockchain status
    """
    logger.info("=" * 60)
    logger.info("dchat Blockchain Client Example")
    logger.info("=" * 60)

    # Create blockchain client with local configuration
    try:
        blockchain = BlockchainClient.local()
        logger.info("✓ BlockchainClient initialized")
    except Exception as e:
        logger.error(f"✗ Failed to initialize BlockchainClient: {e}")
        return

    # Step 1: Check blockchain status
    is_accessible = await check_blockchain_status(blockchain)
    if not is_accessible:
        logger.error("Cannot proceed: blockchain is not accessible")
        return

    # Step 2: Register user
    try:
        tx_id = await register_user_example(
            blockchain=blockchain,
            user_id=DEFAULT_USER_ID,
            username=DEFAULT_USERNAME,
            public_key=DEFAULT_PUBLIC_KEY,
        )

        if tx_id:
            logger.info(f"✓ User registered successfully. TX: {tx_id}")

            # Step 3: Verify transaction status
            logger.info("Verifying transaction status...")
            is_confirmed: bool = await blockchain.is_transaction_confirmed(tx_id)
            logger.info(f"Transaction confirmed: {is_confirmed}")
        else:
            logger.error("✗ Failed to register user")

    except TimeoutError as e:
        logger.error(f"✗ Operation timed out: {e}")
    except ValueError as e:
        logger.error(f"✗ Validation error: {e}")
    except Exception as e:
        logger.error(f"✗ Unexpected error: {type(e).__name__}: {e}")

    logger.info("=" * 60)
    logger.info("Example completed")
    logger.info("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())

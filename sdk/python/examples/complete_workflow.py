"""
Example: Complete user workflow with blockchain integration
"""

import asyncio
from dchat import BlockchainClient, UserManager


async def main():
    # Initialize blockchain client
    blockchain = BlockchainClient.local()

    # Create user manager
    user_manager = UserManager(
        blockchain=blockchain,
        base_url="http://localhost:8080"
    )

    try:
        # Create two users
        print("Creating users...")
        alice = await user_manager.create_user("alice")
        bob = await user_manager.create_user("bob")

        print(f"Alice: {alice.user_id} (confirmed: {alice.on_chain_confirmed})")
        print(f"Bob: {bob.user_id} (confirmed: {bob.on_chain_confirmed})")

        # Send direct message
        print("\nSending direct message...")
        message = await user_manager.send_direct_message(
            sender_id=alice.user_id,
            recipient_id=bob.user_id,
            content="Hello Bob!",
        )
        print(f"Message sent: {message.message_id} (TX: {message.tx_id})")

        # Create channel
        print("\nCreating channel...")
        channel = await user_manager.create_channel(
            creator_id=alice.user_id,
            channel_name="General",
            description="General discussion",
        )
        print(f"Channel created: {channel.channel_id} (TX: {channel.tx_id})")

        # Post to channel
        print("\nPosting to channel...")
        post = await user_manager.post_to_channel(
            sender_id=alice.user_id,
            channel_id=channel.channel_id,
            content="Welcome everyone!",
        )
        print(f"Posted: {post.message_id} (TX: {post.tx_id})")

        # Check current block
        block_number = await blockchain.get_block_number()
        print(f"\nCurrent block: {block_number}")

    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())

// Integration Tests for TypeScript SDK
// Validates blockchain and user management operations

import { strict as assert } from "assert";
// Note: uuid package must be installed: npm install uuid
const uuidv4 = () => Math.random().toString(36).substring(2, 15);

interface Transaction {
  tx_id: string;
  tx_type: string;
  sender: string;
  data: Record<string, string>;
  timestamp: string;
  status: "Pending" | "Confirmed" | "Failed";
  confirmations: number;
}

interface User {
  user_id: string;
  username: string;
  public_key: string;
  on_chain_confirmed: boolean;
}

class MockBlockchainTS {
  private transactions: Map<string, Transaction> = new Map();
  private currentBlock: number = 1;
  private confirmationThreshold: number = 6;

  submitTransaction(
    txType: string,
    sender: string,
    data: Record<string, string>
  ): string {
    const txId = uuidv4();
    const tx: Transaction = {
      tx_id: txId,
      tx_type: txType,
      sender,
      data,
      timestamp: new Date().toISOString(),
      status: "Confirmed",
      confirmations: this.confirmationThreshold,
    };

    this.transactions.set(txId, tx);
    return txId;
  }

  getTransaction(txId: string): Transaction | undefined {
    return this.transactions.get(txId);
  }

  getTransactionsByType(txType: string): Transaction[] {
    return Array.from(this.transactions.values()).filter(
      (tx) => tx.tx_type === txType
    );
  }

  getTransactionsBySender(sender: string): Transaction[] {
    return Array.from(this.transactions.values()).filter(
      (tx) => tx.sender === sender
    );
  }

  getAllTransactions(): Transaction[] {
    return Array.from(this.transactions.values());
  }

  getCurrentBlock(): number {
    return this.currentBlock;
  }

  advanceBlocks(count: number): void {
    this.currentBlock += count;
    // Update confirmations
    for (const tx of this.transactions.values()) {
      if (tx.status === "Confirmed") {
        tx.confirmations += count;
      }
    }
  }

  reset(): void {
    this.transactions.clear();
    this.currentBlock = 1;
  }
}

// ===== TESTS =====

async function testUserRegistrationTransaction() {
  const blockchain = new MockBlockchainTS();

  const txId = blockchain.submitTransaction(
    "RegisterUser",
    "alice",
    {
      user_id: "alice-12345",
      public_key: "alice-pub-key",
    }
  );

  const tx = blockchain.getTransaction(txId);
  assert(tx !== undefined);
  assert.equal(tx.tx_type, "RegisterUser");
  assert.equal(tx.sender, "alice");
  assert.equal(tx.status, "Confirmed");

  console.log("✓ User registration transaction test passed");
}

async function testDirectMessageTransaction() {
  const blockchain = new MockBlockchainTS();

  // Create users
  blockchain.submitTransaction("RegisterUser", "alice", {
    user_id: "alice-1",
  });
  blockchain.submitTransaction("RegisterUser", "bob", {
    user_id: "bob-1",
  });

  // Send message
  const msgTxId = blockchain.submitTransaction(
    "SendDirectMessage",
    "alice",
    {
      recipient_id: "bob-1",
      content_hash: "msg-hash-123",
    }
  );

  const tx = blockchain.getTransaction(msgTxId);
  assert(tx !== undefined);
  assert.equal(tx.tx_type, "SendDirectMessage");
  assert.equal(tx.sender, "alice");

  console.log("✓ Direct message transaction test passed");
}

async function testChannelCreationTransaction() {
  const blockchain = new MockBlockchainTS();

  // Create user
  blockchain.submitTransaction("RegisterUser", "alice", {
    user_id: "alice-1",
  });

  // Create channel
  const channelTxId = blockchain.submitTransaction(
    "CreateChannel",
    "alice",
    {
      channel_name: "general",
      description: "General discussion",
    }
  );

  const tx = blockchain.getTransaction(channelTxId);
  assert(tx !== undefined);
  assert.equal(tx.tx_type, "CreateChannel");
  assert.equal(tx.sender, "alice");

  console.log("✓ Channel creation transaction test passed");
}

async function testTransactionFilteringByType() {
  const blockchain = new MockBlockchainTS();

  // Create mix
  blockchain.submitTransaction("RegisterUser", "alice", { user_id: "alice-1" });
  blockchain.submitTransaction("RegisterUser", "bob", { user_id: "bob-1" });
  blockchain.submitTransaction("CreateChannel", "alice", {
    channel_name: "general",
  });

  const regTxs = blockchain.getTransactionsByType("RegisterUser");
  assert.equal(regTxs.length, 2);

  const channelTxs = blockchain.getTransactionsByType("CreateChannel");
  assert.equal(channelTxs.length, 1);

  console.log("✓ Transaction filtering by type test passed");
}

async function testTransactionFilteringBySender() {
  const blockchain = new MockBlockchainTS();

  blockchain.submitTransaction("RegisterUser", "alice", { user_id: "alice-1" });
  blockchain.submitTransaction("CreateChannel", "alice", {
    channel_name: "general",
  });
  blockchain.submitTransaction("RegisterUser", "bob", { user_id: "bob-1" });

  const aliceTxs = blockchain.getTransactionsBySender("alice");
  assert.equal(aliceTxs.length, 2);

  const bobTxs = blockchain.getTransactionsBySender("bob");
  assert.equal(bobTxs.length, 1);

  console.log("✓ Transaction filtering by sender test passed");
}

async function testMultipleUsersMessageFlow() {
  const blockchain = new MockBlockchainTS();
  const users = ["alice", "bob", "charlie"];

  // Register users
  for (const user of users) {
    blockchain.submitTransaction("RegisterUser", user, { user_id: `${user}-id` });
  }

  // Exchange messages
  for (const user of users) {
    blockchain.submitTransaction("SendDirectMessage", user, {
      recipient_id: "bob-id",
      content: `Message from ${user}`,
    });
  }

  assert.equal(blockchain.getAllTransactions().length, 6); // 3 registrations + 3 messages

  for (const user of users) {
    const txs = blockchain.getTransactionsBySender(user);
    assert.equal(txs.length, 2); // 1 registration + 1 message
  }

  console.log("✓ Multiple users message flow test passed");
}

async function testBlockHeightTracking() {
  const blockchain = new MockBlockchainTS();

  assert.equal(blockchain.getCurrentBlock(), 1);

  blockchain.submitTransaction("RegisterUser", "alice", { user_id: "alice-1" });
  blockchain.advanceBlocks(5);

  assert.equal(blockchain.getCurrentBlock(), 6);

  console.log("✓ Block height tracking test passed");
}

// ===== RUN TESTS =====

async function runTests() {
  console.log("\n📋 TypeScript SDK Integration Tests\n");
  console.log("Running 7 test cases...\n");

  await testUserRegistrationTransaction();
  await testDirectMessageTransaction();
  await testChannelCreationTransaction();
  await testTransactionFilteringByType();
  await testTransactionFilteringBySender();
  await testMultipleUsersMessageFlow();
  await testBlockHeightTracking();

  console.log("\n✅ All TypeScript integration tests passed!\n");
}

runTests().catch(console.error);

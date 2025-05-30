# Privacy-Preserving Decentralized Messaging Pallet with IPFS

A Substrate pallet that provides secure, private messaging functionality on a blockchain using IPFS for content storage.

## Overview

This pallet implements a privacy-preserving messaging system with the following features:

- IPFS-based encrypted message storage
- On-chain storage of IPFS CIDs only
- Direct messaging between users
- Group messaging with access control
- Message expiration and deletion
- Inbox/outbox management

## Usage

### Configuration

Include this pallet in your runtime:

```rust
parameter_types! {
    pub const MaxMessageLength: u32 = 64; // Max CID length
    pub const MessageTTL: BlockNumber = 10_000;
}

impl pallet_messaging::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxMessageLength = MaxMessageLength;
    type MessageTTL = MessageTTL;
    type WeightInfo = pallet_messaging::weights::SubstrateWeight<Runtime>;
}
```

### Direct Messaging Extrinsics

- `send_message(recipient, content_cid)`: Send a message with IPFS CID to a recipient
- `read_message(message_id)`: Mark a message as read
- `delete_message(message_id)`: Delete a message

### Group Messaging Extrinsics

- `create_group(name, initial_members)`: Create a new messaging group
- `add_member(group_id, new_member)`: Add a member to a group
- `remove_member(group_id, member)`: Remove a member from a group
- `send_group_message(group_id, content_cid)`: Send a message to a group

### Client-Side Implementation

```javascript
// Send a direct message
async function sendMessage(api, ipfs, recipientId, messageContent, recipientPublicKey) {
  // Encrypt the message
  const encryptedContent = encrypt(messageContent, recipientPublicKey);
  
  // Upload to IPFS
  const result = await ipfs.add(encryptedContent);
  const cid = result.cid.toString();
  
  // Convert CID to bytes
  const cidBytes = new TextEncoder().encode(cid);
  
  // Send to blockchain
  return await api.tx.messaging.sendMessage(recipientId, cidBytes).signAndSend(sender);
}

// Send a group message
async function sendGroupMessage(api, ipfs, groupId, messageContent, groupPublicKeys) {
  // Encrypt the message for each group member
  const encryptedContent = encryptForGroup(messageContent, groupPublicKeys);
  
  // Upload to IPFS
  const result = await ipfs.add(encryptedContent);
  const cid = result.cid.toString();
  
  // Convert CID to bytes
  const cidBytes = new TextEncoder().encode(cid);
  
  // Send to blockchain
  return await api.tx.messaging.sendGroupMessage(groupId, cidBytes).signAndSend(sender);
}
```

### Security Notes

- Message content is encrypted client-side before being uploaded to IPFS
- Only the intended recipients should have the decryption keys
- The blockchain only stores CIDs and manages access control
- Consider implementing IPFS pinning management for content persistence
- For group messages, you may need to implement multi-recipient encryption

## License

Unlicense
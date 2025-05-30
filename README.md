# Secura Chain

A privacy-focused blockchain platform that combines secure messaging with a validator system, creating a decentralized ecosystem where users can communicate privately while participating in network security through staking.

![Secura Chain]

## 🔒 Overview

Secura Chain is a blockchain platform built on Substrate that prioritizes privacy and security. It offers:

- **Privacy-Preserving Messaging**: End-to-end encrypted communication using IPFS for content storage
- **Secure Validator System**: Participate in network consensus through staking
- **Decentralized Architecture**: No central authority controlling user data
- **Scalable Infrastructure**: Built on Substrate for future-proof development

## 🚀 Features

### Privacy-Preserving Messaging

- End-to-end encrypted direct messaging
- Group messaging with access control
- IPFS-based content storage (only CIDs stored on-chain)
- Message expiration and deletion
- Inbox/outbox management

### Validator System

- Secure network participation through staking
- Consensus mechanism using Aura for block production and GRANDPA for finality
- Rewards for validators who maintain network security

### Technical Foundation

- Built on Substrate framework for modular blockchain development
- Custom pallets for messaging and validation
- FRAME runtime for flexible blockchain composition

## 🛠️ Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- IPFS daemon (for messaging functionality)
- Git

### Installation

1. Clone the repository:

```bash
git clone https://github.com/SecuraChain-Official/secura-chain.git
cd secura-chain
```

2. Build the project:

```bash
cargo build --release
```

### Running a Node

Start a development node:

```bash
./target/release/secura-chain-node --dev
```

Start a node with persistent state:

```bash
./target/release/secura-chain-node --dev --base-path ./my-chain-state/
```

Purge chain state:

```bash
./target/release/secura-chain-node purge-chain --dev
```

### Connecting to the Network

You can interact with the network using the Polkadot/Substrate Portal:

- Local node: [https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)

## 📱 Using the Messaging System

### Direct Messaging

Send an encrypted message:

1. Encrypt your message using the recipient's public key
2. Upload the encrypted content to IPFS
3. Send the IPFS CID to the recipient through the blockchain

```javascript
// Client-side example
async function sendMessage(api, ipfs, recipientId, messageContent, recipientPublicKey) {
  // Encrypt the message
  const encryptedContent = encrypt(messageContent, recipientPublicKey);
  
  // Upload to IPFS
  const result = await ipfs.add(encryptedContent);
  const cid = result.cid.toString();
  
  // Send to blockchain
  return await api.tx.messaging.sendMessage(recipientId, cid).signAndSend(sender);
}
```

### Group Messaging

Create and manage secure group conversations:

1. Create a group with initial members
2. Encrypt messages for all group members
3. Share the IPFS CID through the blockchain

```javascript
// Client-side example
async function sendGroupMessage(api, ipfs, groupId, messageContent, groupPublicKeys) {
  // Encrypt the message for each group member
  const encryptedContent = encryptForGroup(messageContent, groupPublicKeys);
  
  // Upload to IPFS
  const result = await ipfs.add(encryptedContent);
  const cid = result.cid.toString();
  
  // Send to blockchain
  return await api.tx.messaging.sendGroupMessage(groupId, cid).signAndSend(sender);
}
```

## 🔍 Architecture

### Core Components

- **Node**: Handles networking, consensus, and RPC server functionality
- **Runtime**: Implements the state transition function using FRAME
- **Pallets**:
  - **Messaging**: Provides privacy-preserving communication
  - **Template**: Base for custom functionality

### Privacy Model

Secura Chain's privacy model is based on:

1. Client-side encryption of all message content
2. On-chain storage of only IPFS CIDs, not actual content
3. Access control through public key cryptography
4. Message expiration and deletion capabilities

## 🧪 Development

### Project Structure

```
secura-chain/
├── docs/               # Documentation
├── env-setup/          # Environment setup tools
├── node/               # Node implementation
│   └── src/            # Node source code
├── pallets/            # Custom pallets
│   ├── messaging/      # Privacy-preserving messaging
│   └── template/       # Template for new pallets
├── runtime/            # Runtime implementation
└── ...
```

### Adding New Features

To add new features to Secura Chain:

1. Create a new pallet or extend existing ones
2. Implement the required functionality
3. Update the runtime to include your changes
4. Test thoroughly before deployment

### Testing

Run the test suite:

```bash
cargo test
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔮 Roadmap

- Mobile client application
- Enhanced privacy features
- Cross-chain messaging capabilities
- Governance system for protocol upgrades
- Integration with existing privacy tools

## ⚠️ Security Notes

- Message content is encrypted client-side before being uploaded to IPFS
- Only the intended recipients have the decryption keys
- The blockchain only stores CIDs and manages access control
- Consider implementing IPFS pinning management for content persistence
- For group messages, implement multi-recipient encryption

## 📞 Contact

For questions or support, please open an issue on GitHub or contact the team at [info@secura.tech](mailto:info@secura.tech).
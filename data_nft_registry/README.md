# 🎨 Data NFT Registry

Smart contract for minting and managing data NFTs with access control and privacy settings in the PolkadotAiMesh network.

**Status:** ✅ Builds and compiles  
**Environment:** EVM-like primitives (H160 addresses, U256 balances)

---

## 🏗️ Constructor

```mermaid
graph LR
    A[new] --> B[Initialize empty registry<br/>Deployer becomes admin<br/>Total supply = 0]
    style A fill:#e1f5ff
    style B fill:#d4edda
```

---

## 📝 Contract Functions

```mermaid
graph TB
    subgraph "💰 Payable Functions"
        A1[mint<br/>💵 Create new data NFT]
        A2[grant_access<br/>💵 Grant access with payment]
    end
    
    subgraph "👤 Owner Functions"
        B1[transfer<br/>Transfer NFT if transferable]
        B2[approve<br/>Approve address for transfer]
        B3[revoke_access<br/>Remove access rights]
        B4[update_data_uri<br/>Update NFT URI]
        B5[update_access_price<br/>Change access price]
        B6[burn<br/>Destroy NFT]
    end
    
    subgraph "📊 Query Functions"
        C1[get_nft<br/>Retrieve NFT metadata]
        C2[balance_of<br/>Get NFT count]
        C3[get_approved<br/>Get approved address]
        C4[has_access<br/>Check access rights]
        C5[total_supply<br/>Total NFTs minted]
        C6[get_admin<br/>Get admin address]
    end
    
    style A1 fill:#fff3cd
    style A2 fill:#fff3cd
    style B1 fill:#d1ecf1
    style B2 fill:#d1ecf1
    style B3 fill:#d1ecf1
    style B4 fill:#d1ecf1
    style B5 fill:#d1ecf1
    style B6 fill:#d1ecf1
    style C1 fill:#e2e3e5
    style C2 fill:#e2e3e5
    style C3 fill:#e2e3e5
    style C4 fill:#e2e3e5
    style C5 fill:#e2e3e5
    style C6 fill:#e2e3e5
```

---

## 🔄 NFT Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Minted: mint() 💰
    
    Minted --> Transferred: transfer() if transferable
    Transferred --> Transferred: transfer() again
    
    Minted --> AccessGranted: grant_access() 💰
    Transferred --> AccessGranted: grant_access() 💰
    AccessGranted --> AccessRevoked: revoke_access()
    
    Minted --> Burned: burn()
    Transferred --> Burned: burn()
    
    Burned --> [*]
    
    note right of Minted
        NFT created
        Owner = minter
        Access list empty
    end note
    
    note right of AccessGranted
        3rd party granted access
        Owner retains ownership
    end note
    
    note right of Burned
        NFT destroyed
        Cannot be recovered
    end note
```

---

## 📋 Function Details

### 💰 mint (Payable)

```mermaid
graph LR
    A[data_uri: String<br/>privacy_level: u8<br/>access_price: U256<br/>is_transferable: bool<br/>+ OPTIONAL PAYMENT] --> B[mint]
    B --> C[Create NFT<br/>token_id = supply + 1]
    C --> D[Set owner = caller]
    D --> E[Emit NFTMinted]
    E --> F[Return token_id: u128]
    
    style A fill:#e1f5ff
    style B fill:#fff3cd
    style C fill:#d4edda
    style D fill:#d4edda
    style E fill:#dfe6e9
    style F fill:#d4edda
```

**Note:** Minting is always successful. Payment is optional (for future extensions).

---

### 👤 transfer (Owner Only)

```mermaid
graph LR
    A[token_id: u128<br/>to: H160] --> B[transfer]
    B --> C{Caller = Owner?<br/>is_transferable?}
    C -->|✅ Yes| D[Update owner<br/>Update balances<br/>Clear approvals]
    C -->|❌ No| E[Return false]
    D --> F[Emit NFTTransferred]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

**Requirements:**
- Caller must be owner
- `is_transferable == true`

---

### 👤 approve (Owner Only)

```mermaid
graph LR
    A[token_id: u128<br/>approved: H160] --> B[approve]
    B --> C{Caller = Owner?<br/>NFT exists?}
    C -->|✅ Yes| D[Set approval]
    C -->|❌ No| E[Return false]
    D --> F[Emit NFTApproved]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

---

### 💰 grant_access (Owner Only, Payable)

```mermaid
graph LR
    A[token_id: u128<br/>grantee: H160<br/>+ PAYMENT] --> B[grant_access]
    B --> C{Caller = Owner?<br/>payment ≥ access_price?}
    C -->|✅ Yes| D[Grant access<br/>Add to access list]
    C -->|❌ No| E[Return false]
    D --> F[Emit AccessGranted]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#fff3cd
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

**Requirements:**
- Caller must be owner
- `transferred_value >= access_price`

---

### 👤 revoke_access (Owner/Admin Only)

```mermaid
graph LR
    A[token_id: u128<br/>grantee: H160] --> B[revoke_access]
    B --> C{Owner or Admin?<br/>NFT exists?}
    C -->|✅ Yes| D[Remove access]
    C -->|❌ No| E[Return false]
    D --> F[Emit AccessRevoked]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

---

### 👤 update_data_uri (Owner Only)

```mermaid
graph LR
    A[token_id: u128<br/>new_uri: String] --> B[update_data_uri]
    B --> C{Caller = Owner?<br/>NFT exists?}
    C -->|✅ Yes| D[Update URI]
    C -->|❌ No| E[Return false]
    D --> F[Emit DataURIUpdated]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

---

### 👤 update_access_price (Owner Only)

```mermaid
graph LR
    A[token_id: u128<br/>new_price: U256] --> B[update_access_price]
    B --> C{Caller = Owner?<br/>NFT exists?}
    C -->|✅ Yes| D[Update price]
    C -->|❌ No| E[Return false]
    D --> F[Emit AccessPriceUpdated]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

---

### 👤 burn (Owner/Admin Only)

```mermaid
graph LR
    A[token_id: u128] --> B[burn]
    B --> C{Owner or Admin?<br/>NFT exists?}
    C -->|✅ Yes| D[Delete NFT<br/>Update balance]
    C -->|❌ No| E[Return false]
    D --> F[Emit NFTBurned]
    D --> G[Return true]
    
    style A fill:#e1f5ff
    style B fill:#d1ecf1
    style C fill:#ffeaa7
    style D fill:#d4edda
    style E fill:#f8d7da
    style F fill:#dfe6e9
    style G fill:#d4edda
```

---

## 📊 Query Functions

```mermaid
graph TB
    A[get_nft] -->|token_id| A1[Returns: Option DataNFT]
    B[balance_of] -->|owner: H160| B1[Returns: u128 count]
    C[get_approved] -->|token_id| C1[Returns: Option H160]
    D[has_access] -->|token_id, account| D1[Returns: bool]
    E[total_supply] --> E1[Returns: u128 total]
    F[get_admin] --> F1[Returns: H160 admin]
    
    style A fill:#e2e3e5
    style B fill:#e2e3e5
    style C fill:#e2e3e5
    style D fill:#e2e3e5
    style E fill:#e2e3e5
    style F fill:#e2e3e5
    style A1 fill:#d4edda
    style B1 fill:#d4edda
    style C1 fill:#d4edda
    style D1 fill:#d4edda
    style E1 fill:#d4edda
    style F1 fill:#d4edda
```

---

## 📤 Events

```mermaid
graph LR
    subgraph Events
        E1[NFTMinted<br/>token_id, owner, data_uri, privacy_level]
        E2[NFTTransferred<br/>token_id, from, to]
        E3[NFTApproved<br/>token_id, owner, approved]
        E4[AccessGranted<br/>token_id, grantee, payment]
        E5[AccessRevoked<br/>token_id, grantee]
        E6[DataURIUpdated<br/>token_id, new_uri]
        E7[AccessPriceUpdated<br/>token_id, new_price]
        E8[NFTBurned<br/>token_id, owner]
    end
    
    style E1 fill:#dfe6e9
    style E2 fill:#dfe6e9
    style E3 fill:#dfe6e9
    style E4 fill:#dfe6e9
    style E5 fill:#dfe6e9
    style E6 fill:#dfe6e9
    style E7 fill:#dfe6e9
    style E8 fill:#dfe6e9
```

---

## 🏗️ Data Types

### DataNFT Structure

```mermaid
classDiagram
    class DataNFT {
        +u128 token_id
        +H160 owner
        +String data_uri
        +u8 privacy_level
        +u64 minted_at
        +U256 access_price
        +bool is_transferable
    }
    
    note for DataNFT "Immutable:\n- token_id\n- minted_at\n\nMutable:\n- owner (via transfer)\n- data_uri (owner)\n- access_price (owner)\n\nFixed after mint:\n- privacy_level\n- is_transferable"
```

---

## ⚙️ Access Control

```mermaid
graph TB
    subgraph "🔐 Permissions"
        A[NFT Owner] -->|Can call| A1[transfer if transferable]
        A -->|Can call| A2[approve]
        A -->|Can call| A3[grant_access 💰]
        A -->|Can call| A4[revoke_access]
        A -->|Can call| A5[update_data_uri]
        A -->|Can call| A6[update_access_price]
        A -->|Can call| A7[burn]
        
        B[Admin] -->|Can call| B1[revoke_access any]
        B -->|Can call| B2[burn any]
        
        C[Anyone] -->|Can call| C1[mint 💰]
        C -->|Can call| C2[Query functions]
    end
    
    style A fill:#d1ecf1
    style B fill:#f8d7da
    style C fill:#e2e3e5
```

---

## 🎯 Minting & Access Flow

```mermaid
sequenceDiagram
    participant Alice
    participant Contract
    participant Bob
    
    Alice->>Contract: mint(uri, privacy=1, price, transferable) 💰
    Contract-->>Alice: NFT #1 minted ✅
    
    Bob->>Contract: grant_access(token_id=1, Bob) 💰
    Note over Bob: ❌ Only owner can grant
    
    Alice->>Contract: grant_access(token_id=1, Bob) 💰
    Contract-->>Alice: Access granted to Bob ✅
    
    Bob->>Contract: has_access(token_id=1, Bob)
    Contract-->>Bob: true ✅
    
    Alice->>Contract: revoke_access(token_id=1, Bob)
    Contract-->>Alice: Access revoked ✅
    
    Bob->>Contract: has_access(token_id=1, Bob)
    Contract-->>Bob: false ❌
```

---

## 🔐 Privacy Levels

```mermaid
graph LR
    subgraph "Privacy Scale 0-255"
        A[0: Public<br/>Open data]
        B[1-99: Low<br/>Semi-public]
        C[100-199: Medium<br/>Restricted]
        D[200-255: High<br/>Confidential]
    end
    
    style A fill:#d4edda
    style B fill:#fff3cd
    style C fill:#ffc107
    style D fill:#f8d7da
```

**Privacy Level Usage:**
- **0**: Public data, no restrictions
- **1-99**: Low privacy, minimal protection
- **100-199**: Medium privacy, restricted access
- **200-255**: High privacy, highly confidential

*Note: Privacy enforcement happens off-chain via access verification*

---

## 🔒 Constraints & Rules

```mermaid
graph TB
    subgraph "✅ Minting"
        A1[Anyone can mint NFTs]
        A2[Token IDs auto-increment]
        A3[Minter becomes initial owner]
    end
    
    subgraph "✅ Transfers"
        B1[Only owner can transfer]
        B2[Cannot transfer if is_transferable = false]
        B3[Approvals cleared on transfer]
        B4[Owner counts updated automatically]
    end
    
    subgraph "✅ Access Control"
        C1[Owner has automatic access]
        C2[Must pay ≥ access_price to grant]
        C3[Owner can revoke anytime]
        C4[Admin can revoke any access]
    end
    
    subgraph "✅ Updates"
        D1[Only owner can update data URI]
        D2[Only owner can update access price]
        D3[Settings persist across transfers]
    end
    
    subgraph "✅ Burning"
        E1[Owner can burn their NFT]
        E2[Admin can burn any NFT]
        E3[Burned NFTs cannot be recovered]
    end
    
    style A1 fill:#d4edda
    style A2 fill:#d4edda
    style A3 fill:#d4edda
    style B1 fill:#d1ecf1
    style B2 fill:#d1ecf1
    style B3 fill:#d1ecf1
    style B4 fill:#d1ecf1
    style C1 fill:#fff3cd
    style C2 fill:#fff3cd
    style C3 fill:#fff3cd
    style C4 fill:#fff3cd
    style D1 fill:#e2e3e5
    style D2 fill:#e2e3e5
    style D3 fill:#e2e3e5
    style E1 fill:#f8d7da
    style E2 fill:#f8d7da
    style E3 fill:#f8d7da
```

---

## 🛡️ Safety Features

✅ **Owner Rights:**
- Full control over their NFTs
- Can update metadata and pricing
- Can grant/revoke access

✅ **Access Management:**
- Payment required for access grants
- Owner always has implicit access
- Granular per-NFT access control

✅ **Transfer Control:**
- NFTs can be made non-transferable
- Useful for soulbound data tokens
- Protects against unwanted transfers

✅ **Admin Oversight:**
- Admin can moderate content (burn)
- Admin can revoke malicious access
- Emergency intervention capability

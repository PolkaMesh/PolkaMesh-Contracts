# 🖥️ Compute Provider Registry

Smart contract for managing compute provider registration and profiles in the PolkadotAiMesh network.

**Status:** ✅ Builds and compiles  
**Environment:** EVM-like primitives (H160 addresses, U256 balances)

## 🧪 Tests

- Unit tests: 22 passing (contract test suite validated locally)

How to run the tests locally:

```bash
# from the repository root
cd PolkaMesh-Contracts/compute_provider_registry
cargo test
```

---

## 🏗️ Constructor

```mermaid
graph LR
    A[new] -->|min_stake: U256| B[Initialize registry<br/>Set minimum stake<br/>Deployer becomes admin]
    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📝 Contract Functions

```mermaid
graph TB
    subgraph "💰 Payable Functions"
        A1[register_provider<br/>💵 Register with stake]
        A2[add_stake<br/>💵 Increase stake]
    end

    subgraph "👤 Provider Functions"
        B1[update_provider<br/>Update endpoint & rate]
        B2[set_active<br/>Toggle active status]
        B3[withdraw_stake<br/>Withdraw stake when inactive]
    end

    subgraph "⚙️ Admin Functions"
        C1[set_reputation<br/>Update reputation score]
        C2[set_min_stake<br/>Update minimum stake]
    end

    subgraph "📊 Query Functions"
        D1[get_provider<br/>Retrieve profile]
        D2[get_admin<br/>Get admin address]
        D3[get_provider_count<br/>Total providers]
        D4[get_min_stake<br/>Minimum stake value]
    end

    style A1 fill:#ffffff,stroke:#000000,color:#000000
    style A2 fill:#ffffff,stroke:#000000,color:#000000
    style B1 fill:#ffffff,stroke:#000000,color:#000000
    style B2 fill:#ffffff,stroke:#000000,color:#000000
    style B3 fill:#ffffff,stroke:#000000,color:#000000
    style C1 fill:#ffffff,stroke:#000000,color:#000000
    style C2 fill:#ffffff,stroke:#000000,color:#000000
    style D1 fill:#ffffff,stroke:#000000,color:#000000
    style D2 fill:#ffffff,stroke:#000000,color:#000000
    style D3 fill:#ffffff,stroke:#000000,color:#000000
    style D4 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🔄 Provider Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Unregistered
    Unregistered --> Active: register_provider() 💰

    Active --> Inactive: set_active(false)
    Inactive --> Active: set_active(true)

    Inactive --> [*]: withdraw_stake()

    note right of Active
        Provider accepting jobs
        Can update profile
        Can add stake
    end note

    note right of Inactive
        Provider not accepting jobs
        Can withdraw stake
        Can reactivate
    end note
```

---

## 📋 Function Details

### 💰 register_provider (Payable)

```mermaid
graph LR
    A[endpoint: String<br/>compute_units: u64<br/>hourly_rate: U256<br/>+ STAKE] --> B[register_provider]
    B --> C{stake ≥ min_stake?<br/>Not registered?}
    C -->|✅ Yes| D[Create Profile<br/>Active = true<br/>Reputation = 100]
    C -->|❌ No| E[Return false]
    D --> F[Emit ProviderRegistered]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

**Requirements:**

- `transferred_value >= min_stake`
- Provider address not already registered

---

### 👤 update_provider (Provider Only)

```mermaid
graph LR
    A[endpoint: String<br/>hourly_rate: U256] --> B[update_provider]
    B --> C{Caller = Provider?<br/>Provider exists?}
    C -->|✅ Yes| D[Update endpoint<br/>Update rate]
    C -->|❌ No| E[Return false]
    D --> F[Emit ProviderUpdated]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

---

### 🔄 set_active (Provider Only)

```mermaid
graph LR
    A[is_active: bool] --> B[set_active]
    B --> C{Caller = Provider?<br/>Provider exists?}
    C -->|✅ Yes| D[Update is_active]
    C -->|❌ No| E[Return false]
    D --> F[Emit ProviderActiveChanged]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

---

### 💰 add_stake (Provider Only, Payable)

```mermaid
graph LR
    A[+ ADDITIONAL_STAKE] --> B[add_stake]
    B --> C{Caller = Provider?<br/>amount > 0?}
    C -->|✅ Yes| D[Increase stake]
    C -->|❌ No| E[Return false]
    D --> F[Emit StakeAdded]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

---

### 💸 withdraw_stake (Provider/Admin Only)

```mermaid
graph LR
    A[amount: U256] --> B[withdraw_stake]
    B --> C{Provider inactive<br/>OR caller = admin?<br/>stake ≥ amount?}
    C -->|✅ Yes| D[Transfer to provider<br/>Reduce stake]
    C -->|❌ No| E[Return false]
    D --> F[Emit StakeWithdrawn]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

**Requirements:**

- Provider must be inactive OR caller is admin
- Sufficient stake available

---

### ⚙️ set_reputation (Admin Only)

```mermaid
graph LR
    A[provider: H160<br/>score: u32] --> B[set_reputation]
    B --> C{Caller = Admin?<br/>Provider exists?}
    C -->|✅ Yes| D[Update reputation]
    C -->|❌ No| E[Return false]
    D --> F[Emit ReputationUpdated]
    D --> G[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

---

### ⚙️ set_min_stake (Admin Only)

```mermaid
graph LR
    A[new_min_stake: U256] --> B[set_min_stake]
    B --> C{Caller = Admin?}
    C -->|✅ Yes| D[Update min_stake]
    C -->|❌ No| E[Return false]
    D --> F[Return true]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📊 Query Functions

```mermaid
graph TB
    A[get_provider] -->|provider: H160| A1[Returns: Option ProviderProfile]
    B[get_admin] --> B1[Returns: H160 admin]
    C[get_provider_count] --> C1[Returns: u64 total]
    D[get_min_stake] --> D1[Returns: U256 minimum]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style A1 fill:#ffffff,stroke:#000000,color:#000000
    style B1 fill:#ffffff,stroke:#000000,color:#000000
    style C1 fill:#ffffff,stroke:#000000,color:#000000
    style D1 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📤 Events

```mermaid
graph LR
    subgraph Events
        E1[ProviderRegistered<br/>provider, stake, compute_units]
        E2[ProviderUpdated<br/>provider, endpoint, hourly_rate]
        E3[ProviderActiveChanged<br/>provider, is_active]
        E4[StakeAdded<br/>provider, amount]
        E5[StakeWithdrawn<br/>provider, amount]
        E6[ReputationUpdated<br/>provider, score]
    end

    style E1 fill:#ffffff,stroke:#000000,color:#000000
    style E2 fill:#ffffff,stroke:#000000,color:#000000
    style E3 fill:#ffffff,stroke:#000000,color:#000000
    style E4 fill:#ffffff,stroke:#000000,color:#000000
    style E5 fill:#ffffff,stroke:#000000,color:#000000
    style E6 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🏗️ Data Types

### ProviderProfile Structure

```mermaid
classDiagram
    class ProviderProfile {
        +H160 provider
        +String endpoint
        +u64 compute_units
        +U256 hourly_rate
        +u64 registered_at
        +bool is_active
        +U256 stake
        +u32 reputation_score
    }

    note for ProviderProfile "Immutable:\n- provider\n- registered_at\n\nMutable:\n- endpoint\n- compute_units\n- hourly_rate\n- is_active\n- stake\n- reputation_score"
```

---

## ⚙️ Access Control

```mermaid
graph TB
    subgraph "🔐 Permissions"
        A[Provider] -->|Can call| A1[register_provider 💰]
        A -->|Can call| A2[update_provider]
        A -->|Can call| A3[set_active]
        A -->|Can call| A4[add_stake 💰]
        A -->|Can call| A5[withdraw_stake inactive]

        B[Admin] -->|Can call| B1[set_reputation]
        B -->|Can call| B2[set_min_stake]
        B -->|Can call| B3[withdraw_stake any]

        C[Anyone] -->|Can call| C1[get_provider]
        C -->|Can call| C2[get_admin]
        C -->|Can call| C3[get_provider_count]
        C -->|Can call| C4[get_min_stake]
    end

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🎯 Registration Flow

```mermaid
sequenceDiagram
    participant Provider
    participant Contract
    participant Admin

    Provider->>Contract: register_provider(endpoint, units, rate) 💰
    Contract-->>Provider: Registered ✅ (reputation = 100)

    Provider->>Contract: update_provider(new_endpoint, new_rate)
    Contract-->>Provider: Updated ✅

    Provider->>Contract: add_stake() 💰
    Contract-->>Provider: Stake increased ✅

    Admin->>Contract: set_reputation(provider, score)
    Contract-->>Admin: Reputation updated ✅

    Provider->>Contract: set_active(false)
    Contract-->>Provider: Inactive ✅

    Provider->>Contract: withdraw_stake(amount)
    Contract->>Provider: Transfer funds 💸
    Contract-->>Provider: Withdrawn ✅
```

---

## 🔒 Constraints & Rules

```mermaid
graph TB
    subgraph "✅ Registration"
        A1[Must stake ≥ min_stake]
        A2[Cannot register twice]
        A3[Initial reputation = 100]
    end

    subgraph "✅ Updates"
        B1[Only provider can update own profile]
        B2[Only provider can toggle active status]
        B3[Only provider can add stake]
    end

    subgraph "✅ Withdrawals"
        C1[Provider must be inactive to withdraw]
        C2[Admin can withdraw from any provider]
        C3[Cannot withdraw more than current stake]
    end

    subgraph "✅ Admin Actions"
        D1[Only admin can set reputation]
        D2[Only admin can update min_stake]
    end

    style A1 fill:#ffffff,stroke:#000000,color:#000000
    style A2 fill:#ffffff,stroke:#000000,color:#000000
    style A3 fill:#ffffff,stroke:#000000,color:#000000
    style B1 fill:#ffffff,stroke:#000000,color:#000000
    style B2 fill:#ffffff,stroke:#000000,color:#000000
    style B3 fill:#ffffff,stroke:#000000,color:#000000
    style C1 fill:#ffffff,stroke:#000000,color:#000000
    style C2 fill:#ffffff,stroke:#000000,color:#000000
    style C3 fill:#ffffff,stroke:#000000,color:#000000
    style D1 fill:#ffffff,stroke:#000000,color:#000000
    style D2 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📊 Reputation System

```mermaid
graph LR
    A[New Provider] -->|Initial| B[Score: 100]
    B -->|Good Performance| C[Score: 101-1000]
    B -->|Poor Performance| D[Score: 0-99]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
```

**Reputation Range:** 0 - 1000 (u32)

- **100**: Default for new providers
- **0-99**: Below average
- **100-199**: Average
- **200+**: Above average

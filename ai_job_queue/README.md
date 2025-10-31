# 📋 AI Job Queue Contract

Smart contract for managing AI compute job submissions and lifecycle tracking in the PolkadotAiMesh network.

**Status:** ✅ Builds and compiles  
**Environment:** EVM-like primitives (H160 addresses, U256 balances)

---

## 🏗️ Constructor

```mermaid
graph LR
    A[new] -->|min_budget: U256| B[Initialize job queue<br/>Deployer becomes owner]
    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📝 Contract Functions

```mermaid
graph TB
    subgraph "💰 Payable Functions"
        A1[submit_job<br/>💵 Creates new job with payment]
    end

    subgraph "👤 Owner Functions"
        B1[assign_provider<br/>Assign compute provider to job]
        B2[cancel_job<br/>Cancel job if not completed]
        B3[set_min_budget<br/>Update minimum budget requirement]
    end

    subgraph "🔧 Provider Functions"
        C1[mark_in_progress<br/>Start job execution]
        C2[mark_completed<br/>Finish job with results]
    end

    subgraph "📊 Query Functions"
        D1[get_job<br/>Retrieve job details]
        D2[get_job_counter<br/>Total jobs count]
        D3[get_min_budget<br/>Minimum budget value]
    end

    style A1 fill:#ffffff,stroke:#000000,color:#000000
    style B1 fill:#ffffff,stroke:#000000,color:#000000
    style B2 fill:#ffffff,stroke:#000000,color:#000000
    style B3 fill:#ffffff,stroke:#000000,color:#000000
    style C1 fill:#ffffff,stroke:#000000,color:#000000
    style C2 fill:#ffffff,stroke:#000000,color:#000000
    style D1 fill:#ffffff,stroke:#000000,color:#000000
    style D2 fill:#ffffff,stroke:#000000,color:#000000
    style D3 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🔄 Job Lifecycle Flow

```mermaid
stateDiagram-v2
    [*] --> Registered: submit_job() 💰
    Registered --> Assigned: assign_provider() 👤
    Assigned --> InProgress: mark_in_progress() 🔧
    InProgress --> Completed: mark_completed() 🔧

    Registered --> Cancelled: cancel_job() 👤
    Assigned --> Cancelled: cancel_job() 👤

    Completed --> [*]
    Cancelled --> [*]

    note right of Registered
        Job created with budget
        Status: Registered
    end note

    note right of Assigned
        Provider assigned
        Status: Assigned
    end note

    note right of InProgress
        Provider working
        Status: InProgress
    end note

    note right of Completed
        Job finished
        Result hash stored
    end note
```

---

## 📋 Function Details

### 💰 submit_job (Payable)

```mermaid
graph LR
    A[Input Parameters] -->|model_ref: String<br/>data_ref: String<br/>deadline: u32<br/>privacy_required: bool<br/>+ PAYMENT| B[submit_job]
    B -->|Validation| C{value ≥ min_budget?<br/>deadline > block?}
    C -->|✅ Yes| D[Create Job]
    C -->|❌ No| E[Return false]
    D --> F[Emit JobSubmitted]
    D --> G[Return job_id: u128]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style D fill:#ffffff,stroke:#000000,color:#000000
    style E fill:#ffffff,stroke:#000000,color:#000000
    style F fill:#ffffff,stroke:#000000,color:#000000
    style G fill:#ffffff,stroke:#000000,color:#000000
```

**Requirements:**

- `transferred_value >= min_budget`
- `deadline > current_block`

---

### 👤 assign_provider (Owner Only)

```mermaid
graph LR
    A[job_id: u128<br/>provider: H160] --> B[assign_provider]
    B --> C{Owner?<br/>Status = Registered?}
    C -->|✅ Yes| D[Set provider<br/>Status → Assigned]
    C -->|❌ No| E[Return false]
    D --> F[Emit JobAssigned]
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

### 🔧 mark_in_progress (Provider Only)

```mermaid
graph LR
    A[job_id: u128] --> B[mark_in_progress]
    B --> C{Caller = Provider?<br/>Status = Assigned?}
    C -->|✅ Yes| D[Status → InProgress]
    C -->|❌ No| E[Return false]
    D --> F[Emit JobStatusChanged]
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

### 🔧 mark_completed (Provider Only)

```mermaid
graph LR
    A[job_id: u128<br/>result_hash: String] --> B[mark_completed]
    B --> C{Caller = Provider?<br/>Status = InProgress?}
    C -->|✅ Yes| D[Store result_hash<br/>Status → Completed]
    C -->|❌ No| E[Return false]
    D --> F[Emit JobCompleted]
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

### 👤 cancel_job (Owner Only)

```mermaid
graph LR
    A[job_id: u128] --> B[cancel_job]
    B --> C{Owner?<br/>Status ≠ Completed?}
    C -->|✅ Yes| D[Status → Cancelled]
    C -->|❌ No| E[Return false]
    D --> F[Emit JobCancelled]
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

## 📊 Query Functions

```mermaid
graph TB
    A[get_job] -->|job_id: u128| A1[Returns: Option Job]
    B[get_job_counter] --> B1[Returns: u128 total jobs]
    C[get_min_budget] --> C1[Returns: U256 minimum budget]

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
    style A1 fill:#ffffff,stroke:#000000,color:#000000
    style B1 fill:#ffffff,stroke:#000000,color:#000000
    style C1 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 📤 Events

```mermaid
graph LR
    subgraph Events
        E1[JobSubmitted<br/>job_id, owner, budget]
        E2[JobAssigned<br/>job_id, provider]
        E3[JobStatusChanged<br/>job_id, new_status]
        E4[JobCompleted<br/>job_id, provider, result_hash]
        E5[JobCancelled<br/>job_id]
    end

    style E1 fill:#ffffff,stroke:#000000,color:#000000
    style E2 fill:#ffffff,stroke:#000000,color:#000000
    style E3 fill:#ffffff,stroke:#000000,color:#000000
    style E4 fill:#ffffff,stroke:#000000,color:#000000
    style E5 fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🏗️ Data Types

### Job Structure

```mermaid
classDiagram
    class Job {
        +u128 id
        +H160 owner
        +String model_ref
        +String data_ref
        +U256 budget
        +JobStatus status
        +Option~H160~ assigned_provider
        +u32 deadline
        +bool privacy_required
    }

    class JobStatus {
        <<enumeration>>
        Registered
        Assigned
        InProgress
        Completed
        Cancelled
    }

    Job --> JobStatus
```

---

## ⚙️ Access Control

```mermaid
graph TB
    subgraph "🔐 Permissions"
        A[Owner/Job Creator] -->|Can call| A1[assign_provider]
        A -->|Can call| A2[cancel_job]
        A -->|Can call| A3[set_min_budget]

        B[Assigned Provider] -->|Can call| B1[mark_in_progress]
        B -->|Can call| B2[mark_completed]

        C[Anyone] -->|Can call| C1[submit_job 💰]
        C -->|Can call| C2[get_job]
        C -->|Can call| C3[get_job_counter]
        C -->|Can call| C4[get_min_budget]
    end

    style A fill:#ffffff,stroke:#000000,color:#000000
    style B fill:#ffffff,stroke:#000000,color:#000000
    style C fill:#ffffff,stroke:#000000,color:#000000
```

---

## 🔒 Constraints & Rules

✅ **Submission Requirements:**

- Payment must be ≥ `min_budget`
- Deadline must be in the future

✅ **State Transitions:**

- Jobs must follow: Registered → Assigned → InProgress → Completed
- Cancellation only allowed before Completed state

✅ **Provider Actions:**

- Only assigned provider can mark progress/completion
- Provider cannot be changed after assignment

✅ **Owner Actions:**

- Owner can cancel job anytime before completion
- Only owner can assign initial provider

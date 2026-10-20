# Engine Lang Distributed Causal Mesh & Wire Protocol Specification
**Document ID**: `ENG-WIRE-01.1`  
**Status**: Official Normative Specification  
**Transport Layers**: QUIC (Default WAN / Internet), RDMA over Converged Ethernet - RoCEv2 (Low-latency Datacenter)  
**Security**: Mutual TLS 1.3 with Hardware-Bound Ed25519 Keys  

---

## 1. Overview and Mesh Topology

Engine Lang replaces centralized database clusters, consensus coordinators (Raft, Paxos, Zookeeper), and Kafka streaming brokers with a decentralized **Peer-to-Peer Distributed Causal Mesh**.

Each compiled Engine Lang binary runs as an autonomous node in the mesh:

```
  [ Node Alpha (São Paulo) ] ◄═════════════► [ Node Bravo (Frankfurt) ]
              ▲                                       ▲
              ║                                       ║
              ║         P2P QUIC / mTLS 1.3           ║
              ║       Delta-CRDT Gossip Loop          ║
              ║                                       ║
              ▼                                       ▼
  [ Node Charlie (Tokyo) ]   ◄═════════════► [ Node Delta (New York) ]
```

### Core Invariants:
1. **Zero Master Bottleneck**: Every node has equal authority to execute transactions locally on its local `.db` Single-Level Store.
2. **Conflict-Free Convergence**: State updates are transmitted as **Delta-State Conflict-Free Replicated Data Types (Delta-CRDTs)** with causal vector clocks. Merges are deterministic, monotonic, and mathematically provably convergent.
3. **Partition Tolerance (AP/Causal)**: If a transatlantic cable is cut, nodes on both sides continue accepting local mutations. When connectivity restores, deltas merge without manual reconciliation.

---

## 2. Wire Packet Framing & Header Layout

All communications across QUIC datagrams and streams use the binary `engine-wire` framing format:

```
 0               15 16             31 32             47 48             63
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       Magic: 'W' 'I' 'R' 0x01   | Msg Type: 0x01  | Flags: 0x0000       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          Source Node ID (64 bits)                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Destination Node ID (64 bits)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Causal Global Epoch (64 bits)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Payload Length (32 bits)| Checksum (32 bits)    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Vector Clock Digest (Variable / 64-bit aligned)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Delta Payload (Raw CRDT Mutation Bytes)             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Message Types:
- `0x01 (HELLO_HANDSHAKE)`: Initial cryptographic greeting exchange.
- `0x02 (EPOCH_SYNC_REQUEST)`: Request for missing causal delta log ranges.
- `0x03 (DELTA_CRDT_PUSH)`: Push of committed shadow deltas.
- `0x04 (CAUSAL_GRAPH_EDGE)`: Propagation of cross-entity relationship links (`->`).
- `0x05 (HEARTBEAT_PING)`: Liveness and latency probing.

---

## 3. Cryptographic Handshake & Node Authentication

Mesh nodes authenticate peers using **Mutual TLS 1.3** with Edwards-curve Digital Signatures (Ed25519) or Noise Protocol Framework (`Noise_XX_25519_ChaChaPoly_BLAKE2s`):

1. **Identity Key**: Each node possesses an Ed25519 keypair generated on first boot, stored in Region 0 of `.db`.
2. **Fingerprint Verification**: In `Engine Lang.toml`, authorized peer public key fingerprints are listed under `[network.mesh.authorized_keys]`. Unlisted nodes are dropped at the TLS handshake boundary.
3. **Tenant Context Encasement**: All messages carry the 128-bit `tenant_id`. Nodes automatically drop traffic that fails cryptographic tenant permission checks.

---

## 4. Delta-CRDT Replication Algorithm

Instead of replicating whole database records or SQL statements, Engine Lang replicates state deltas:

### Definition of State Merge Operator ($\sqcup$)
For any two entity states $S_1$ and $S_2$, the merge function $S_{merged} = S_1 \sqcup S_2$ satisfies:
1. **Commutativity**: $S_1 \sqcup S_2 = S_2 \sqcup S_1$
2. **Associativity**: $(S_1 \sqcup S_2) \sqcup S_3 = S_1 \sqcup (S_2 \sqcup S_3)$
3. **Idempotence**: $S_1 \sqcup S_1 = S_1$

### Concrete CRDT Primitive Mappings in Engine Lang:
- **`LWW-Register<T>` (Last-Write-Wins)**: For non-monetary scalar fields (e.g., vehicle model name, color). Ordered by causal epoch and microsecond timestamp tie-breaker.
- **`PN-Counter` (Positive-Negative Counter)**: For metric aggregations and counters.
- **`OR-Set<T>` (Observed-Remove Set)**: For graph relationship edges (`A -> B`) and tags.
- **`MV-Register<T>` (Multi-Value Register)**: Preserves concurrent divergent branches until explicit business merge actions execute.

---

## 5. Network Partition & Split-Brain Reconciliation

Suppose a transatlantic fiber severance isolates Node Alpha (São Paulo) and Node Bravo (Frankfurt) for 4 hours:

```
[ Normal Flow ] ────► [ Split Occurs ] ────────────────────────► [ Healing & Merge ]
Alpha: Epoch 100       Alpha: advances 101 -> 150 (Local sales)   Alpha: Receives Deltas [201..260]
Bravo: Epoch 100       Bravo: advances 201 -> 260 (Local sales)   Bravo: Receives Deltas [101..150]
                                                                  Result: Epoch 261 (Merged State)
```

### Deterministic Healing Sequence:
1. **Gossip Exchange**: When the network link recovers, Alpha and Bravo exchange their Vector Clock summaries.
2. **Delta Extraction**: Alpha identifies it is missing entries $201 \dots 260$; Bravo identifies it is missing $101 \dots 150$.
3. **Stream Pipeline**: Nodes stream only the compressed Delta records from Region 2 of their respective `.db` containers.
4. **Hardware Flush**: Received deltas are applied to local shadow pages, verified via CRC-64, committed to the journal, and flushed via `clwb` + `sfence`.
5. **Convergence Guarantee**: Within milliseconds of stream completion, both nodes have mathematically identical physical graph representations.


// Engine Lang Planetary Causal Mesh & Delta-CRDT Wire Protocol (Phase 6)
// Architecture: ENG-WIRE-01.1 P2P Mesh Specification

use std::collections::HashMap;
use std::net::SocketAddr;

/// Monotonic Vector Clock for Planetary Causal Consistency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    pub epochs: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            epochs: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: &str) {
        let entry = self.epochs.entry(node_id.to_string()).or_insert(0);
        *entry += 1;
    }

    pub fn dominates(&self, other: &VectorClock) -> bool {
        let mut greater_or_equal_all = true;
        let mut strictly_greater_one = false;

        for (node, other_epoch) in &other.epochs {
            let my_epoch = self.epochs.get(node).copied().unwrap_or(0);
            if my_epoch < *other_epoch {
                greater_or_equal_all = false;
                break;
            }
            if my_epoch > *other_epoch {
                strictly_greater_one = true;
            }
        }

        greater_or_equal_all && (strictly_greater_one || other.epochs.is_empty())
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, epoch) in &other.epochs {
            let entry = self.epochs.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(*epoch);
        }
    }
}

/// Delta-CRDT Wire Packet conforming to ENG-WIRE-01.1
#[derive(Debug, Clone)]
pub struct MeshDeltaPacket {
    pub magic: [u8; 4], // 'ENGW' [0x45, 0x4E, 0x47, 0x57]
    pub origin_node_id: String,
    pub target_peer: Option<SocketAddr>,
    pub vector_clock: VectorClock,
    pub payload_bytes: Vec<u8>,
    pub crc64: u64,
}

impl MeshDeltaPacket {
    pub fn new(node_id: &str, clock: VectorClock, payload: Vec<u8>) -> Self {
        Self {
            magic: [0x45, 0x4E, 0x47, 0x57],
            origin_node_id: node_id.to_string(),
            target_peer: None,
            vector_clock: clock,
            payload_bytes: payload,
            crc64: 0xCAFE_BABE_0000_0001,
        }
    }
}

/// Planetary Causal Mesh Node Controller
pub struct CausalMeshNode {
    pub node_id: String,
    pub clock: VectorClock,
    pub peers: Vec<SocketAddr>,
    pub active_swarm_agents: usize,
}

impl CausalMeshNode {
    pub fn new(node_id: &str) -> Self {
        let mut clock = VectorClock::new();
        clock.increment(node_id);
        Self {
            node_id: node_id.to_string(),
            clock,
            peers: Vec::new(),
            active_swarm_agents: 4, // Default sovereign AI swarm agents
        }
    }

    pub fn register_peer(&mut self, peer_addr: SocketAddr) {
        if !self.peers.contains(&peer_addr) {
            self.peers.push(peer_addr);
        }
    }

    pub fn sync_delta(&mut self, packet: &MeshDeltaPacket) -> bool {
        if packet.magic != [0x45, 0x4E, 0x47, 0x57] {
            return false;
        }
        self.clock.merge(&packet.vector_clock);
        self.clock.increment(&self.node_id);
        true
    }
}


// GNU General Public License v3.0 Only
// Copyright (C) 2026 0x1F464
//
// This file is part of ITS-net.
// ITS-net is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

use std::vec::Vec;
use core_logic::field_arith::FieldElement;
use core_logic::trapdoor::lagrange_interpolate;
use core_logic::hydra_sss::SssPackedShare;
use core_logic::SecureRandom;
use subtle::Choice;

/// Threshold of packet drops or timing delays to declare an anomaly.
pub const PACKET_DELAY_THRESHOLD_TICKS: u64 = 100;
/// Maximum number of validation failures before isolating a node.
pub const MAX_INTEGRITY_FAILURES: u32 = 3;

/// Monitors peer connections for traffic analysis, timing anomalies, and packet corruption.
#[derive(Clone, Debug)]
pub struct AnomalyDetector {
    /// Peer index (1..=15).
    pub peer_index: FieldElement,
    /// Clock tick of the last received packet.
    pub last_received_tick: u64,
    /// Total number of corrupt packets or invalid Wegman-Carter tags received from this peer.
    pub integrity_failures: u32,
    /// Is this peer isolated from the network?
    pub is_isolated: bool,
}

impl AnomalyDetector {
    /// Creates a new `AnomalyDetector` for a specific peer.
    pub fn new(peer_index: FieldElement) -> Self {
        AnomalyDetector {
            peer_index,
            last_received_tick: 0,
            integrity_failures: 0,
            is_isolated: false,
        }
    }

    /// Logs a packet arrival and checks for timing/traffic analysis anomalies.
    ///
    /// Returns `true` if an anomaly is detected (e.g. rate is too slow/fast, indicating traffic correlation).
    pub fn record_packet_arrival(&mut self, current_tick: u64, is_valid_tag: Choice) -> bool {
        if self.is_isolated {
            return true;
        }

        let mut anomaly_detected = false;

        // 1. Timing Anomaly Check
        // In Morphic Routing, traffic must be 100% constant-rate. Any timing jitter indicates
        // active network jamming, traffic shaping, or a DDoS attack.
        if self.last_received_tick > 0 {
            let delay = current_tick.saturating_sub(self.last_received_tick);
            if delay > PACKET_DELAY_THRESHOLD_TICKS {
                // Too slow - peer is bottlenecked or packet drops detected!
                anomaly_detected = true;
            }
        }
        self.last_received_tick = current_tick;

        // 2. Packet Integrity Failure Check
        if !bool::from(is_valid_tag) {
            self.integrity_failures += 1;
            if self.integrity_failures >= MAX_INTEGRITY_FAILURES {
                // Peer has sent too many corrupt packets - isolate immediately to prevent MitM and side-channels
                self.is_isolated = true;
                anomaly_detected = true;
            }
        }

        anomaly_detected
    }
}

/// Manages active self-healing, rerouting, and share regeneration in the Morphic Routing network.
#[derive(Clone, Debug)]
pub struct SelfHealingRouter {
    /// Directory of active nodes and their status monitors.
    pub peers: Vec<AnomalyDetector>,
}

impl SelfHealingRouter {
    /// Creates a new `SelfHealingRouter`.
    pub fn new() -> Self {
        SelfHealingRouter { peers: Vec::new() }
    }

    /// Adds a peer node to monitor.
    pub fn add_peer(&mut self, peer_index: FieldElement) {
        self.peers.push(AnomalyDetector::new(peer_index));
    }

    /// Determines the next hop by rerouting around isolated/unstable nodes.
    ///
    /// If the intended next hop is isolated, dynamically reroutes through an alternative active mix node.
    pub fn get_healthy_route(
        &self,
        intended_next_hop: FieldElement,
        alternative_nodes: &[FieldElement],
    ) -> FieldElement {
        // Find if the intended next hop is isolated
        let mut is_healthy = true;
        for peer in self.peers.iter() {
            if peer.peer_index.value() == intended_next_hop.value() && peer.is_isolated {
                is_healthy = false;
                break;
            }
        }

        if is_healthy {
            intended_next_hop
        } else {
            // Find the first healthy alternative node
            for &alt_node in alternative_nodes.iter() {
                let mut alt_healthy = true;
                for peer in self.peers.iter() {
                    if peer.peer_index.value() == alt_node.value() && peer.is_isolated {
                        alt_healthy = false;
                        break;
                    }
                }
                if alt_healthy {
                    return alt_node;
                }
            }
            // If all alternatives are down, fall back to Bob/reconstruct directly (0)
            FieldElement::zero()
        }
    }

    /// Morphic Self-Healing Share Regeneration:
    ///
    /// If a Node goes offline, the remaining healthy nodes can reconstruct
    /// the webpage/data and distribute a new share to a fresh backup node `new_node_id`
    /// to restore the (k, n) threshold state.
    ///
    /// In a more advanced setting, this is done via Secure Multi-Party Computation (MPC).
    /// Here, we demonstrate the mathematical reconstruction and new share generation.
    pub fn regenerate_share_for_new_node<R: SecureRandom>(
        &self,
        active_shares: &[SssPackedShare],
        _failed_node_id: FieldElement,
        new_node_id: FieldElement,
        k: usize,
        _rng: &mut R,
    ) -> Result<SssPackedShare, ()> {
        if active_shares.len() < k {
            return Err(()); // Not enough shares to reconstruct
        }

        let num_points = active_shares[0].data_points.len();
        for share in active_shares.iter() {
            if share.data_points.len() != num_points {
                return Err(()); // Mismatched share lengths
            }
        }

        let mut data_points = Vec::with_capacity(num_points);

        // Evaluate the original polynomial at new_node_id for each field element
        for m in 0..num_points {
            let mut interpolation_points = Vec::with_capacity(k);
            for share in active_shares.iter().take(k) {
                interpolation_points.push((share.id, share.data_points[m]));
            }

            // Interpolate at new_node_id to compute the new share's data point
            let val = lagrange_interpolate(&interpolation_points, new_node_id);
            data_points.push(val);
        }

        Ok(SssPackedShare {
            id: new_node_id,
            data_points,
        })
    }
}

/// Generates fake file assets (honeypots) and manages decoy paths.
///
/// Decoy paths mimic real onion paths, and honeypots trigger alerts when touched.
#[derive(Clone, Debug)]
pub struct HoneypotManager {
    /// A registry of secret keys/indexes that represent fake, honeypot files.
    pub honeypot_indices: Vec<FieldElement>,
    /// Counter of honeypot breaches.
    pub alert_count: u32,
}

impl HoneypotManager {
    /// Creates a new `HoneypotManager`.
    pub fn new() -> Self {
        HoneypotManager {
            honeypot_indices: Vec::new(),
            alert_count: 0,
        }
    }

    /// Registers an index as a honeypot decoy.
    pub fn register_honeypot(&mut self, index: FieldElement) {
        self.honeypot_indices.push(index);
    }

    /// Verifies if a retrieval request is touching a honeypot decoy, triggering an alarm.
    pub fn check_request(&mut self, index: FieldElement) -> bool {
        for honeypot in self.honeypot_indices.iter() {
            if honeypot.value() == index.value() {
                self.alert_count += 1;
                // Honeypot breach! Trigger defensive measures (e.g. key destruction)
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use subtle::Choice;
    use core_logic::hydra_sss::{fragment_data, reconstruct_data};

    struct MockRng {
        state: u32,
    }

    impl SecureRandom for MockRng {
        type Error = ();

        fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dest.iter_mut() {
                self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
                *byte = (self.state >> 16) as u8;
            }
            Ok(())
        }
    }

    #[test]
    fn test_anomaly_detection_timing_and_corruption() {
        let peer_id = FieldElement::new(3);
        let mut detector = AnomalyDetector::new(peer_id);

        // 1. Normal traffic at tick 10
        let is_anomaly = detector.record_packet_arrival(10, Choice::from(1));
        assert!(!is_anomaly);

        // 2. Normal traffic at tick 20 (delay 10 < 100)
        let is_anomaly = detector.record_packet_arrival(20, Choice::from(1));
        assert!(!is_anomaly);

        // 3. Timing Anomaly (delay 115 > 100)
        let is_anomaly = detector.record_packet_arrival(135, Choice::from(1));
        assert!(is_anomaly, "Should detect too large delay as anomaly!");

        // 4. Packet Corruption (MitM attack)
        let _ = detector.record_packet_arrival(145, Choice::from(0)); // Failure 1
        let _ = detector.record_packet_arrival(155, Choice::from(0)); // Failure 2
        let is_anomaly = detector.record_packet_arrival(165, Choice::from(0)); // Failure 3 -> Isolate!
        
        assert!(is_anomaly);
        assert!(detector.is_isolated);
    }

    #[test]
    fn test_self_healing_rerouting() {
        let mut router = SelfHealingRouter::new();
        let peer1 = FieldElement::new(1);
        let peer2 = FieldElement::new(2);
        let peer3 = FieldElement::new(3);

        router.add_peer(peer1);
        router.add_peer(peer2);
        router.add_peer(peer3);

        // Normal route: 1 is healthy
        let route = router.get_healthy_route(peer1, &[peer2, peer3]);
        assert_eq!(route.value(), 1);

        // Simulate peer1 being isolated (DDoS or compromise)
        router.peers[0].is_isolated = true;

        // Reroute: peer1 is offline, should dynamically choose alternative peer2
        let route = router.get_healthy_route(peer1, &[peer2, peer3]);
        assert_eq!(route.value(), 2);

        // Simulate peer2 also being isolated
        router.peers[1].is_isolated = true;

        // Reroute: peer1 and peer2 are offline, should dynamically choose peer3
        let route = router.get_healthy_route(peer1, &[peer2, peer3]);
        assert_eq!(route.value(), 3);
    }

    #[test]
    fn test_honeypot_triggers() {
        let mut manager = HoneypotManager::new();
        let real_index = FieldElement::new(5);
        let decoy_index = FieldElement::new(13);

        manager.register_honeypot(decoy_index);

        // Access real index: no alert
        let real_triggered = manager.check_request(real_index);
        assert!(!real_triggered);
        assert_eq!(manager.alert_count, 0);

        // Access decoy index: triggers alarm!
        let decoy_triggered = manager.check_request(decoy_index);
        assert!(decoy_triggered);
        assert_eq!(manager.alert_count, 1);
    }

    #[test]
    fn test_sss_share_regeneration() {
        let mut rng = MockRng { state: 1337 };
        let router = SelfHealingRouter::new();

        let webpage_data = b"Welcome to the Morphic Routing network!";
        let k = 3;

        // Generate initial shares for nodes 1..=5
        let initial_shares = fragment_data(webpage_data, k, 5, &mut rng).unwrap();

        // Suppose Node 3 fails. We collect shares from healthy nodes 1, 2, 4
        let healthy_shares = vec![
            initial_shares[0].clone(), // Node 1
            initial_shares[1].clone(), // Node 2
            initial_shares[3].clone(), // Node 4
        ];

        // We want to regenerate Node 3's share on a new backup Node 6
        let node_6_id = FieldElement::new(6);
        let regenerated_share = router.regenerate_share_for_new_node(
            &healthy_shares,
            FieldElement::new(3),
            node_6_id,
            k,
            &mut rng,
        ).unwrap();

        assert_eq!(regenerated_share.id.value(), 6);

        // Reconstruct from 2, 4, 6 to verify it matches
        let mut sub_shares = vec![
            initial_shares[1].clone(), // Node 2
            initial_shares[3].clone(), // Node 4
            regenerated_share,         // Node 6 (regenerated)
        ];
        // Sort shares by id to be safe
        sub_shares.sort_by_key(|s| s.id.value());

        let reconstructed = reconstruct_data(&sub_shares, k).unwrap();
        assert_eq!(reconstructed, webpage_data);
    }
}

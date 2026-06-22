//! Witness consensus — k-of-n A2′ harvest agreement (Lean: `WitnessConsensus.lean`).

/// Count witnesses in `members` that harvest `expected` at `epoch`.
pub fn witness_harvest_count(
    members: &[String],
    harvests: &[(String, Option<Vec<u8>>)],
    _epoch: u64,
    expected: &[u8],
) -> usize {
    members
        .iter()
        .filter(|m| {
            harvests
                .iter()
                .find(|(url, _)| url == *m)
                .and_then(|(_, cell)| cell.as_deref())
                .map(|c| c == expected)
                .unwrap_or(false)
        })
        .count()
}

/// k-of-n witness consensus at `epoch` for cell `expected`.
pub fn consensus_at_epoch(
    harvests: &[(String, Option<Vec<u8>>)],
    _epoch: u64,
    expected: &[u8],
    k: usize,
) -> bool {
    if k == 0 {
        return false;
    }
    let agreeing = harvests
        .iter()
        .filter(|(_, cell)| cell.as_deref().map(|c| c == expected).unwrap_or(false))
        .count();
    agreeing >= k
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(tag: u8) -> Vec<u8> {
        vec![tag; 16]
    }

    #[test]
    fn witness_consensus_at_epoch_k_of_n() {
        let expected = cell(1);
        let wrong = cell(2);
        let harvests = vec![
            ("w1".to_string(), Some(expected.clone())),
            ("w2".to_string(), Some(expected.clone())),
            ("w3".to_string(), Some(wrong)),
        ];
        assert!(consensus_at_epoch(
            &harvests
                .iter()
                .map(|(u, c)| (u.clone(), c.clone()))
                .collect::<Vec<_>>(),
            0,
            &expected,
            2
        ));
        assert!(!consensus_at_epoch(
            &harvests
                .iter()
                .map(|(u, c)| (u.clone(), c.clone()))
                .collect::<Vec<_>>(),
            0,
            &expected,
            3
        ));
    }

    #[test]
    fn witness_consensus_harvest_count() {
        let expected = cell(9);
        let members = vec!["w1".to_string(), "w2".to_string(), "w3".to_string()];
        let harvests = vec![
            ("w1".to_string(), Some(expected.clone())),
            ("w2".to_string(), Some(expected.clone())),
            ("w3".to_string(), None),
        ];
        assert_eq!(
            witness_harvest_count(&members, &harvests, 4, &expected),
            2
        );
    }
}

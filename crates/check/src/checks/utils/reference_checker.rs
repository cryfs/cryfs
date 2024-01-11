use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;

/// [ReferenceChecker] is useful for checking references in tree structures.
/// It remembers for each node id
/// - whether it was seen
/// - which other nodes referenced it
pub struct ReferenceChecker<NodeId, SeenInfo, ReferenceInfo>
where
    NodeId: Hash + PartialEq + Eq,
    SeenInfo: Clone,
{
    // `SeenInfo` is set if the node was *seen*.
    // `ReferenceInfo` remembers all references to the node.
    nodes: HashMap<NodeId, (Option<SeenInfo>, Vec<ReferenceInfo>)>,
}

impl<NodeId, SeenInfo, ReferenceInfo> ReferenceChecker<NodeId, SeenInfo, ReferenceInfo>
where
    NodeId: Hash + PartialEq + Eq,
    // TODO don't require `SeenInfo: Clone`
    SeenInfo: Clone,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    #[must_use]
    pub fn mark_as_seen(
        &mut self,
        node_id: NodeId,
        seen_info: SeenInfo,
    ) -> MarkAsSeenResult<SeenInfo> {
        match self.nodes.entry(node_id) {
            Entry::Occupied(mut entry) => {
                if let Some(ref prev_seen_info) = entry.get().0 {
                    // TODO This way of dealing with nodes that were already seen before kind of works but it still means the main runner loads and processes these
                    // nodes multiple times, including all of their subtree. It would be better if we had the runner itself handle this and just not call updates
                    // on nodes it has already processed.
                    MarkAsSeenResult::AlreadySeenBefore {
                        prev_seen_info: prev_seen_info.clone(),
                    }
                } else {
                    entry.get_mut().0 = Some(seen_info);
                    MarkAsSeenResult::NotSeenBeforeYet
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((Some(seen_info), vec![]));
                MarkAsSeenResult::NotSeenBeforeYet
            }
        }
    }

    pub fn mark_as_referenced(&mut self, node_id: NodeId, reference_info: ReferenceInfo) {
        match self.nodes.entry(node_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().1.push(reference_info);
            }
            Entry::Vacant(entry) => {
                entry.insert((None, vec![reference_info]));
            }
        }
    }

    // Returns a list of errors and a list of nodes that were processed without errors
    pub fn finalize(self) -> impl Iterator<Item = (NodeId, Option<SeenInfo>, Vec<ReferenceInfo>)> {
        self.nodes
            .into_iter()
            .map(|(node_id, (seen_info, references))| (node_id, seen_info, references))
    }
}

#[must_use]
pub enum MarkAsSeenResult<SeenInfo> {
    AlreadySeenBefore { prev_seen_info: SeenInfo },
    NotSeenBeforeYet,
}

use std::collections::{HashMap, hash_map::Entry};
use std::fmt::Debug;
use std::hash::Hash;

/// [ReferenceChecker] is useful for checking references in tree structures.
/// It remembers for each node id
/// - whether it was seen
/// - which other nodes referenced it
pub struct ReferenceChecker<NodeId, SeenInfo, ReferenceInfo>
where
    NodeId: Debug + Hash + PartialEq + Eq,
{
    // `SeenInfo` is set if the node was *seen*.
    // `ReferenceInfo` remembers all references to the node.
    nodes: HashMap<NodeId, (Option<SeenInfo>, Vec<ReferenceInfo>)>,
}

impl<NodeId, SeenInfo, ReferenceInfo> ReferenceChecker<NodeId, SeenInfo, ReferenceInfo>
where
    NodeId: Debug + Hash + PartialEq + Eq,
{
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn mark_as_seen(&mut self, node_id: NodeId, seen_info: SeenInfo) {
        match self.nodes.entry(node_id) {
            Entry::Occupied(mut entry) => {
                if entry.get().0.is_some() {
                    panic!(
                        "Node {node_id:?} was seen twice. The runner should guarantee that each node is only seen once.",
                        node_id = entry.key()
                    );
                }
                entry.get_mut().0 = Some(seen_info);
            }
            Entry::Vacant(entry) => {
                entry.insert((Some(seen_info), vec![]));
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

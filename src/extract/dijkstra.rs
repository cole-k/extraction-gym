use std::cmp::{Ordering, Reverse};

use priority_queue::PriorityQueue;

use super::*;

pub struct DijkstraExtractor;

#[derive(PartialEq, Eq, Hash)]
struct CostNode {
    cost: Cost,
    node: NodeId,
}

impl CostNode {
    fn new(cost: Cost, node: NodeId) -> Self {
        CostNode { cost, node }
    }
}

impl Ord for CostNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for CostNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// impl PartialEq for CostNode {
//     fn eq(&self, other: &Self) -> bool {
//         self.cost == other.cost && self.node == other.node
//     }
// }

impl Extractor for DijkstraExtractor {
    fn extract(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        class_parents: &IndexMap<ClassId, HashSet<NodeId>>,
    ) -> ExtractionResult {
        let mut result = ExtractionResult::default();
        let mut costs = IndexMap::<ClassId, Cost>::default();
        // We initialize the queue with the leaves
        // let mut queue: PriorityQueue<_, _> = egraph
        //     .classes()
        //     .iter()
        //     .
        //     ;

        let mut queue: PriorityQueue<ClassId, _> = egraph
            .classes()
            .iter()
            .flat_map(|(class_id, class)| {
                let min_node = class
                    .nodes
                    .iter()
                    .flat_map(|node_id| {
                        let node = &egraph.nodes[node_id];
                        if node.is_leaf() {
                            Some((node, node_id))
                        } else {
                            None
                        }
                    })
                    .min_by_key(|(node, _node_id)| node.cost);
                if let Some((min_node, min_node_id)) = min_node {
                    Some((
                        class_id.clone(),
                        Reverse(CostNode::new(min_node.cost, min_node_id.clone())),
                    ))
                } else {
                    None
                }
            })
            .collect();

        while let Some((class_id, Reverse(cost_node))) = queue.pop() {
            println!(
                "Dequeued node with cost:{} and e-node id: {}",
                cost_node.cost, cost_node.node
            );
            // let class_id = egraph.nid_to_cid(&cost_node.node);
            // We already will have found the minimum cost
            // if costs.contains_key(class_id) {
            //     continue;
            // }
            // This is the minimum cost and the e-node that has it
            costs.insert(class_id.clone(), cost_node.cost);
            result.choose(class_id.clone(), cost_node.node.clone());

            for node_id in class_parents.get(&class_id).into_iter().flatten() {
                let curr_class_id = egraph.nid_to_cid(node_id);
                if costs.contains_key(curr_class_id) {
                    continue;
                }

                let new_cost = result.node_sum_cost(egraph, &egraph[node_id], &costs);
                // Small optimization to avoid polluting the queue with nodes
                // for which we can't compute a cost yet
                // if new_cost < INFINITY {
                //     // println!("Enqueued node with cost:{} and e-node id: {}", new_cost, cost_node.node);
                //     queue.push(Reverse(CostNode::new(new_cost, node_id.clone())));
                // }
                println!(
                    "Trying to enqueue node with cost:{} and e-node id: {}",
                    new_cost, cost_node.node
                );
                if queue.get(curr_class_id).is_some() {
                    queue.change_priority_by(curr_class_id, |mut existing_cost_node| {
                        if new_cost < existing_cost_node.0.cost {
                            existing_cost_node.0.node = node_id.clone();
                            existing_cost_node.0.cost = new_cost;
                            println!(
                                "Enqueued node with cost:{} and e-node id: {}",
                                new_cost, cost_node.node
                            );
                        }
                    });
                } else {
                    queue.push(
                        curr_class_id.clone(),
                        Reverse(CostNode::new(new_cost, node_id.clone())),
                    );
                }
            }
        }

        // println!("result: {:?}", result.choices);

        result
    }
}

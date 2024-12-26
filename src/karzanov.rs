use std::collections::HashMap;

use crate::graph::{ArcId, GraphNetwork, NodeId};
use crate::utils::min;

#[derive(Debug, Clone)]
pub struct KarzanovNode {
    stack: Vec<(ArcId, u32)>,
    grouped: bool, // to group nodes by layers
}

impl KarzanovNode {
    pub fn new() -> Self {
        KarzanovNode {
            stack: Vec::new(),
            grouped: false,
        }
    }
}

#[derive(Debug)]
pub struct KarzanovArc {
    capacity: u32,
    flow: u32,
    open: bool,
}

impl KarzanovArc {
    pub fn new(capacity: u32) -> Self {
        KarzanovArc {
            capacity,
            flow: 0,
            open: true,
        }
    }
}

fn clean_network(network: &mut GraphNetwork<KarzanovNode, KarzanovArc>) {
    for node in &mut network.node_data {
        if let Some(node) = node {
            node.stack.clear();
            node.grouped = false;
        }
    }
    for arc in &mut network.arc_data {
        if let Some(arc) = arc {
            arc.flow = 0;
            arc.open = true;
        }
    }
}

fn grouping_nodes_by_layer(
    source_id: NodeId,
    sink_id: NodeId,
    network: &mut GraphNetwork<KarzanovNode, KarzanovArc>,
) -> Vec<Vec<NodeId>> {
    if !network.is_node_in(source_id) {
        panic!("Node does not exist");
    }
    // split into layers
    let mut layers: Vec<Vec<NodeId>> = vec![vec![source_id]];
    loop {
        let mut next_layer: Vec<NodeId> = Vec::new();
        // collect nodes which is connected to the last layer into `next_layer`
        for node_id in layers.last().unwrap() {
            let arcs: Vec<(NodeId, ArcId)> = network.from_node(node_id.clone()).collect();
            for (dist_node_id, _) in arcs {
                if network.data_of_node(dist_node_id).unwrap().grouped {
                    continue;
                }
                network.mut_data_of_node(dist_node_id).unwrap().grouped = true;
                next_layer.push(dist_node_id);
            }
        }
        // if there is no node to add, break
        if next_layer.len() == 0 {
            break;
        }
        layers.push(next_layer);
    }
    if layers.last().unwrap() != &vec![sink_id] {
        panic!("this type of problem cannot be solved with karzanov's algorithm")
    }
    // sort the layers by the connection
    // they should be ordered so that incoming-arc is calculated before the node is focused
    for layer in layers.iter_mut() {
        layer.sort_by(|a, b| {
            let a_lt_b = network.is_arc_in(a.clone(), b.clone());
            let b_lt_a = network.is_arc_in(b.clone(), a.clone());
            match (a_lt_b, b_lt_a) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
    }
    return layers;
}

fn incoming_flux_of_flow(
    node_id: NodeId,
    network: &GraphNetwork<KarzanovNode, KarzanovArc>,
) -> u32 {
    let mut incoming_flux = 0;
    for (_, arc_id) in network.into_node(node_id) {
        let arc = network.data_of_arc(arc_id).unwrap();
        incoming_flux += arc.flow;
    }
    return incoming_flux;
}

fn outgoing_flux_of_flow(
    node_id: NodeId,
    network: &GraphNetwork<KarzanovNode, KarzanovArc>,
) -> u32 {
    let mut outgoing_flux = 0;
    for (_, arc_id) in network.from_node(node_id) {
        let arc = network.data_of_arc(arc_id).unwrap();
        outgoing_flux += arc.flow;
    }
    return outgoing_flux;
}

/// maximize outgoing fluxes of preflows
fn maximize_outgoing(
    layers: &Vec<Vec<NodeId>>,
    mut start_layer: usize,
    network: &mut GraphNetwork<KarzanovNode, KarzanovArc>,
) {
    // saturate the first preflows
    let source_node_id = layers.first().unwrap().first().unwrap().clone();
    let arcs: Vec<(NodeId, ArcId)> = network.from_node(source_node_id).collect();
    for (node_id, arc_id) in arcs {
        let arc = network.mut_data_of_arc(arc_id).unwrap();
        let capacity = arc.capacity;
        let delta = capacity - arc.flow;
        arc.flow = capacity;
        let mut_node = network.mut_data_of_node(node_id).unwrap();
        if delta > 0 {
            mut_node.stack.push((arc_id, delta));
        }
    }
    // skip the first layer (== start node) / up to the start_layer
    if start_layer < 1 {
        start_layer = 1;
    }
    for layer in layers.iter().skip(start_layer) {
        for node_id in layer {
            let incoming_flux = incoming_flux_of_flow(node_id.clone(), network);
            let mut consumed_flux = 0;

            let arcs: Vec<(NodeId, ArcId)> = network.from_node(node_id.clone()).collect();

            // collect consumed flux from closed or saturated arcs
            for (_, arc_id) in arcs.clone() {
                // passive assignments
                let arc = network.data_of_arc(arc_id).unwrap();
                if arc.open && arc.flow < arc.capacity {
                    continue;
                }
                // if closed or saturated
                consumed_flux += arc.flow;
                // make no assignment because the closed arc always has the identical flow and preflow
            }

            // distribute flux to the remaining arcs
            for (node_id, arc_id) in arcs {
                let arc = network.data_of_arc(arc_id).unwrap();
                let capacity = arc.capacity;
                if !arc.open || arc.flow >= capacity {
                    continue;
                }
                // if open and unsaturated
                let available_flux = incoming_flux - consumed_flux;
                if available_flux <= 0 {
                    // passive assignment
                    let arc = network.mut_data_of_arc(arc_id).unwrap();
                    arc.flow = 0;
                } else {
                    // active assignment
                    // assign flux as much as capacity allows
                    let arc = network.mut_data_of_arc(arc_id).unwrap();
                    let preflow = min(capacity, available_flux);
                    let delta = preflow - arc.flow;
                    // there is no need to keep flow now
                    arc.flow = preflow;
                    consumed_flux += preflow;
                    if delta > 0 {
                        let mut_node = network.mut_data_of_node(node_id).unwrap();
                        mut_node.stack.push((arc_id, delta));
                    }
                }
            }
        }
    }
}

/// balance incoming fluxes of preflows
/// return new s (= start_layer) and update the network
fn balance_incoming(
    layers: &Vec<Vec<NodeId>>,
    network: &mut GraphNetwork<KarzanovNode, KarzanovArc>,
) -> Option<usize> {
    // search for the last deficient layer
    let mut last_deficient_layer: Option<usize> = None;
    // skip the last layer (== sink node) and the first layer (== source node)
    for (d, layer) in layers.iter().enumerate().skip(1).rev().skip(1) {
        // is the node deficient?
        for node_id in layer {
            let outgoing_flux = outgoing_flux_of_flow(node_id.clone(), network);
            let mut incoming_flux = incoming_flux_of_flow(node_id.clone(), network);
            if incoming_flux == outgoing_flux {
                // it is not deficient
                continue;
            }
            if incoming_flux < outgoing_flux {
                panic!("this situation cannot be occured. something went wrong!!")
            }
            // it is deficient
            if last_deficient_layer.is_none() {
                // memorize the last deficient layer
                // watch out: this is a reverse iteration
                last_deficient_layer = Some(d);
            }

            loop {
                if incoming_flux <= outgoing_flux {
                    // it is finally balanced
                    break;
                }
                let node = network.mut_data_of_node(node_id.clone()).unwrap();
                // pop the stack and decrease the flow based on it
                // `delta` is an amount of the flow (of an arc of the arc_id) was increased at once
                if let Some((arc_id, delta)) = node.stack.pop() {
                    let arc = network.mut_data_of_arc(arc_id).unwrap();
                    // if the flow is decreased by `max_decrease`, the incoming_flux coincides with the outgoing_flux
                    let max_decrease = incoming_flux - outgoing_flux;
                    arc.flow -= min(delta, max_decrease);
                    incoming_flux -= min(delta, max_decrease);
                } else {
                    panic!("this situation cannot be occured. something went wrong!!")
                }
            }

            // close the arcs which hit the `over-incoming` state. (and it's balanced now)
            // if the arc's flow were increased, the node overflows again.
            let arcs: Vec<(NodeId, ArcId)> = network.into_node(node_id.clone()).collect();
            for (_, arc_id) in arcs {
                let arc = network.mut_data_of_arc(arc_id).unwrap();
                arc.open = false;
            }
        }
    }

    // start with d-1 th layer. re-distribution or overflow-propagation maybe occur in d-1 th layer
    if let Some(d) = last_deficient_layer {
        return Some(d - 1);
    } else {
        return None;
    }
}

pub fn maxflow(
    source_id: NodeId,
    sink_id: NodeId,
    network: &mut GraphNetwork<KarzanovNode, KarzanovArc>,
) {
    clean_network(network);
    let layers = grouping_nodes_by_layer(source_id, sink_id, network);
    let mut start_layer = 0;
    let mut flow_snapshot = HashMap::<NodeId, u32>::new();

    loop {
        maximize_outgoing(&layers, start_layer, network);
        let new_start_layer = balance_incoming(&layers, network);
        if new_start_layer.is_none() {
            break;
        }
        start_layer = new_start_layer.unwrap();

        // compare with the snapshot
        let mut different = false;
        for (arc_id, arc) in network.arc_data.iter().enumerate() {
            if let Some(arc) = arc {
                if flow_snapshot
                    .get(&arc_id)
                    .is_none_or(|snapshot| &arc.flow != snapshot)
                {
                    different = true;
                    break;
                }
            }
        }
        if !different {
            break;
        }
        // take a snapshot of the flow
        for (arc_id, arc) in network.arc_data.iter().enumerate() {
            if let Some(arc) = arc {
                flow_snapshot.insert(arc_id, arc.flow);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// source, sink, network
    pub fn make_network_instance() -> (NodeId, NodeId, GraphNetwork<KarzanovNode, KarzanovArc>) {
        let mut network = GraphNetwork::new();
        network.add_nodes(vec![KarzanovNode::new(); 9].into_iter());
        network.bulk_connect(
            vec![
                (0, 1, KarzanovArc::new(1)),
                (0, 3, KarzanovArc::new(8)),
                (1, 2, KarzanovArc::new(2)),
                (1, 4, KarzanovArc::new(1)),
                (2, 5, KarzanovArc::new(1)),
                (3, 1, KarzanovArc::new(4)),
                (3, 4, KarzanovArc::new(2)),
                (3, 6, KarzanovArc::new(4)),
                (4, 5, KarzanovArc::new(3)),
                (5, 8, KarzanovArc::new(4)),
                (6, 7, KarzanovArc::new(2)),
                (6, 5, KarzanovArc::new(1)),
                (7, 8, KarzanovArc::new(2)),
            ]
            .into_iter(),
        );
        return (0, 8, network);
    }

    #[test]
    fn karzanov() {
        let (source_id, sink_id, mut network) = make_network_instance();
        let layers = grouping_nodes_by_layer(source_id, sink_id, &mut network);
        println!("Network: {:?}", network);
        println!("Layers: {:?}", layers);
        let mut start_layer = 0;
        let mut flow_snapshot = HashMap::<NodeId, u32>::new();

        loop {
            println!("===compleation===");
            maximize_outgoing(&layers, start_layer, &mut network);
            println!("Network: {:?}", network);
            println!("===balancing===");
            let new_start_layer = balance_incoming(&layers, &mut network);
            println!("Network: {:?}", network);

            if new_start_layer.is_none() {
                println!("nothing to be balanced");
                break;
            }
            start_layer = new_start_layer.unwrap();

            // compare with the snapshot
            let mut different = false;
            for (arc_id, arc) in network.arc_data.iter().enumerate() {
                if let Some(arc) = arc {
                    if flow_snapshot
                        .get(&arc_id)
                        .is_none_or(|snapshot| &arc.flow != snapshot)
                    {
                        println!("{:?} != {:?}", flow_snapshot.get(&arc_id), &arc.flow);
                        different = true;
                        break;
                    }
                    println!("{:?} == {:?}", flow_snapshot.get(&arc_id), &arc.flow)
                }
            }
            if !different {
                println!("no change");
                break;
            }
            // take a snapshot of the flow
            for (arc_id, arc) in network.arc_data.iter().enumerate() {
                if let Some(arc) = arc {
                    flow_snapshot.insert(arc_id, arc.flow);
                }
            }
        }
    }
}

use alg::graph::{GraphNetwork, NodeId};
use alg::karzanov::{maxflow, KarzanovArc, KarzanovNode};

/// source, sink, network
pub fn network_instance1() -> (NodeId, NodeId, GraphNetwork<KarzanovNode, KarzanovArc>) {
    let mut network = GraphNetwork::new();
    network.add_nodes(vec![KarzanovNode::new(); 6].into_iter());
    network.bulk_connect(
        vec![
            (0, 1, KarzanovArc::new(2)),
            (0, 2, KarzanovArc::new(3)),
            (1, 3, KarzanovArc::new(2)),
            (2, 3, KarzanovArc::new(4)),
            (2, 4, KarzanovArc::new(2)),
            (3, 5, KarzanovArc::new(3)),
            (4, 5, KarzanovArc::new(2)),
        ]
        .into_iter(),
    );
    return (0, 5, network);
}

/// source, sink, network
pub fn network_instance2() -> (NodeId, NodeId, GraphNetwork<KarzanovNode, KarzanovArc>) {
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

fn network_instance3() -> (NodeId, NodeId, GraphNetwork<KarzanovNode, KarzanovArc>) {
    let mut network = GraphNetwork::new();
    network.add_nodes(vec![KarzanovNode::new(); 3].into_iter());
    network
        .bulk_connect(vec![(0, 1, KarzanovArc::new(1)), (1, 2, KarzanovArc::new(2))].into_iter());
    return (0, 2, network);
}

fn network_instance4() -> (NodeId, NodeId, GraphNetwork<KarzanovNode, KarzanovArc>) {
    let mut network = GraphNetwork::new();
    network.add_nodes(vec![KarzanovNode::new(); 2].into_iter());
    network.bulk_connect(vec![(0, 1, KarzanovArc::new(1))].into_iter());
    return (0, 1, network);
}

fn main() {
    let (source, sink, mut network) = network_instance1();
    maxflow(source, sink, &mut network);
    println!("network: {:?}", network);

    let (source, sink, mut network) = network_instance2();
    maxflow(source, sink, &mut network);
    println!("network: {:?}", network);

    let (source, sink, mut network) = network_instance3();
    maxflow(source, sink, &mut network);
    println!("network: {:?}", network);

    let (source, sink, mut network) = network_instance4();
    maxflow(source, sink, &mut network);
    println!("network: {:?}", network);
}

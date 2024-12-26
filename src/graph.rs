use genawaiter::sync::*;
use std::collections::HashMap;

pub type NodeId = usize;
pub type ArcId = usize;

#[derive(Debug, Clone)]
struct ArcConnection {
    from: NodeId,
    into: NodeId,
}

#[derive(Debug)]
pub struct GraphNetwork<N, A> {
    pub node_data: Vec<Option<N>>, // Option is to support removal of nodes
    arcs_into: Vec<Vec<ArcId>>,    // The length of this vector is the number of nodes
    arcs_from: Vec<Vec<ArcId>>,    // The length of this vector is the number of nodes
    pub arc_data: Vec<Option<A>>,  // Option is to support removal of arcs
    arc_connections: Vec<ArcConnection>, // The length of this vector is the number of arcs
}

impl<'g, N, A> GraphNetwork<N, A> {
    pub fn new() -> Self {
        GraphNetwork {
            node_data: Vec::new(),
            arcs_into: Vec::new(),
            arcs_from: Vec::new(),
            arc_data: Vec::new(),
            arc_connections: Vec::new(),
        }
    }

    pub fn clean(self) -> Self {
        let mut old_new_map = HashMap::<NodeId, NodeId>::new();
        let mut brand_new = Self::new();

        for (old_node_id, node_data) in self.node_data.into_iter().enumerate() {
            if let Some(node_data) = node_data {
                let new_node_id = brand_new.add_node(node_data);
                old_new_map.insert(old_node_id, new_node_id);
            }
        }

        for (old_arc_id, arc_data) in self.arc_data.into_iter().enumerate() {
            if let Some(arc_data) = arc_data {
                let ArcConnection { from, into } = self.arc_connections[old_arc_id];
                brand_new.connect(old_new_map[&from], old_new_map[&into], arc_data);
            }
        }

        return brand_new;
    }

    pub fn is_node_in(&self, node: NodeId) -> bool {
        self.node_data.len() > node && self.node_data[node].is_some()
    }

    pub fn is_arc_in(&self, from: NodeId, into: NodeId) -> bool {
        // if the nodes do not exist, then the arc does not exist
        if !self.is_node_in(from) || !self.is_node_in(into) {
            return false;
        }
        // if the same arc is in both the outarcs and inarcs, then it is an arc which connects the two nodes
        for arc in &self.arcs_from[from] {
            // skip 'None' arcs
            if self.arc_data[arc.clone()].is_some() && self.arcs_into[into].contains(&arc) {
                return true;
            }
        }
        return false;
    }

    pub fn data_of_node(&self, node: NodeId) -> Option<&N> {
        self.node_data[node].as_ref()
    }

    pub fn mut_data_of_node(&mut self, node: NodeId) -> Option<&mut N> {
        self.node_data[node].as_mut()
    }

    pub fn data_of_arc(&self, arc: ArcId) -> Option<&A> {
        self.arc_data[arc].as_ref()
    }

    pub fn mut_data_of_arc(&mut self, arc: ArcId) -> Option<&mut A> {
        self.arc_data[arc].as_mut()
    }

    pub fn between_nodes(&'g self, from: NodeId, into: NodeId) -> impl Iterator<Item = ArcId> + 'g {
        Gen::new(|co| async move {
            // if the nodes do not exist, then the arc does not exist
            if !self.is_node_in(from) || !self.is_node_in(into) {
                panic!("Node does not exist");
            }
            // if the same arc is in both the outarcs and inarcs, then it is an arc which connects the two nodes
            for arc_id in &self.arcs_from[from] {
                if self.arc_data[arc_id.clone()].is_some() {
                    if self.arcs_into[into].contains(arc_id) {
                        co.yield_(arc_id.clone()).await;
                    }
                }
            }
        })
        .into_iter()
    }

    pub fn from_node(&'g self, from: NodeId) -> impl Iterator<Item = (NodeId, ArcId)> + 'g {
        Gen::new(|co| async move {
            // if the nodes do not exist, then the arc does not exist
            if !self.is_node_in(from) {
                panic!("Node does not exist");
            }
            // if the same arc is in both the outarcs and inarcs, then it is an arc which connects the two nodes
            for arc_id in &self.arcs_from[from] {
                if self.arc_data[arc_id.clone()].is_some() {
                    co.yield_((self.arc_connections[arc_id.clone()].into, arc_id.clone()))
                        .await;
                }
            }
        })
        .into_iter()
    }

    pub fn into_node(&'g self, into: NodeId) -> impl Iterator<Item = (NodeId, ArcId)> + 'g {
        Gen::new(|co| async move {
            // if the nodes do not exist, then the arc does not exist
            if !self.is_node_in(into) {
                panic!("Node does not exist");
            }
            // if the same arc is in both the outarcs and inarcs, then it is an arc which connects the two nodes
            for arc_id in &self.arcs_into[into] {
                if self.arc_data[arc_id.clone()].is_some() {
                    co.yield_((self.arc_connections[arc_id.clone()].from, arc_id.clone()))
                        .await;
                }
            }
        })
        .into_iter()
    }

    pub fn add_node(&mut self, data: N) -> NodeId {
        let node_id = self.node_data.len();
        self.node_data.push(Some(data));
        self.arcs_into.push(Vec::new());
        self.arcs_from.push(Vec::new());
        return node_id;
    }

    pub fn add_nodes<I: Iterator<Item = N>>(&mut self, data: I) {
        for node in data {
            self.add_node(node);
        }
    }

    pub fn remove_node(&mut self, node: NodeId) -> Option<N> {
        // do not pop from the vector, as to keep its index the same
        if !self.is_node_in(node) {
            return None;
        }
        // release the arcs
        self.arcs_into[node].clear();
        self.arcs_from[node].clear();
        self.node_data[node].take()
    }

    pub fn connect(&mut self, from: NodeId, into: NodeId, value: A) -> ArcId {
        if !self.is_node_in(from) || !self.is_node_in(into) {
            panic!("Node does not exist");
        }
        let arc_id = self.arc_data.len();
        self.arc_data.push(Some(value));
        self.arc_connections.push(ArcConnection { from, into });
        self.arcs_from[from].push(arc_id);
        self.arcs_into[into].push(arc_id);
        return arc_id;
    }

    pub fn bulk_connect<I: Iterator<Item = (NodeId, NodeId, A)>>(&mut self, arcs: I) {
        for (from, into, value) in arcs {
            self.connect(from, into, value);
        }
    }

    pub fn disconnect(&mut self, arc: ArcId) -> Option<A> {
        // do not pop from the vector, as to keep its index the same
        // NOTE: there is not method to check if an arc is in the graph with ArcId
        if self.arc_data.len() <= arc {
            return None;
        }
        self.arc_data[arc].take()
        // arc_connections is left as it.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network() -> Result<(), ()> {
        let mut network = GraphNetwork::<usize, i32>::new();
        network.add_nodes(vec![0, 1, 2, 3, 4, 5, 6].into_iter());
        network.bulk_connect(
            vec![
                (0, 1, 2), //0
                (0, 2, 3), //1
                (1, 3, 2), //2
                (1, 4, 0),
                (2, 3, 4), //3
                (2, 4, 2), //4
                (3, 5, 3), //5
                (4, 5, 2), //6
            ]
            .into_iter(),
        );
        network.disconnect(3);
        network.remove_node(6);
        network = network.clean();
        println!("Network: {:?}", network);
        assert_eq!(network.from_node(3).collect::<Vec<_>>(), vec![(5, 5)]);
        assert_eq!(
            network.into_node(3).collect::<Vec<_>>(),
            vec![(1, 2), (2, 3)]
        );
        assert_eq!(network.is_arc_in(1, 4), false);
        assert_eq!(network.is_node_in(1), true);
        assert_eq!(network.is_node_in(6), false);
        assert_eq!(network.between_nodes(0, 1).collect::<Vec<_>>(), vec![0]);
        assert_eq!(network.data_of_node(0), Some(&0));
        Ok(())
    }
}

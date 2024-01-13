use core::fmt;
use std::fmt::Debug;

use cgmath::Vector3;

#[derive(PartialEq, Eq)]
pub enum Node<T> {
    Leaf(Option<T>),
    Branch(Box<[Node<T>; 8]>),
}

impl<T: Clone> Node<T> {
    pub fn insert(&mut self, position: Vector3<u32>, depth: u32, node: Node<T>, scale: u32) {
        let position = Vector3 {
            x: position.x.min(scale - 1),
            y: position.y.min(scale - 1),
            z: position.z.min(scale - 1),
        };
        let depth = depth.min(scale.leading_zeros()); // depth = max(depth, log2(scale))
                                                      // TODO: limit depth so that it's not outside of scale if node is a branch

        if depth == 0 {
            *self = node;
            return;
        }
        match self {
            Node::Leaf(voxel) => {
                *self = Node::Branch(Box::new(std::array::from_fn(|_| Node::Leaf(voxel.clone()))));

                self.insert(position % scale, depth - 1, node, scale);
            }
            Node::Branch(children) => {
                let scale = scale / 2;
                let index = ((position.x >= scale) as usize) << 0
                    | ((position.y >= scale) as usize) << 1
                    | ((position.z >= scale) as usize) << 2;

                children[index].insert(position % scale, depth - 1, node, scale)
            }
        }
    }

    pub fn fetch(&self, position: Vector3<u32>, depth: u32, scale: u32) -> &Self {
        let position = Vector3 {
            x: position.x.min(scale - 1),
            y: position.y.min(scale - 1),
            z: position.z.min(scale - 1),
        };

        if depth == 0 {
            return self;
        }
        match self {
            Node::Leaf(_) => self,
            Node::Branch(children) => {
                let scale = scale >> 1;
                let index = ((position.x >= scale) as usize) << 0
                    | ((position.y >= scale) as usize) << 1
                    | ((position.z >= scale) as usize) << 2;

                children[index].fetch(position % scale, depth - 1, scale)
            }
        }
    }

    pub fn fetch_mut(&mut self, position: Vector3<u32>, depth: u32, scale: u32) -> &mut Self {
        let position = Vector3 {
            x: position.x.min(scale - 1),
            y: position.y.min(scale - 1),
            z: position.z.min(scale - 1),
        };

        if depth == 0 {
            return self;
        }
        match self {
            Node::Leaf(_) => self,
            Node::Branch(children) => {
                let scale = scale >> 1;
                let index = ((position.x >= scale) as usize) << 0
                    | ((position.y >= scale) as usize) << 1
                    | ((position.z >= scale) as usize) << 2;

                children[index].fetch_mut(position % scale, depth - 1, scale)
            }
        }
    }

    pub fn pack_traverse(&self, nodes: &mut Vec<[u32; 8]>, voxels: &mut Vec<T>) -> u32 {
        match self {
            Node::Leaf(Some(voxel)) => {
                voxels.push(voxel.clone());
                (voxels.len() - 1) as u32
            }
            Node::Leaf(None) => u32::MAX,
            Node::Branch(children) => {
                let mut packed_children = [u32::MAX; 8];
                children
                    .iter()
                    .enumerate()
                    .for_each(|(i, child)| packed_children[i] = child.pack_traverse(nodes, voxels));

                nodes.push(packed_children);
                (nodes.len() as u32 - 1) | (1 << 31)
            }
        }
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaf(voxel) => write!(f, "Leaf({:#?})", voxel),
            Self::Branch(children) => write!(f, "Branch({:#?})", children),
        }
    }
}

use std::fmt::Debug;

use cgmath::Vector3;

use crate::node::Node;

pub struct PackedData<T: Sized> {
    pub root: u32,
    pub scale: u32,
    pub nodes: Vec<[u32; 8]>,
    pub voxels: Vec<T>,
}

#[derive(Debug)]
pub struct Voxtree<T: Clone> {
    scale: u32,
    root: Node<T>,
}

pub struct VoxtreeBuilder<T: Clone> {
    scale: u32,
    root: Node<T>,
}

impl<T: Clone> Voxtree<T> {
    pub fn default() -> Self {
        Self {
            scale: 1 << 8,
            root: Node::Leaf(None),
        }
    }

    pub fn builder() -> VoxtreeBuilder<T> {
        VoxtreeBuilder::default()
    }

    pub fn insert(&mut self, position: Vector3<u32>, depth: u32, node: Node<T>) {
        self.root.insert(position, depth, node, self.scale);
    }

    pub fn fetch(&self, position: Vector3<u32>, depth: u32) -> &Node<T> {
        self.root.fetch(position, depth, self.scale)
    }

    pub fn fetch_mut(&mut self, position: Vector3<u32>, depth: u32) -> &mut Node<T> {
        self.root.fetch_mut(position, depth, self.scale)
    }

    pub fn optimize(&mut self) {
        todo!()
    }

    pub fn pack(&self) -> PackedData<T> {
        let mut data = PackedData {
            root: 0,
            scale: self.scale,
            nodes: vec![],
            voxels: vec![],
        };

        self.root.pack_traverse(&mut data.nodes, &mut data.voxels);
        data.root = (data.nodes.len() as u32 - 1) | (1 << 31);

        data
    }
}

impl<T: Clone> VoxtreeBuilder<T> {
    pub fn default() -> Self {
        Self {
            scale: 1 << 8,
            root: Node::Leaf(None),
        }
    }

    pub fn with_max_depth(mut self, depth: u32) -> VoxtreeBuilder<T> {
        self.scale = 1 << depth;
        self
    }

    pub fn with_root(mut self, root: Node<T>) -> VoxtreeBuilder<T> {
        self.root = root;
        self
    }

    pub fn build(self) -> Voxtree<T> {
        Voxtree {
            scale: self.scale,
            root: self.root,
        }
    }
}

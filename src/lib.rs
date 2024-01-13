#[cfg(not(feature = "no_std"))]
mod node;
mod packed;
#[cfg(not(feature = "no_std"))]
mod voxtree;

#[cfg(not(feature = "no_std"))]
pub use node::Node;
#[cfg(not(feature = "no_std"))]
pub use voxtree::Voxtree;

#[cfg(test)]
#[cfg(not(feature = "no_std"))]
mod voxtree_tests {
    use cgmath::Vector3;

    use super::*;

    #[test]
    fn fetch() {
        let tree: Voxtree<u32> = Voxtree::builder()
            .with_max_depth(2)
            .with_root(Node::Branch(Box::new([
                Node::Leaf(Some(0)),
                Node::Leaf(Some(1)),
                Node::Branch(Box::new([
                    Node::Leaf(Some(8)),
                    Node::Leaf(Some(9)),
                    Node::Leaf(Some(10)),
                    Node::Leaf(Some(11)),
                    Node::Leaf(Some(12)),
                    Node::Leaf(Some(13)),
                    Node::Leaf(Some(14)),
                    Node::Leaf(Some(15)),
                ])),
                Node::Leaf(Some(3)),
                Node::Leaf(Some(4)),
                Node::Leaf(Some(5)),
                Node::Leaf(Some(6)),
                Node::Leaf(Some(7)),
            ])))
            .build();

        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 0, 0), 2),
            Node::Leaf(Some(0))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(2, 0, 0), 2),
            Node::Leaf(Some(1))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(2, 2, 0), 2),
            Node::Leaf(Some(3))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 0, 2), 2),
            Node::Leaf(Some(4))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(2, 0, 2), 2),
            Node::Leaf(Some(5))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 2, 2), 2),
            Node::Leaf(Some(6))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(2, 2, 2), 2),
            Node::Leaf(Some(7))
        );

        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 2, 0), 2),
            Node::Leaf(Some(8))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(1, 2, 0), 2),
            Node::Leaf(Some(9))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 3, 0), 2),
            Node::Leaf(Some(10))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(1, 3, 0), 2),
            Node::Leaf(Some(11))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 2, 1), 2),
            Node::Leaf(Some(12))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(1, 2, 1), 2),
            Node::Leaf(Some(13))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 3, 1), 2),
            Node::Leaf(Some(14))
        );
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(1, 3, 1), 2),
            Node::Leaf(Some(15))
        );
    }

    #[test]
    fn fetch_mut() {
        let mut tree: Voxtree<u32> = Voxtree::builder()
            .with_max_depth(2)
            .with_root(Node::Branch(Box::new([
                Node::Leaf(Some(0)),
                Node::Leaf(Some(1)),
                Node::Branch(Box::new([
                    Node::Leaf(Some(8)),
                    Node::Leaf(Some(9)),
                    Node::Leaf(Some(10)),
                    Node::Leaf(Some(11)),
                    Node::Leaf(Some(12)),
                    Node::Leaf(Some(13)),
                    Node::Leaf(Some(14)),
                    Node::Leaf(Some(15)),
                ])),
                Node::Leaf(Some(3)),
                Node::Leaf(Some(4)),
                Node::Leaf(Some(5)),
                Node::Leaf(Some(6)),
                Node::Leaf(Some(7)),
            ])))
            .build();

        let node = tree.fetch_mut(Vector3::<u32>::new(0, 2, 0), 2);

        assert_eq!(*node, Node::Leaf(Some(8)));

        *node = Node::Leaf(Some(69420));

        assert_eq!(*node, Node::Leaf(Some(69420)));
        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 2, 0), 2),
            Node::Leaf(Some(69420))
        );
    }

    #[test]
    fn insert() {
        let mut tree: Voxtree<u32> = Voxtree::builder()
            .with_max_depth(2)
            .with_root(Node::Branch(Box::new([
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
                Node::Leaf(None),
            ])))
            .build();

        tree.insert(
            Vector3::<u32>::new(0, 2, 0),
            2,
            Node::Branch(Box::new([
                Node::Leaf(Some(0)),
                Node::Leaf(Some(1)),
                Node::Leaf(Some(2)),
                Node::Leaf(Some(3)),
                Node::Leaf(Some(4)),
                Node::Leaf(Some(5)),
                Node::Leaf(Some(6)),
                Node::Leaf(Some(7)),
            ])),
        );

        assert_eq!(
            *tree.fetch(Vector3::<u32>::new(0, 2, 0), 1),
            Node::Branch(Box::new([
                Node::Leaf(Some(0)),
                Node::Leaf(Some(1)),
                Node::Leaf(Some(2)),
                Node::Leaf(Some(3)),
                Node::Leaf(Some(4)),
                Node::Leaf(Some(5)),
                Node::Leaf(Some(6)),
                Node::Leaf(Some(7)),
            ]))
        );
    }
}

#[cfg(test)]
#[cfg(not(feature = "no_std"))]
mod packed_voxtree_tests {
    use crate::{packed::PackedVoxtree, Node, Voxtree};

    #[test]
    fn fetch() {
        let tree: Voxtree<u32> = Voxtree::builder()
            .with_max_depth(2)
            .with_root(Node::Branch(Box::new([
                Node::Leaf(Some(0)),
                Node::Leaf(Some(1)),
                Node::Branch(Box::new([
                    Node::Leaf(Some(8)),
                    Node::Leaf(Some(9)),
                    Node::Leaf(Some(10)),
                    Node::Leaf(Some(11)),
                    Node::Leaf(Some(12)),
                    Node::Leaf(Some(13)),
                    Node::Leaf(Some(14)),
                    Node::Leaf(Some(15)),
                ])),
                Node::Leaf(Some(3)),
                Node::Leaf(Some(4)),
                Node::Leaf(Some(5)),
                Node::Leaf(Some(6)),
                Node::Leaf(Some(7)),
            ])))
            .build();

        let packed_data = tree.pack();

        let packed_tree = PackedVoxtree {
            root: packed_data.root,
            scale: packed_data.scale,
            nodes: &packed_data.nodes,
            voxels: &packed_data.voxels,
        };
        dbg!(packed_tree.nodes);
        dbg!(packed_tree.voxels);

        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 0, 0, 2) as usize],
            0
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(2, 0, 0, 2) as usize],
            1
        );
        // gap branch
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(2, 2, 0, 2) as usize],
            3
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 0, 2, 2) as usize],
            4
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(2, 0, 2, 2) as usize],
            5
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 2, 2, 2) as usize],
            6
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(2, 2, 2, 2) as usize],
            7
        );

        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 2, 0, 2) as usize],
            8
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(1, 2, 0, 2) as usize],
            9
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 3, 0, 2) as usize],
            10
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(1, 3, 0, 2) as usize],
            11
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 2, 1, 2) as usize],
            12
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(1, 2, 1, 2) as usize],
            13
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(0, 3, 1, 2) as usize],
            14
        );
        assert_eq!(
            packed_tree.voxels[packed_tree.fetch(1, 3, 1, 2) as usize],
            15
        );
    }
}

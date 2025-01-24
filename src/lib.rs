use std::{fmt::Debug, ops::Range};

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use either::Either::{self, Left, Right};

#[derive(Debug, Clone)]
pub struct Voxtree<T> {
    pub depth: u8,
    pub branches: Vec<Branch>,
    pub leaves: Vec<T>,
    pub features: Features,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct Branch {
    pub bitmask: u64,
    address: u32,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    // TODO: need to figure out what features will be added
    pub struct Features: u8 {
        const None = 0b0;
        const DirectedAcyclicGraph = 0b1;
    }
}

impl Debug for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Branch {{\n    bitmask: [\n        {}\n    ],\n    address: {}\n}}",
            (0..4)
                .map(|y| (0..4)
                    .map(|z| (0..4)
                        .map(|x| ((self.bitmask >> ((z << 4) | (y << 2) | (x))) & 1).to_string())
                        .collect::<Vec<String>>()
                        .join(" "))
                    .collect::<Vec<String>>()
                    .join("  "))
                .collect::<Vec<String>>()
                .join("\n        "),
            format_args!(
                "{}{}: {}",
                if self.is_solid() { "S_" } else { "" },
                if self.is_leaf() { "LEAF" } else { "BRANCH" },
                self.get_address().to_string()
            )
        ))
    }
}

// TODO: do better error handling
// #[derive(Debug)]
// pub struct DepthOutOfBounds;

impl<T: Clone + Copy + PartialEq> Voxtree<T> {
    pub fn empty(scale: u8, features: Features) -> Self {
        Self {
            depth: scale,
            branches: vec![Branch {
                bitmask: 0,
                address: 1,
            }],
            leaves: vec![],
            features,
        }
    }

    pub fn get(
        &mut self,
        mut x: u32,
        mut y: u32,
        mut z: u32,
        w: u8,
    ) -> Option<Either<&mut T, &mut Branch>> {
        let dimension_length = 1 << (self.depth << 1);
        assert!(x < dimension_length && y < dimension_length && z < dimension_length);

        let mut branch_address = 0;

        for depth in (0..w.min(self.depth)).rev() {
            let branch = self.branches[branch_address];

            let index = branch.get_index(x, y, z, depth);

            x &= 3;
            y &= 3;
            z &= 3;

            if (branch.bitmask >> index & 1) == 0 {
                return None;
            }

            let transformed_index = branch.get_transformed_index(index);

            if branch.is_leaf() {
                if branch.is_solid() {
                    return Some(Left(&mut self.leaves[branch.get_address()]));
                }

                return Some(Left(
                    &mut self.leaves[branch.get_address() + transformed_index],
                ));
            }

            branch_address = branch.get_address() + transformed_index;
        }

        Some(Right(&mut self.branches[branch_address]))
    }

    // TODO-MAYBE: change to simply remake the entire tree structure (copy-on-write)
    pub fn set(
        &mut self,
        mut x: u32,
        mut y: u32,
        mut z: u32,
        w: u8,
        node: Option<Either<T, Branch>>,
    ) -> Result<(), ()> {
        let dimension_length = 1 << (self.depth << 1);
        assert!(x < dimension_length && y < dimension_length && z < dimension_length);

        let mut previous_address = 0;
        let mut address = 0;
        let mut index_store = 0;

        for depth in (0..w).rev() {
            let traversal_node = self.branches[address];

            let index = traversal_node.get_index(x, y, z, depth);

            x &= 3;
            y &= 3;
            z &= 3;

            let transformed_index = traversal_node.get_transformed_index(index);

            previous_address = address;
            address = traversal_node.get_address() + transformed_index;
            index_store = index;
        }

        let parent = self.branches[previous_address];

        match node {
            Some(node) => {
                // TODO: needs to account for solid nodes
                if parent.bitmask >> index_store & 1 == 0 {
                    self.shift_addresses(address..address + 1, node.is_left(), false);
                }

                self.branches[previous_address].bitmask |= 1 << index_store;

                match node {
                    Left(node) => {
                        if parent.is_branch() {
                            self.recursive_delete(previous_address);

                            // TODO-MAYBE: maybe add the condition that makes solid leaf children still exist in branch

                            // TODO-OPT: make it just modify the already existing branch instead of deleting it in recursive_delete
                            self.branches.insert(
                                previous_address,
                                Branch {
                                    bitmask: (1 << 31) + (1 << index_store) as u64,
                                    address: self.leaves.len() as u32, // TODO-OPT: make it put the address at the "farthest" out leaf child of the branch parent
                                },
                            );
                        }

                        // if solid leaf don't make new node if the same, but if not then make all new leaves
                        if parent.is_solid() {
                            if node == self.leaves[parent.get_address()] {
                                return Ok(());
                            } else {
                                let range = parent.get_address()
                                    ..parent.get_address() + parent.bitmask.count_ones() as usize
                                        - 1;
                                self.leaves.splice(
                                    range.clone(),
                                    vec![self.leaves[parent.get_address()].clone(); range.len()],
                                );
                                self.shift_addresses(range, true, false);
                            }
                        }

                        self.leaves[address] = node;
                    }
                    Right(node) => {
                        if parent.is_leaf() {
                            // TODO: implement early leaf condition (needs to modify parent)

                            let range = parent.get_address()
                                ..parent.get_address()
                                    + if parent.is_solid() {
                                        1
                                    } else {
                                        parent.bitmask.count_ones() as usize
                                    };

                            let leaves = self.leaves[range.clone()].len();
                            // TODO-OPT: could make a BIG optimization here by just storing a small cache of the current node's parents that way you can just directly count the ones
                            let count = self.count_previous_branches(0, previous_address);
                            let new_address = (self.branches[previous_address].address
                                & (u32::MAX >> 2))
                                + count as u32;

                            self.branches[previous_address].address = new_address;

                            self.branches.splice(
                                new_address as usize..new_address as usize,
                                (0..range.len()).map(|i| Branch {
                                    bitmask: u64::MAX,
                                    address: (0b11 << 30) + parent.get_address() as u32 + i as u32,
                                }),
                            );
                            self.shift_addresses(
                                new_address as usize..new_address as usize + leaves,
                                false,
                                false,
                            );

                            address = new_address as usize + index_store;
                        }

                        self.branches[address] = node;
                    }
                }
            }
            None => {
                self.branches[previous_address].bitmask &= !(1 << index_store);

                if parent.is_leaf() && !parent.is_solid() {
                    self.leaves.remove(address);
                    self.shift_addresses(address..address + 1, true, true);
                } else {
                    let count = parent.bitmask.count_ones() as usize;

                    // TODO-OPT: batch shifts
                    self.recursive_delete(address);
                    self.shift_addresses(
                        address..address + count,
                        self.branches[address].is_leaf(),
                        true,
                    );
                    self.branches.remove(address);
                }
            }
        }

        Ok(())
    }

    fn recursive_delete(&mut self, address: usize) {
        let branch = self.branches[address];
        let next = branch.get_address();
        let count = if branch.is_solid() {
            1
        } else {
            branch.bitmask.count_ones() as usize
        };

        if branch.is_leaf() {
            self.leaves.drain(next..next + count);
        } else {
            for i in 0..count {
                self.recursive_delete(next + i);
            }
            self.branches.drain(next..next + count);
            self.branches[address].address += 1 << 31;
        }
        self.branches[address].bitmask = 0;

        self.shift_addresses(next..next + count, branch.is_leaf(), true); // TODO-OPT: batch shift addresses
    }

    fn shift_addresses(&mut self, range: Range<usize>, leaf: bool, removal: bool) {
        for branch in self.branches.iter_mut() {
            if leaf != branch.is_leaf() || branch.get_address() < range.end {
                continue;
            }

            if removal {
                branch.address -= range.len() as u32;
            } else {
                branch.address += range.len() as u32;
            }
        }
    }

    fn count_previous_branches(&self, root: usize, address: usize) -> usize {
        let branch = self.branches[root];

        let count = branch.bitmask.count_ones();

        // TODO-OPT: hotfix probably better way to do this
        if root >= address {
            return 0;
        }

        if branch.is_leaf() {
            1
        } else {
            let mut sum = 0;
            for i in 0..count {
                if branch.get_address() + i as usize == address {
                    return branch.bitmask.count_ones() as usize + 1;
                }
                sum += self.count_previous_branches(branch.get_address() + i as usize, address);
            }
            sum
        }
    }

    pub fn pack(&self) -> Voxtree<T> {
        let mut branches = self.branches.clone();
        let mut leaves = self.leaves.clone();

        if self.features.contains(Features::DirectedAcyclicGraph) {
            // TODO: DAG must be able to have a configurable height
            // TODO: include an optimization to the data structure so that when it is not using a DAG it doesn't store addresses and instead has them lined up sequentially in memory and uses relative positioning for indices
            // TODO: DAG should have something to help with the usage of materials, materials will be a function that take in position and spit out an output regardless of data, then having voxel data be like a u8 or something to represent what material it is, this should also be toggleable because it's a tradeoff for memory but will increase processing time palette compressiong for mixing materials could help
        }

        // Self {
        //     branches,
        //     leaves,
        //     ..*self
        // }

        todo!()
    }

    pub fn with_depth(&self, depth: u8) -> Voxtree<T> {
        Self {
            depth,
            ..self.clone()
        }
    }

    pub fn with_features(&self, additional_features: Features) -> Voxtree<T> {
        Self {
            features: self.features | additional_features,
            ..self.clone()
        }
    }
}

impl Branch {
    pub fn is_leaf(&self) -> bool {
        self.address >> 31 == 1
    }

    pub fn is_branch(&self) -> bool {
        self.address >> 31 == 0
    }

    pub fn is_solid(&self) -> bool {
        self.address >> 30 == 0b11
    }

    pub fn get_address(&self) -> usize {
        (self.address & !(0b11u32.wrapping_shl(30 | (!(self.address >> 31))))) as usize
    }

    /// Generates a "transformed" index that conserves sparsity.
    pub fn get_index(&self, x: u32, y: u32, z: u32, w: u8) -> usize {
        let double_depth = w << 1;

        ((z >> double_depth) << 4 | (y >> double_depth) << 2 | (x >> double_depth)) as usize
    }

    pub fn get_transformed_index(&self, index: usize) -> usize {
        (self.bitmask.count_ones() - (self.bitmask >> index).count_ones()) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        let mut tree: Voxtree<u8> = Voxtree::empty(2, Features::DirectedAcyclicGraph);

        tree.branches[0] = Branch {
            bitmask:
                0b0000_0000_0000_0100__0000_0000_0000_0001__0000_0000_0000_0000__0000_0000_0000_0001,
            address: 1,
        };
        tree.branches.push(Branch {
            bitmask: u64::MAX,
            address: 1 << 31,
        });
        tree.branches.push(Branch {
            bitmask: u64::MAX,
            address: (0b11 << 30) + 64,
        });
        tree.branches.push(Branch {
            bitmask: 0b0100_0110_1000_1110,
            address: (0b11 << 30) + 65,
        });

        tree.leaves = (0..64).collect();
        tree.leaves.push(99);
        tree.leaves.push(100);

        let mut counter = 0;
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    let value = tree.get(x, y, z, 2).unwrap().unwrap_left();
                    assert_eq!(&counter, value);

                    let value = tree.get(x, y, 8 + z, 2).unwrap().unwrap_left();
                    assert_eq!(value, &99);

                    counter += 1;
                }
            }
        }
    }

    #[test]
    fn set() {
        let mut tree: Voxtree<u8> = Voxtree::empty(2, Features::DirectedAcyclicGraph);
        tree.branches[0] = Branch {
            bitmask:
                0b0000_0000_0000_0100__0000_0000_0000_0001__0000_0000_0000_0000__0000_0000_0000_0001,
            address: 1,
        };
        tree.branches.push(Branch {
            bitmask: u16::MAX as u64,
            address: 1 << 31,
        });
        tree.branches.push(Branch {
            bitmask: u64::MAX,
            address: (0b11 << 30) + 16,
        });
        tree.branches.push(Branch {
            bitmask: 0b0100_0110_1000_1110,
            address: (0b11 << 30) + 17,
        });

        tree.leaves = (0..16).collect();
        tree.leaves.push(99);
        tree.leaves.push(100);

        dbg!(&tree);
        dbg!(tree.get(2, 0, 0, 2).unwrap());

        tree.set(
            2,
            0,
            0,
            2,
            Some(Right(Branch {
                bitmask: 0,
                address: 999,
            })),
        )
        .unwrap();

        dbg!(&tree);
        dbg!(tree.get(0, 0, 0, 3));
    }

    #[test]
    fn node_utils() {
        for i in 0..u16::MAX {
            let branch = Branch {
                bitmask: u64::MAX,
                address: i as u32,
            };
            assert!(!branch.is_leaf());
        }
        let solid_leaf_node = Branch {
            bitmask: u64::MAX,
            address: 0b11111111111111111111111111111111,
        };

        assert_eq!(solid_leaf_node.is_leaf(), true);
        assert_eq!(solid_leaf_node.is_branch(), false);
        assert_eq!(solid_leaf_node.is_solid(), true);
        assert_eq!(
            solid_leaf_node.get_address(),
            0b00111111111111111111111111111111
        );

        let branch_node = Branch {
            bitmask: 0,
            address: 0b00111111001110111110111100011111,
        };

        assert_eq!(branch_node.is_leaf(), false);
        assert_eq!(branch_node.is_branch(), true);
        assert_eq!(branch_node.is_solid(), false);
        assert_eq!(
            branch_node.get_address(),
            0b00111111001110111110111100011111
        );
    }
}

pub struct Voxtree<T> {
    pub scale: u8,
    pub root: u64,
    pub branches: Vec<u64>,
    pub leaves: Vec<T>,
}

#[derive(Debug)]
pub enum Node<T> {
    Leaf(T),
    Branch(u64),
}

#[derive(Debug)]
pub struct DepthOutOfBounds;

impl<T> Voxtree<T> {
    pub fn empty(scale: u8) -> Self {
        Self {
            scale,
            root: 0,
            branches: vec![],
            leaves: vec![],
        }
    }

    pub fn get(&self, x: u64, y: u64, z: u64, depth: u8) -> Node<T> {
        todo!()
    }

    pub fn set(&mut self) -> Result<(), DepthOutOfBounds> {
        todo!()
    }
}

impl<T> Node<T> {
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branch(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        todo!()
    }

    #[test]
    fn set() {
        todo!()
    }
}

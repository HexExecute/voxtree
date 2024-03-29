// TODO: shouldn't use anything that's from std
pub struct PackedVoxtree<'a, T: Sized> {
    pub root: u32,
    pub scale: u32,
    pub nodes: &'a [[u32; 8]], // TODO:
    pub voxels: &'a [T],       // TODO:
}

impl<'a, T> PackedVoxtree<'a, T> {
    pub fn fetch(&self, x: u32, y: u32, z: u32, depth: u32) -> usize {
        let mut x = x.min(self.scale - 1);
        let mut y = y.min(self.scale - 1);
        let mut z = z.min(self.scale - 1);
        let depth = depth.min(self.scale.leading_zeros()); // depth = max(depth, log2(scale))
                                                           // TODO: limit depth so that it's not outside of scale if node is a branch
        let mut node = self.root;

        for i in 0..depth {
            if node >> 31 == 0 {
                break; // leaf node
            }

            let scale = self.scale >> (1 << i);

            let index = ((x >= scale) as usize) << 0
                | ((y >= scale) as usize) << 1
                | ((z >= scale) as usize) << 2;

            node = self.nodes[node as usize & !(1 << 31)][index];

            x %= scale;
            y %= scale;
            z %= scale;
        }

        node as usize
    }
}

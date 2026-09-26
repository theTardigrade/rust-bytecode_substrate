pub struct SegmentTree {
	size: usize,
	nodes: Vec<Vec<usize>>,
}

impl SegmentTree {
	pub fn new(size: usize) -> Self {
		Self {
			size,
			nodes: vec![Vec::new(); size * 4],
		}
	}
}
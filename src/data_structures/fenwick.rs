pub struct FenwickTree {
	tree: Vec<usize>,
}

impl FenwickTree {
	pub fn new(size: usize) -> Self {
		Self {
			tree: vec![0; size + 1],
		}
	}

	pub fn from_slice(values: &[usize]) -> Self {
		let mut tree = Vec::with_capacity(values.len() + 1);

		tree.push(0);
		tree.extend_from_slice(values);

		for tree_index in 1..tree.len() {
			let parent_index = (tree_index + lowest_set_bit(tree_index)) as usize;

			if parent_index < tree.len() {
				tree[parent_index] += tree[tree_index];
			}
		}

		Self {
			tree,
		}
	}

	pub fn add(&mut self, index: usize, amount: usize) {
		assert!(
			index < self.tree.len() - 1,
			"Fenwick tree index out of bounds",
		);

		let mut tree_index = index + 1;

		while tree_index < self.tree.len() {
			self.tree[tree_index] += amount;
			tree_index += lowest_set_bit(tree_index);
		}
	}
}

fn lowest_set_bit(value: usize) -> usize {
	value & value.wrapping_neg()
}
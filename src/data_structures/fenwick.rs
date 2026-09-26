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
		let mut fenwick_tree = Self::new(values.len());

		fenwick_tree.tree[1..].copy_from_slice(values);

		for tree_index in 1..fenwick_tree.tree.len() {
			let parent_index = tree_index + lowest_set_bit(tree_index);

			if parent_index < fenwick_tree.tree.len() {
				fenwick_tree.tree[parent_index] += fenwick_tree.tree[tree_index];
			}
		}

		fenwick_tree
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

	pub fn prefix_sum(&self, end: usize) -> usize {
		assert!(
			end < self.tree.len(),
			"Fenwick tree prefix end out of bounds",
		);

		let mut sum = 0;
		let mut tree_index = end;

		while tree_index > 0 {
			sum += self.tree[tree_index];
			tree_index -= lowest_set_bit(tree_index);
		}

		sum
	}
}

fn lowest_set_bit(value: usize) -> usize {
	value & value.wrapping_neg()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prefix_sum_returns_sum_before_end() {
		let tree = FenwickTree::from_slice(&[3, 1, 4, 1, 5, 9]);

		assert_eq!(tree.prefix_sum(0), 0);
		assert_eq!(tree.prefix_sum(1), 3);
		assert_eq!(tree.prefix_sum(2), 4);
		assert_eq!(tree.prefix_sum(3), 8);
		assert_eq!(tree.prefix_sum(4), 9);
		assert_eq!(tree.prefix_sum(5), 14);
		assert_eq!(tree.prefix_sum(6), 23);
	}

	#[test]
	fn add_updates_prefix_sums() {
		let mut tree = FenwickTree::from_slice(&[3, 1, 4, 1]);

		tree.add(2, 2);

		assert_eq!(tree.prefix_sum(2), 4);
		assert_eq!(tree.prefix_sum(3), 10);
		assert_eq!(tree.prefix_sum(4), 11);
	}
}
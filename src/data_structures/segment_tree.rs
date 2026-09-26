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

	pub fn add_range(
		&mut self,
		start: usize,
		end: usize,
		value: usize,
	) {
		assert!(
			start <= end && end <= self.size,
			"segment tree range out of bounds",
		);

		if start == end {
			return;
		}

		self.add_range_recursive(
			0,
			0,
			self.size,
			start,
			end,
			value,
		);
	}

	fn add_range_recursive(
		&mut self,
		node_index: usize,
		node_start: usize,
		node_end: usize,
		range_start: usize,
		range_end: usize,
		value: usize,
	) {
		if range_end <= node_start || range_start >= node_end {
			return;
		}

		if range_start <= node_start && node_end <= range_end {
			self.nodes[node_index].push(value);
			return;
		}

		let middle = node_start + (node_end - node_start) / 2;

		self.add_range_recursive(
			node_index * 2 + 1,
			node_start,
			middle,
			range_start,
			range_end,
			value,
		);

		self.add_range_recursive(
			node_index * 2 + 2,
			middle,
			node_end,
			range_start,
			range_end,
			value,
		);
	}

	pub fn query_point(&self, index: usize) -> Vec<usize> {
		assert!(
			index < self.size,
			"segment tree index out of bounds",
		);

		let mut values = Vec::new();

		self.query_point_recursive(
			0,
			0,
			self.size,
			index,
			&mut values,
		);

		values
	}

	fn query_point_recursive(
		&self,
		node_index: usize,
		node_start: usize,
		node_end: usize,
		index: usize,
		values: &mut Vec<usize>,
	) {
		values.extend_from_slice(&self.nodes[node_index]);

		if node_end - node_start == 1 {
			return;
		}

		let middle = node_start + (node_end - node_start) / 2;

		if index < middle {
			self.query_point_recursive(
				node_index * 2 + 1,
				node_start,
				middle,
				index,
				values,
			);
		} else {
			self.query_point_recursive(
				node_index * 2 + 2,
				middle,
				node_end,
				index,
				values,
			);
		}
	}
}
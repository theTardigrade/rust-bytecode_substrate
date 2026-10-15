use crate::vm::Word;

pub struct HostContext<'a> {
	stack: &'a mut Vec<Word>,
	memory: &'a mut Vec<u8>,
}

impl<'a> HostContext<'a> {
	pub(crate) fn new(
		stack: &'a mut Vec<Word>,
		memory: &'a mut Vec<u8>,
	) -> Self {
		Self {
			stack,
			memory,
		}
	}

	pub fn pop_word(&mut self) -> Word {
		self.stack
			.pop()
			.expect("stack underflow during host call")
	}

	pub fn push_word(&mut self, value: Word) {
		self.stack.push(value);
	}

	pub fn memory(&self) -> &[u8] {
		self.memory
	}

	pub fn memory_mut(&mut self) -> &mut [u8] {
		self.memory
	}
}

pub trait Host {
	fn call(
		&mut self,
		id: Word,
		context: &mut HostContext,
	);
}
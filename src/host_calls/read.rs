use std::io::Read;

use crate::host::HostContext;

pub(super) fn read_into_memory(
	context: &mut HostContext,
	reader: &mut impl Read,
) {
	let max_length = context.pop_word();
	let address = context.pop_word();

	let address = usize::try_from(address)
		.expect("host read address is too large");

	let max_length = usize::try_from(max_length)
		.expect("host read maximum length is too large");

	let end = address
		.checked_add(max_length)
		.expect("host read memory range overflow");

	let buffer = context
		.memory_mut()
		.get_mut(address..end)
		.expect("host read memory range out of bounds");

	let bytes_read = reader
		.read(buffer)
		.expect("failed to read from input stream");

	context.push_word(bytes_read as u64);
}
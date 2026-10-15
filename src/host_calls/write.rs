use std::io::Write;

use crate::host::HostContext;

pub(super) fn write_from_memory(
	context: &mut HostContext,
	writer: &mut impl Write,
) {
	let length = context.pop_word();
	let address = context.pop_word();

	let address = usize::try_from(address)
		.expect("host write address is too large");

	let length = usize::try_from(length)
		.expect("host write length is too large");

	let end = address
		.checked_add(length)
		.expect("host write memory range overflow");

	let bytes = context
		.memory()
		.get(address..end)
		.expect("host write memory range out of bounds");

	writer
		.write_all(bytes)
		.expect("failed to write to output stream");

	writer
		.flush()
		.expect("failed to flush output stream");
}
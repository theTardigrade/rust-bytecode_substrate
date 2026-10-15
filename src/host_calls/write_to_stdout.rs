use std::io;

use crate::host::HostContext;

use super::write;

pub fn call(context: &mut HostContext) {
	let mut stdout = io::stdout().lock();

	write::write_from_memory(context, &mut stdout);
}
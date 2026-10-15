use std::io;

use crate::host::HostContext;

use super::read;

pub fn call(context: &mut HostContext) {
	let mut stdin = io::stdin().lock();

	read::read_into_memory(context, &mut stdin);
}
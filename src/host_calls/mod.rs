mod write;
mod write_to_stdout;
mod write_to_stderr;

use crate::host::{Host, HostContext};
use crate::vm::Word;

use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u64)]
pub enum HostCallId {
	WriteToStdout = 0,
	WriteToStderr = 1,
}

pub struct HostCallDispatcher;

impl Host for HostCallDispatcher {
	fn call(
		&mut self,
		id: Word,
		context: &mut HostContext,
	) {
		let host_call_id = HostCallId::try_from(id)
			.unwrap_or_else(|_| panic!("unknown host-call id: {}", id));

		match host_call_id {
			HostCallId::WriteToStdout => {
				write_to_stdout::call(context);
			}

			HostCallId::WriteToStderr => {
				write_to_stderr::call(context);
			}
		}
	}
}
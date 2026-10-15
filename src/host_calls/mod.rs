mod read;
mod read_from_stdin;
mod terminate_current_process;
mod write;
mod write_to_stdout;
mod write_to_stderr;

use crate::host::{Host, HostContext, HostCallOutcome};
use crate::vm::Word;

use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u64)]
pub enum HostCallId {
	WriteToStdout = 0,
	WriteToStderr = 1,
	ReadFromStdin = 2,
	TerminateCurrentProcess = 255,
}

pub struct HostCallDispatcher;

impl Host for HostCallDispatcher {
	fn call(&mut self, id: Word, context: &mut HostContext) -> HostCallOutcome {
		let host_call_id = HostCallId::try_from(id)
			.unwrap_or_else(|_| panic!("unknown host call ID: {}", id));

		match host_call_id {
			HostCallId::WriteToStdout => {
				write_to_stdout::call(context);
				HostCallOutcome::Continue
			}
			HostCallId::WriteToStderr => {
				write_to_stderr::call(context);
				HostCallOutcome::Continue
			}
			HostCallId::ReadFromStdin => {
				read_from_stdin::call(context);
				HostCallOutcome::Continue
			}
			HostCallId::TerminateCurrentProcess => terminate_current_process::call(context),
		}
	}
}
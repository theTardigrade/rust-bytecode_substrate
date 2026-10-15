use crate::host::{HostCallOutcome, HostContext};

pub fn call(context: &mut HostContext) -> HostCallOutcome {
	HostCallOutcome::Terminate(context.pop_word())
}
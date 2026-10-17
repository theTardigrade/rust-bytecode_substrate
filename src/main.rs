use std::env;
use std::fs;
use std::path::Path;

mod assembler;
mod data_structures;
mod host;
mod host_calls;
mod opcodes;
mod vm;

const ASSEMBLY_FILE_EXTENSION: &str = ".sub.asm";
const BYTECODE_FILE_EXTENSION: &str = ".sub.bc";

fn main() -> std::process::ExitCode {
	let arguments: Vec<String> = env::args().collect();

	if arguments.len() != 2 {
		eprintln!(
			"Usage: {} <program{}|program{}>",
			arguments[0],
			ASSEMBLY_FILE_EXTENSION,
			BYTECODE_FILE_EXTENSION,
		);
		return std::process::ExitCode::FAILURE;
	}

	let path = Path::new(&arguments[1]);
	let path_string = path.to_string_lossy();

	let program = if path_string.ends_with(ASSEMBLY_FILE_EXTENSION) {
		assembler::assemble_file(path)
			.unwrap_or_else(|error| panic!("assembly failed: {}", error))
	} else if path_string.ends_with(BYTECODE_FILE_EXTENSION) {
		fs::read(path)
			.unwrap_or_else(|error| panic!("failed to read {}: {}", path.display(), error))
	} else {
		panic!("expected a {} or {} file", ASSEMBLY_FILE_EXTENSION, BYTECODE_FILE_EXTENSION);
	};

	println!("Bytecode: {:02X?}", program);

	let mut host = host_calls::HostCallDispatcher;
	let result = vm::run_program(&program, &mut host);

	println!("Stack: {:?}", result.stack);

	match result.exit_code {
		Some(code) => {
			let code = u8::try_from(code).expect("process exit code must be between 0 and 255");
			std::process::ExitCode::from(code)
		}
		None => std::process::ExitCode::SUCCESS,
	}
}
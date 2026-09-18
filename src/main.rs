mod opcodes;
mod vm;
mod assembler;

fn main() {
	let source = r#"
		JMP finished

		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0
		PUSH 0

	finished:
		PUSH 99
		HALT
	"#;

	let program = assembler::assemble(source)
		.expect("assembly failed");

	println!("Bytecode: {:02X?}", program);

	let stack = vm::run_program(&program);

	println!("Stack: {:?}", stack);
}
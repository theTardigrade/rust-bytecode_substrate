mod opcodes;
mod vm;
mod assembler;

fn main() {
	let source = r#"
		CALL function
		PUSH 2
		ADD
		HALT

		function:
			PUSH 40
			RET
	"#;

	let program = assembler::assemble(source)
		.expect("assembly failed");

	println!("Bytecode: {:02X?}", program);

	let stack = vm::run_program(&program);

	println!("Stack: {:?}", stack);
}
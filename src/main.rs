mod opcodes;
mod vm;
mod assembler;

fn main() {
	println!("Running: Language Substrate VM");


	let source = r#"
		PUSH 10
		PUSH 20
		ADD
		HALT
	"#;

	let program = assembler::assemble(source).expect("Assembly failed");

	println!("Bytecode: {:02X?}", program);

	let stack = vm::run_program(&program);

	println!("Stack: {:?}", stack);
}
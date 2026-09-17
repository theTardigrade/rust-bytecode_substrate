mod opcodes;
mod vm;

fn main() {
	println!("Running: Language Substrate VM");

	let example_program = [
		0xEA,                         // PUSH_UNSIGNED4  10
		0x12, 0x14,                   // PUSH_UNSIGNED8  20
		0x13, 0xE8, 0x03,             // PUSH_UNSIGNED16 1000
		0x14, 0x40, 0x42, 0x0F, 0x00, // PUSH_UNSIGNED32 1,000,000
		0xFF,                         // HALT
	];

	let stack = vm::run_program(&example_program);

	println!("Stack: {:?}", stack);
}
fn main() {
	println!("Running: Language Substrate VM");

	let example_program =  [
        1, 10, // PUSH 10
        255,   // HALT
    ];

	let mut stack: Vec<i64> = Vec::new();
	let mut ip: usize = 0; // instruction pointer

	loop {
		let opcode = example_program[ip];
		ip += 1;

		match opcode {
			1 => {
				let value = example_program[ip] as i64;
				ip += 1;

				stack.push(value);
			}

			255 => {
				break;
			}

			_ => {
				panic!("Unknown opcode: {}", opcode);
			}
		}
	}

	println!("Stack: {:?}", stack);
}

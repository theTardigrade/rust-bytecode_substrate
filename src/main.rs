use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum Opcode {
	Noop = 0x0,
	Push = 0x1,
	Add  = 0x2,
	Halt = 0xe,
}

fn get_next_nibble_from_program(program: &[u8], ip: &mut usize) -> u8 {
    let byte = program[*ip / 2];

    let nibble = if *ip % 2 == 0 {
        byte >> 4
    } else {
        byte & 0x0F
    };

    *ip += 1;

    nibble
}

fn main() {
	println!("Running: Language Substrate VM");

	let example_program = [
		0x1, 0x0A, // PUSH 10
		0x1, 0x14, // PUSH 20
		0x2,       // ADD
		0xE,       // HALT
	];

	let mut stack: Vec<i64> = Vec::with_capacity(1 << 12);
	let mut ip: usize = 0; // instruction pointer, counts in nibbles

	loop {
		let raw_opcode = get_next_nibble_from_program(&example_program, &mut ip);
	
		let opcode = Opcode::try_from(raw_opcode)
			.expect("Unknown opcode");

		match opcode {
			Opcode::Noop => {}

			Opcode::Push => {
				let high = get_next_nibble_from_program(&example_program, &mut ip);
				let low = get_next_nibble_from_program(&example_program, &mut ip);

				let byte_value = ((high << 4) | low) as i64;

				stack.push(byte_value);
			}

			Opcode::Add => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a + b);
			}

			Opcode::Halt => {
				break;
			}
		}
	}

	println!("Stack: {:?}", stack);
}

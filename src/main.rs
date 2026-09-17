use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum Opcode {
	Noop  = 0x0,
	Push4 = 0x1,
	Add   = 0x2,
	Halt  = 0xE,
	Ext   = 0xF,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum ExtendedOpcode {
	Push8  = 0x0,
	Push16 = 0x1,
	Push32 = 0x2,
	Push64 = 0x3,
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

fn get_next_byte_from_program(program: &[u8], ip: &mut usize) -> u8 {
	let high = get_next_nibble_from_program(program, ip);
	let low = get_next_nibble_from_program(program, ip);

	(high << 4) | low
}

fn get_next_u16_from_program(program: &[u8], ip: &mut usize) -> u16 {
	let bytes = [
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
	];

	u16::from_le_bytes(bytes)
}

fn get_next_u32_from_program(program: &[u8], ip: &mut usize) -> u32 {
	let bytes = [
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
	];

	u32::from_le_bytes(bytes)
}

fn get_next_u64_from_program(program: &[u8], ip: &mut usize) -> u64 {
	let bytes = [
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
	];

	u64::from_le_bytes(bytes)
}

fn main() {
	println!("Running: Language Substrate VM");

	let example_program = [
		0x1A,       // PUSH4 10
		0xF0, 0x14, // PUSH8 20
		0x2E,       // ADD, HALT
	];

	let mut stack: Vec<u64> = Vec::with_capacity(1 << 12);
	let mut ip: usize = 0; // instruction pointer, counts in nibbles

	loop {
		let raw_opcode = get_next_nibble_from_program(&example_program, &mut ip);
		let opcode = Opcode::try_from(raw_opcode).expect("Unknown opcode");

		match opcode {
			Opcode::Noop => {}

			Opcode::Push4 => {
				let value = get_next_nibble_from_program(&example_program, &mut ip);
				stack.push(value as u64);
			}

			Opcode::Add => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a + b);
			}

			Opcode::Halt => {
				break;
			}

			Opcode::Ext => {
				if ip % 2 == 0 {
					panic!("Extended opcode is not byte-aligned");
				}

				let raw_extended_opcode = get_next_nibble_from_program(&example_program, &mut ip);
				let extended_opcode = ExtendedOpcode::try_from(raw_extended_opcode).expect("Unknown extended opcode");

				match extended_opcode {
					ExtendedOpcode::Push8 => {
						let value = get_next_byte_from_program(&example_program, &mut ip);
						stack.push(value as u64);
					}

					ExtendedOpcode::Push16 => {
						let value = get_next_u16_from_program(&example_program, &mut ip);
						stack.push(value as u64);
					}

					ExtendedOpcode::Push32 => {
						let value = get_next_u32_from_program(&example_program, &mut ip);
						stack.push(value as u64);
					}

					ExtendedOpcode::Push64 => {
						let value = get_next_u64_from_program(&example_program, &mut ip);
						stack.push(value);
					}
				}
			}
		}
	}

	println!("Stack: {:?}", stack);
}
use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum OpcodeNibble {
	Noop  = 0x0,
	Push4 = 0x1,
	Add   = 0x2,
	Sub   = 0x3,
	Mul   = 0x4,
	Halt  = 0xE,
	Ext   = 0xF,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum ExtendedOpcodeNibble {
	Push8            = 0x0,
	Push16           = 0x1,
	Push32           = 0x2,
	Push64           = 0x3,
	LessThanUnsigned = 0x4,
	LessThanSigned   = 0x5,
	DivUnsigned      = 0x6,
	DivSigned        = 0x7,
}

type Word = u64;

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
		0x1A,                         // PUSH4  10
		0xF0, 0x14,                   // PUSH8  20
		0xF1, 0xE8, 0x03,             // PUSH16 1000
		0xF2, 0x40, 0x42, 0x0F, 0x00, // PUSH32 1,000,000
		0xE0,                         // HALT, padding
	];

	let mut stack: Vec<Word> = Vec::with_capacity(1 << 12);
	let mut ip: usize = 0; // instruction pointer, counts in nibbles

	loop {
		let raw_opcode_nibble = get_next_nibble_from_program(&example_program, &mut ip);
		let opcode_nibble = OpcodeNibble::try_from(raw_opcode_nibble).expect("Unknown opcode_nibble");

		match opcode_nibble {
			OpcodeNibble::Noop => {}

			OpcodeNibble::Push4 => {
				let value = get_next_nibble_from_program(&example_program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeNibble::Add => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a + b);
			}

			OpcodeNibble::Sub => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a.wrapping_sub(b));
			}

			OpcodeNibble::Mul => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a.wrapping_mul(b));
			}

			OpcodeNibble::Halt => {
				break;
			}

			OpcodeNibble::Ext => {
				if ip % 2 == 0 {
					panic!("Extended opcode_nibble is not byte-aligned");
				}

				let raw_extended_opcode_nibble = get_next_nibble_from_program(&example_program, &mut ip);
				let extended_opcode_nibble = ExtendedOpcodeNibble::try_from(raw_extended_opcode_nibble)
					.expect("Unknown extended opcode_nibble");

				match extended_opcode_nibble {
					ExtendedOpcodeNibble::Push8 => {
						let value = get_next_byte_from_program(&example_program, &mut ip);
						stack.push(value as Word);
					}

					ExtendedOpcodeNibble::Push16 => {
						let value = get_next_u16_from_program(&example_program, &mut ip);
						stack.push(value as Word);
					}

					ExtendedOpcodeNibble::Push32 => {
						let value = get_next_u32_from_program(&example_program, &mut ip);
						stack.push(value as Word);
					}

					ExtendedOpcodeNibble::Push64 => {
						let value = get_next_u64_from_program(&example_program, &mut ip);
						stack.push(value as Word);
					}

					ExtendedOpcodeNibble::LessThanUnsigned => {
						let b = stack.pop().expect("Stack underflow");
						let a = stack.pop().expect("Stack underflow");

						stack.push(if a < b { 1 } else { 0 });
					}

					ExtendedOpcodeNibble::LessThanSigned => {
						let b = stack.pop().expect("Stack underflow") as i64;
						let a = stack.pop().expect("Stack underflow") as i64;

						stack.push(if a < b { 1 } else { 0 });
					}

					ExtendedOpcodeNibble::DivUnsigned => {
						let b = stack.pop().expect("Stack underflow");
						let a = stack.pop().expect("Stack underflow");

						if b == 0 {
							panic!("Division by zero");
						}

						stack.push(a / b);
					}

					ExtendedOpcodeNibble::DivSigned => {
						let b = stack.pop().expect("Stack underflow") as i64;
						let a = stack.pop().expect("Stack underflow") as i64;

						if b == 0 {
							panic!("Division by zero");
						}

						if a == i64::MIN && b == -1 {
							stack.push(i64::MIN as Word);
						} else {
							stack.push((a / b) as Word);
						}
					}
				}
			}
		}
	}

	println!("Stack: {:?}", stack);
}
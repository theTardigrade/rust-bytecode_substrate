mod opcodes;

use opcodes::{
	OpcodeByte,
	CALL_REL_SIGNED4_START,
	CALL_REL_SIGNED4_END,
	JUMP_IF_ZERO_REL_SIGNED4_START,
	JUMP_IF_ZERO_REL_SIGNED4_END,
	JUMP_REL_SIGNED4_START,
	JUMP_REL_SIGNED4_END,
	PUSH_UNSIGNED4_START,
	PUSH_UNSIGNED4_END,
};

type Word = u64;

fn get_next_byte_from_program(program: &[u8], ip: &mut usize) -> u8 {
	let byte = program[*ip];
	*ip += 1;

	byte
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

fn get_next_i8_from_program(program: &[u8], ip: &mut usize) -> i8 {
	get_next_byte_from_program(program, ip) as i8
}

fn get_next_i16_from_program(program: &[u8], ip: &mut usize) -> i16 {
	let bytes = [
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
	];

	i16::from_le_bytes(bytes)
}

fn get_next_i32_from_program(program: &[u8], ip: &mut usize) -> i32 {
	let bytes = [
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
		get_next_byte_from_program(program, ip),
	];

	i32::from_le_bytes(bytes)
}

fn get_next_i64_from_program(program: &[u8], ip: &mut usize) -> i64 {
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

	i64::from_le_bytes(bytes)
}

fn decode_u4(value: u8) -> u8 {
	value & 0x0F
}

fn decode_i4(value: u8) -> i8 {
	let value = decode_u4(value);

	if value & 0x08 != 0 {
		value as i8 - 16
	} else {
		value as i8
	}
}

fn add_relative_offset(ip: usize, offset: i64) -> usize {
	if offset >= 0 {
		let offset = usize::try_from(offset).expect("Relative offset is too large for this platform");
		ip.checked_add(offset).expect("Control-flow target is out of range")
	} else {
		let offset = usize::try_from(offset.unsigned_abs()).expect("Relative offset is too large for this platform");
		ip.checked_sub(offset).expect("Control-flow target is out of range")
	}
}

fn get_memory_range(memory: &[u8], address: Word, length: usize) -> std::ops::Range<usize> {
	let start = usize::try_from(address).expect("Memory address is too large for this platform");
	let end = start.checked_add(length).expect("Memory access is out of range");

	if end > memory.len() {
		panic!("Memory access is out of bounds");
	}

	start..end
}

fn main() {
	println!("Running: Language Substrate VM");

	let example_program = [
		0xEA,                         // PUSH_UNSIGNED4  10
		0x12, 0x14,                   // PUSH_UNSIGNED8  20
		0x13, 0xE8, 0x03,             // PUSH_UNSIGNED16 1000
		0x14, 0x40, 0x42, 0x0F, 0x00, // PUSH_UNSIGNED32 1,000,000
		0xFF,                         // HALT
	];

	let mut stack: Vec<Word> = Vec::with_capacity(1 << 12);
	let mut call_stack: Vec<usize> = Vec::with_capacity(1 << 12);
	let mut memory: Vec<u8> = Vec::new();

	let mut ip: usize = 0; // instruction pointer, counts in bytes

	loop {
		let raw_opcode_byte = get_next_byte_from_program(&example_program, &mut ip);

		if raw_opcode_byte >= PUSH_UNSIGNED4_START && raw_opcode_byte <= PUSH_UNSIGNED4_END {
			let value = decode_u4(raw_opcode_byte);
			stack.push(value as Word);
			continue;
		}

		if raw_opcode_byte >= JUMP_REL_SIGNED4_START && raw_opcode_byte <= JUMP_REL_SIGNED4_END {
			let offset = decode_i4(raw_opcode_byte);
			ip = add_relative_offset(ip, offset as i64);
			continue;
		}

		if raw_opcode_byte >= JUMP_IF_ZERO_REL_SIGNED4_START && raw_opcode_byte <= JUMP_IF_ZERO_REL_SIGNED4_END {
			let offset = decode_i4(raw_opcode_byte);
			let condition = stack.pop().expect("Stack underflow");

			if condition == 0 {
				ip = add_relative_offset(ip, offset as i64);
			}

			continue;
		}

		if raw_opcode_byte >= CALL_REL_SIGNED4_START && raw_opcode_byte <= CALL_REL_SIGNED4_END {
			let offset = decode_i4(raw_opcode_byte);

			call_stack.push(ip);
			ip = add_relative_offset(ip, offset as i64);
			continue;
		}

		let opcode_byte = OpcodeByte::try_from(raw_opcode_byte).expect("Unknown opcode byte");

		match opcode_byte {
			OpcodeByte::Noop => {}

			OpcodeByte::Add => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a.wrapping_add(b));
			}

			OpcodeByte::Sub => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a.wrapping_sub(b));
			}

			OpcodeByte::Mul => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a.wrapping_mul(b));
			}

			OpcodeByte::DivUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				if b == 0 {
					panic!("Division by zero");
				}

				stack.push(a / b);
			}

			OpcodeByte::DivSigned => {
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

			OpcodeByte::RemUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				if b == 0 {
					panic!("Remainder by zero");
				}

				stack.push(a % b);
			}

			OpcodeByte::RemSigned => {
				let b = stack.pop().expect("Stack underflow") as i64;
				let a = stack.pop().expect("Stack underflow") as i64;

				if b == 0 {
					panic!("Remainder by zero");
				}

				if a == i64::MIN && b == -1 {
					stack.push(0);
				} else {
					stack.push((a % b) as Word);
				}
			}

			OpcodeByte::BitAnd => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a & b);
			}

			OpcodeByte::BitOr => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a | b);
			}

			OpcodeByte::BitXor => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(a ^ b);
			}

			OpcodeByte::BitNot => {
				let a = stack.pop().expect("Stack underflow");
				stack.push(!a);
			}

			OpcodeByte::ShiftLeft => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");
				let shift = b & 0x3F;

				stack.push(a << shift);
			}

			OpcodeByte::LogicShiftRight => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");
				let shift = b & 0x3F;

				stack.push(a >> shift);
			}

			OpcodeByte::ArithShiftRight => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow") as i64;
				let shift = b & 0x3F;

				stack.push((a >> shift) as Word);
			}

			OpcodeByte::Dup => {
				let value = *stack.last().expect("Stack underflow");
				stack.push(value);
			}

			OpcodeByte::Drop => {
				stack.pop().expect("Stack underflow");
			}

			OpcodeByte::Swap => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(b);
				stack.push(a);
			}

			OpcodeByte::PushUnsigned8 => {
				let value = get_next_byte_from_program(&example_program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::PushUnsigned16 => {
				let value = get_next_u16_from_program(&example_program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::PushUnsigned32 => {
				let value = get_next_u32_from_program(&example_program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::Push64 => {
				let value = get_next_u64_from_program(&example_program, &mut ip);
				stack.push(value);
			}

			OpcodeByte::CompEqual => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a == b { 1 } else { 0 });
			}

			OpcodeByte::CompNotEqual => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a != b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a < b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanSigned => {
				let b = stack.pop().expect("Stack underflow") as i64;
				let a = stack.pop().expect("Stack underflow") as i64;

				stack.push(if a < b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanOrEqualUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a <= b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanOrEqualSigned => {
				let b = stack.pop().expect("Stack underflow") as i64;
				let a = stack.pop().expect("Stack underflow") as i64;

				stack.push(if a <= b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a > b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanSigned => {
				let b = stack.pop().expect("Stack underflow") as i64;
				let a = stack.pop().expect("Stack underflow") as i64;

				stack.push(if a > b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanOrEqualUnsigned => {
				let b = stack.pop().expect("Stack underflow");
				let a = stack.pop().expect("Stack underflow");

				stack.push(if a >= b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanOrEqualSigned => {
				let b = stack.pop().expect("Stack underflow") as i64;
				let a = stack.pop().expect("Stack underflow") as i64;

				stack.push(if a >= b { 1 } else { 0 });
			}

			OpcodeByte::JumpRelSigned8 => {
				let offset = get_next_i8_from_program(&example_program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned16 => {
				let offset = get_next_i16_from_program(&example_program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned32 => {
				let offset = get_next_i32_from_program(&example_program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned64 => {
				let offset = get_next_i64_from_program(&example_program, &mut ip);
				ip = add_relative_offset(ip, offset);
			}

			OpcodeByte::JumpIfZeroRelSigned8 => {
				let offset = get_next_i8_from_program(&example_program, &mut ip);
				let condition = stack.pop().expect("Stack underflow");

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned16 => {
				let offset = get_next_i16_from_program(&example_program, &mut ip);
				let condition = stack.pop().expect("Stack underflow");

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned32 => {
				let offset = get_next_i32_from_program(&example_program, &mut ip);
				let condition = stack.pop().expect("Stack underflow");

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned64 => {
				let offset = get_next_i64_from_program(&example_program, &mut ip);
				let condition = stack.pop().expect("Stack underflow");

				if condition == 0 {
					ip = add_relative_offset(ip, offset);
				}
			}

			OpcodeByte::CallRelSigned8 => {
				let offset = get_next_i8_from_program(&example_program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned16 => {
				let offset = get_next_i16_from_program(&example_program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned32 => {
				let offset = get_next_i32_from_program(&example_program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned64 => {
				let offset = get_next_i64_from_program(&example_program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset);
			}

			OpcodeByte::JumpAbsIndirect => {
				let target = stack.pop().expect("Stack underflow");
				ip = usize::try_from(target).expect("Jump target is too large for this platform");
			}

			OpcodeByte::CallAbsIndirect => {
				let target = stack.pop().expect("Stack underflow");
				let target = usize::try_from(target).expect("Call target is too large for this platform");

				call_stack.push(ip);
				ip = target;
			}

			OpcodeByte::Return => {
				ip = call_stack.pop().expect("Call stack underflow");
			}

			OpcodeByte::MemLoad8 => {
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 1);

				stack.push(memory[range.start] as Word);
			}

			OpcodeByte::MemLoad16 => {
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 2);
				let bytes = [
					memory[range.start],
					memory[range.start + 1],
				];

				stack.push(u16::from_le_bytes(bytes) as Word);
			}

			OpcodeByte::MemLoad32 => {
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 4);
				let bytes = [
					memory[range.start],
					memory[range.start + 1],
					memory[range.start + 2],
					memory[range.start + 3],
				];

				stack.push(u32::from_le_bytes(bytes) as Word);
			}

			OpcodeByte::MemLoad64 => {
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 8);
				let bytes = [
					memory[range.start],
					memory[range.start + 1],
					memory[range.start + 2],
					memory[range.start + 3],
					memory[range.start + 4],
					memory[range.start + 5],
					memory[range.start + 6],
					memory[range.start + 7],
				];

				stack.push(u64::from_le_bytes(bytes));
			}

			OpcodeByte::MemStore8 => {
				let value = stack.pop().expect("Stack underflow");
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 1);

				memory[range.start] = value as u8;
			}

			OpcodeByte::MemStore16 => {
				let value = stack.pop().expect("Stack underflow");
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 2);
				let bytes = (value as u16).to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemStore32 => {
				let value = stack.pop().expect("Stack underflow");
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 4);
				let bytes = (value as u32).to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemStore64 => {
				let value = stack.pop().expect("Stack underflow");
				let address = stack.pop().expect("Stack underflow");
				let range = get_memory_range(&memory, address, 8);
				let bytes = value.to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemSize => {
				stack.push(memory.len() as Word);
			}

			OpcodeByte::MemGrow => {
				let amount = stack.pop().expect("Stack underflow");
				let amount = usize::try_from(amount).expect("Memory growth amount is too large for this platform");

				let old_size = memory.len();
				let new_size = old_size.checked_add(amount).expect("Memory size overflow");

				memory.resize(new_size, 0);

				stack.push(old_size as Word);
			}

			OpcodeByte::Halt => {
				break;
			}
		}
	}

	println!("Stack: {:?}", stack);
}
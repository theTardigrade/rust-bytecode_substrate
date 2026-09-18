use crate::opcodes::{
	OpcodeByte,
	CALL_REL_SIGNED4_START,
	CALL_REL_SIGNED4_END,
	JUMP_IF_ZERO_REL_SIGNED4_START,
	JUMP_IF_ZERO_REL_SIGNED4_END,
	JUMP_REL_SIGNED4_START,
	JUMP_REL_SIGNED4_END,
	PUSH_UNSIGNED4_START,
	PUSH_UNSIGNED4_END,
	PUSH_SIGNED4_START,
	PUSH_SIGNED4_END,
};

pub type Word = u64;

fn pop_word(stack: &mut Vec<Word>) -> Word {
	stack.pop().expect("stack underflow")
}

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
		let offset = usize::try_from(offset).expect("relative offset is too large for this platform");
		ip.checked_add(offset).expect("control-flow target is out of range")
	} else {
		let offset = usize::try_from(offset.unsigned_abs()).expect("relative offset is too large for this platform");
		ip.checked_sub(offset).expect("control-flow target is out of range")
	}
}

fn get_memory_range(memory: &[u8], address: Word, length: usize) -> std::ops::Range<usize> {
	let start = usize::try_from(address).expect("memory address is too large for this platform");
	let end = start.checked_add(length).expect("memory access is out of range");

	if end > memory.len() {
		panic!("memory access is out of bounds");
	}

	start..end
}

pub fn run_program(program: &[u8]) -> Vec<Word> {
	let mut stack: Vec<Word> = Vec::with_capacity(1 << 12);
	let mut call_stack: Vec<usize> = Vec::with_capacity(1 << 12);
	let mut memory: Vec<u8> = Vec::new();
	let mut ip: usize = 0;

	loop {
		let raw_opcode_byte = get_next_byte_from_program(program, &mut ip);

		if (PUSH_UNSIGNED4_START..=PUSH_UNSIGNED4_END).contains(&raw_opcode_byte) {
			let value = decode_u4(raw_opcode_byte);
			stack.push(value as Word);
			continue;
		}

		if (PUSH_SIGNED4_START..=PUSH_SIGNED4_END).contains(&raw_opcode_byte) {
			let value = decode_i4(raw_opcode_byte);
			stack.push(value as i64 as Word);
			continue;
		}

		if (JUMP_REL_SIGNED4_START..=JUMP_REL_SIGNED4_END).contains(&raw_opcode_byte) {
			let offset = decode_i4(raw_opcode_byte);
			ip = add_relative_offset(ip, offset as i64);
			continue;
		}

		if (JUMP_IF_ZERO_REL_SIGNED4_START..=JUMP_IF_ZERO_REL_SIGNED4_END).contains(&raw_opcode_byte) {
			let offset = decode_i4(raw_opcode_byte);
			let condition = pop_word(&mut stack);

			if condition == 0 {
				ip = add_relative_offset(ip, offset as i64);
			}

			continue;
		}

		if (CALL_REL_SIGNED4_START..=CALL_REL_SIGNED4_END).contains(&raw_opcode_byte) {
			let offset = decode_i4(raw_opcode_byte);

			call_stack.push(ip);
			ip = add_relative_offset(ip, offset as i64);
			continue;
		}

		let opcode = OpcodeByte::try_from(raw_opcode_byte).expect("unknown opcode byte");

		match opcode {
			OpcodeByte::Noop => {}

			OpcodeByte::Add => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a.wrapping_add(b));
			}

			OpcodeByte::Sub => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a.wrapping_sub(b));
			}

			OpcodeByte::Mul => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a.wrapping_mul(b));
			}

			OpcodeByte::DivUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				if b == 0 {
					panic!("division by zero");
				}

				stack.push(a / b);
			}

			OpcodeByte::DivSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				if b == 0 {
					panic!("division by zero");
				}

				if a == i64::MIN && b == -1 {
					stack.push(i64::MIN as Word);
				} else {
					stack.push((a / b) as Word);
				}
			}

			OpcodeByte::RemUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				if b == 0 {
					panic!("remainder by zero");
				}

				stack.push(a % b);
			}

			OpcodeByte::RemSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				if b == 0 {
					panic!("remainder by zero");
				}

				if a == i64::MIN && b == -1 {
					stack.push(0);
				} else {
					stack.push((a % b) as Word);
				}
			}

			OpcodeByte::BitAnd => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a & b);
			}

			OpcodeByte::BitOr => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a | b);
			}

			OpcodeByte::BitXor => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(a ^ b);
			}

			OpcodeByte::BitNot => {
				let a = pop_word(&mut stack);

				stack.push(!a);
			}

			OpcodeByte::ShiftLeft => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);
				let shift = b & 0x3F;

				stack.push(a << shift);
			}

			OpcodeByte::LogicShiftRight => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);
				let shift = b & 0x3F;

				stack.push(a >> shift);
			}

			OpcodeByte::ArithShiftRight => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack) as i64;
				let shift = b & 0x3F;

				stack.push((a >> shift) as Word);
			}

			OpcodeByte::Dup => {
				let value = *stack.last().expect("stack underflow");
				stack.push(value);
			}

			OpcodeByte::Pop => {
				pop_word(&mut stack);
			}

			OpcodeByte::Swap => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(b);
				stack.push(a);
			}

			OpcodeByte::PushUnsigned8 => {
				let value = get_next_byte_from_program(program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::PushSigned8 => {
				let value = get_next_i8_from_program(program, &mut ip);
				stack.push(value as i64 as Word);
			}

			OpcodeByte::PushUnsigned16 => {
				let value = get_next_u16_from_program(program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::PushSigned16 => {
				let value = get_next_i16_from_program(program, &mut ip);
				stack.push(value as i64 as Word);
			}

			OpcodeByte::PushUnsigned32 => {
				let value = get_next_u32_from_program(program, &mut ip);
				stack.push(value as Word);
			}

			OpcodeByte::PushSigned32 => {
				let value = get_next_i32_from_program(program, &mut ip);
				stack.push(value as i64 as Word);
			}

			OpcodeByte::Push64 => {
				let value = get_next_u64_from_program(program, &mut ip);
				stack.push(value);
			}

			OpcodeByte::CompEqual => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a == b { 1 } else { 0 });
			}

			OpcodeByte::CompNotEqual => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a != b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a < b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				stack.push(if a < b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanOrEqualUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a <= b { 1 } else { 0 });
			}

			OpcodeByte::CompLessThanOrEqualSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				stack.push(if a <= b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a > b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				stack.push(if a > b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanOrEqualUnsigned => {
				let b = pop_word(&mut stack);
				let a = pop_word(&mut stack);

				stack.push(if a >= b { 1 } else { 0 });
			}

			OpcodeByte::CompGreaterThanOrEqualSigned => {
				let b = pop_word(&mut stack) as i64;
				let a = pop_word(&mut stack) as i64;

				stack.push(if a >= b { 1 } else { 0 });
			}

			OpcodeByte::JumpRelSigned8 => {
				let offset = get_next_i8_from_program(program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned16 => {
				let offset = get_next_i16_from_program(program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned32 => {
				let offset = get_next_i32_from_program(program, &mut ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::JumpRelSigned64 => {
				let offset = get_next_i64_from_program(program, &mut ip);
				ip = add_relative_offset(ip, offset);
			}

			OpcodeByte::JumpIfZeroRelSigned8 => {
				let offset = get_next_i8_from_program(program, &mut ip);
				let condition = pop_word(&mut stack);

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned16 => {
				let offset = get_next_i16_from_program(program, &mut ip);
				let condition = pop_word(&mut stack);

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned32 => {
				let offset = get_next_i32_from_program(program, &mut ip);
				let condition = pop_word(&mut stack);

				if condition == 0 {
					ip = add_relative_offset(ip, offset as i64);
				}
			}

			OpcodeByte::JumpIfZeroRelSigned64 => {
				let offset = get_next_i64_from_program(program, &mut ip);
				let condition = pop_word(&mut stack);

				if condition == 0 {
					ip = add_relative_offset(ip, offset);
				}
			}

			OpcodeByte::CallRelSigned8 => {
				let offset = get_next_i8_from_program(program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned16 => {
				let offset = get_next_i16_from_program(program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned32 => {
				let offset = get_next_i32_from_program(program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset as i64);
			}

			OpcodeByte::CallRelSigned64 => {
				let offset = get_next_i64_from_program(program, &mut ip);

				call_stack.push(ip);
				ip = add_relative_offset(ip, offset);
			}

			OpcodeByte::JumpAbsIndirect => {
				let target = pop_word(&mut stack);
				let target = usize::try_from(target).expect("jump target is too large for this platform");

				ip = target;
			}

			OpcodeByte::CallAbsIndirect => {
				let target = pop_word(&mut stack);
				let target = usize::try_from(target).expect("call target is too large for this platform");

				call_stack.push(ip);
				ip = target;
			}

			OpcodeByte::Return => {
				ip = call_stack.pop().expect("call stack underflow");
			}

			OpcodeByte::MemLoad8 => {
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 1);

				stack.push(memory[range.start] as Word);
			}

			OpcodeByte::MemLoad16 => {
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 2);

				let bytes = [
					memory[range.start],
					memory[range.start + 1],
				];

				stack.push(u16::from_le_bytes(bytes) as Word);
			}

			OpcodeByte::MemLoad32 => {
				let address = pop_word(&mut stack);
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
				let address = pop_word(&mut stack);
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
				let value = pop_word(&mut stack);
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 1);

				memory[range.start] = value as u8;
			}

			OpcodeByte::MemStore16 => {
				let value = pop_word(&mut stack);
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 2);
				let bytes = (value as u16).to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemStore32 => {
				let value = pop_word(&mut stack);
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 4);
				let bytes = (value as u32).to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemStore64 => {
				let value = pop_word(&mut stack);
				let address = pop_word(&mut stack);
				let range = get_memory_range(&memory, address, 8);
				let bytes = value.to_le_bytes();

				memory[range].copy_from_slice(&bytes);
			}

			OpcodeByte::MemSize => {
				stack.push(memory.len() as Word);
			}

			OpcodeByte::MemGrow => {
				let amount = pop_word(&mut stack);
				let amount = usize::try_from(amount).expect("memory growth amount is too large for this platform");
				let old_size = memory.len();
				let new_size = old_size.checked_add(amount).expect("memory size is too large for this platform");

				memory.resize(new_size, 0);
				stack.push(old_size as Word);
			}

			OpcodeByte::MemCopy8 => {
				let length = pop_word(&mut stack);
				let source = pop_word(&mut stack);
				let destination = pop_word(&mut stack);

				let length = usize::try_from(length).expect("memory copy length is too large for this platform");
				let source_range = get_memory_range(&memory, source, length);
				let destination_range = get_memory_range(&memory, destination, length);

				memory.copy_within(source_range, destination_range.start);
			}

			OpcodeByte::MemFill8 => {
				let length = pop_word(&mut stack);
				let value = pop_word(&mut stack);
				let address = pop_word(&mut stack);

				let length = usize::try_from(length).expect("memory fill length is too large for this platform");
				let range = get_memory_range(&memory, address, length);

				memory[range].fill(value as u8);
			}

			OpcodeByte::Halt => {
				break;
			}
		}
	}

	stack
}
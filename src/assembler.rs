use crate::opcodes::{
	OpcodeByte,
	PUSH_UNSIGNED4_START,
	PUSH_UNSIGNED4_END,
	PUSH_SIGNED4_START,
	PUSH_SIGNED4_END,
};

enum PushValue {
	Unsigned(u64),
	Signed(i64),
}

enum AssemblyInstruction {
	Push(PushValue),
	Add,
	Halt,
}

pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
	let instructions = parse_source(source)?;
	let mut program = Vec::new();

	for instruction in &instructions {
		emit_instruction(&mut program, instruction);
	}

	Ok(program)
}

fn parse_source(source: &str) -> Result<Vec<AssemblyInstruction>, String> {
	let mut instructions = Vec::new();

	for line in source.lines() {
		let line = line.trim();

		if line.is_empty() {
			continue;
		}

		let instruction = parse_instruction(line)?;
		instructions.push(instruction);
	}

	Ok(instructions)
}

fn parse_instruction(line: &str) -> Result<AssemblyInstruction, String> {
	let parts: Vec<&str> = line.split_whitespace().collect();

	match parts[0] {
		"PUSH" => {
			let text = parts[1];

			if text.starts_with('-') {
				let value = text
					.parse::<i64>()
					.map_err(|_| format!("Invalid signed value: {}", text))?;

				Ok(AssemblyInstruction::Push(
					PushValue::Signed(value)
				))
			} else {
				let value = text
					.parse::<u64>()
					.map_err(|_| format!("Invalid unsigned value: {}", text))?;

				Ok(AssemblyInstruction::Push(
					PushValue::Unsigned(value)
				))
			}
		}

		"ADD" => Ok(AssemblyInstruction::Add),

		"HALT" => Ok(AssemblyInstruction::Halt),

		_ => Err(format!("Unknown instruction: {}", parts[0])),
	}
}

fn emit_instruction(program: &mut Vec<u8>, instruction: &AssemblyInstruction) {
	match instruction {
		AssemblyInstruction::Push(push_value) => {
			match push_value {
				PushValue::Unsigned(unsigned_value) => {
					if *unsigned_value <= (PUSH_UNSIGNED4_END - PUSH_UNSIGNED4_START) as u64 {
						program.push(PUSH_UNSIGNED4_START + *unsigned_value as u8);
					} else if *unsigned_value <= u8::MAX as u64 {
						program.push(OpcodeByte::PushUnsigned8 as u8);
						program.push(*unsigned_value as u8);
					} else if *unsigned_value <= u16::MAX as u64 {
						program.push(OpcodeByte::PushUnsigned16 as u8);
						program.extend_from_slice(&(*unsigned_value as u16).to_le_bytes());
					} else if *unsigned_value <= u32::MAX as u64 {
						program.push(OpcodeByte::PushUnsigned32 as u8);
						program.extend_from_slice(&(*unsigned_value as u32).to_le_bytes());
					} else {
						program.push(OpcodeByte::Push64 as u8);
						program.extend_from_slice(&unsigned_value.to_le_bytes());
					}
				}

				PushValue::Signed(signed_value) => {
					let signed4_mask = PUSH_SIGNED4_END - PUSH_SIGNED4_START;
					let signed4_count = (signed4_mask + 1) as i64;
					let signed4_half = signed4_count / 2;

					if *signed_value >= -signed4_half && *signed_value < signed4_half {
						let encoded_value = (*signed_value as i8 as u8) & signed4_mask;
						program.push(PUSH_SIGNED4_START + encoded_value);
					} else if *signed_value >= i8::MIN as i64 && *signed_value <= i8::MAX as i64 {
						program.push(OpcodeByte::PushSigned8 as u8);
						program.push(*signed_value as i8 as u8);
					} else if *signed_value >= i16::MIN as i64 && *signed_value <= i16::MAX as i64 {
						program.push(OpcodeByte::PushSigned16 as u8);
						program.extend_from_slice(&(*signed_value as i16).to_le_bytes());
					} else if *signed_value >= i32::MIN as i64 && *signed_value <= i32::MAX as i64 {
						program.push(OpcodeByte::PushSigned32 as u8);
						program.extend_from_slice(&(*signed_value as i32).to_le_bytes());
					} else {
						program.push(OpcodeByte::Push64 as u8);
						program.extend_from_slice(&signed_value.to_le_bytes());
					}
				}
			}
		}

		AssemblyInstruction::Add => {
			program.push(OpcodeByte::Add as u8);
		}

		AssemblyInstruction::Halt => {
			program.push(OpcodeByte::Halt as u8);
		}
	}
}


use crate::opcodes::{
	OpcodeByte,
	PUSH_UNSIGNED4_START,
	PUSH_UNSIGNED4_END,
	PUSH_SIGNED4_START,
	PUSH_SIGNED4_END,
};

enum IntegerLiteral {
	Unsigned(u64),
	Signed(i64),
}

enum AssemblyInstruction {
	Push(IntegerLiteral),
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

	if parts.is_empty() {
		return Err("expected instruction".to_string());
	}

	match parts[0] {
		"PUSH" => {
			if parts.len() != 2 {
				return Err("PUSH expects exactly one operand".to_string());
			}

			let integer_literal = parse_integer_literal(parts[1])?;

			Ok(AssemblyInstruction::Push(integer_literal))
		}

		"ADD" => {
			if parts.len() != 1 {
				return Err("ADD does not take any operands".to_string());
			}

			Ok(AssemblyInstruction::Add)
		}

		"HALT" => {
			if parts.len() != 1 {
				return Err("HALT does not take any operands".to_string());
			}

			Ok(AssemblyInstruction::Halt)
		}

		_ => Err(format!("unknown instruction: {}", parts[0])),
	}
}

fn parse_integer_literal(text: &str) -> Result<IntegerLiteral, String> {
	let text_without_separators = text.replace('_', "");

	let (negative, unsigned_text) = if let Some(rest) = text_without_separators.strip_prefix('-') {
		(true, rest)
	} else {
		(false, text_without_separators.as_str())
	};

	let (radix, digits) = if let Some(rest) = unsigned_text.strip_prefix("0x")
		.or_else(|| unsigned_text.strip_prefix("0X"))
	{
		(16, rest)
	} else if let Some(rest) = unsigned_text.strip_prefix("0b")
		.or_else(|| unsigned_text.strip_prefix("0B"))
	{
		(2, rest)
	} else if let Some(rest) = unsigned_text.strip_prefix("0o")
		.or_else(|| unsigned_text.strip_prefix("0O"))
	{
		(8, rest)
	} else {
		(10, unsigned_text)
	};

	let magnitude = u64::from_str_radix(digits, radix)
		.map_err(|_| format!("invalid integer literal: {}", text))?;

	if negative {
		let maximum_magnitude = (i64::MAX as u64) + 1;

		if magnitude > maximum_magnitude {
			return Err(format!("signed integer literal is too small: {}", text));
		}

		if magnitude == maximum_magnitude {
			Ok(IntegerLiteral::Signed(i64::MIN))
		} else {
			Ok(IntegerLiteral::Signed(-(magnitude as i64)))
		}
	} else {
		Ok(IntegerLiteral::Unsigned(magnitude))
	}
}

fn emit_instruction(program: &mut Vec<u8>, instruction: &AssemblyInstruction) {
	match instruction {
		AssemblyInstruction::Push(push_value) => {
			match push_value {
				IntegerLiteral::Unsigned(unsigned_value) => {
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

				IntegerLiteral::Signed(signed_value) => {
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


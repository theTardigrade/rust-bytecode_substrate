use std::collections::HashMap;

use crate::opcodes::{
	OpcodeByte,
	PUSH_UNSIGNED4_START,
	PUSH_UNSIGNED4_END,
	PUSH_SIGNED4_START,
	PUSH_SIGNED4_END,
	JUMP_REL_SIGNED4_START,
	JUMP_REL_SIGNED4_END,
	CALL_REL_SIGNED4_START,
	CALL_REL_SIGNED4_END,
	JUMP_IF_ZERO_REL_SIGNED4_START,
	JUMP_IF_ZERO_REL_SIGNED4_END,
};

enum IntegerLiteral {
	Unsigned(u64),
	Signed(i64),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RelativeWidth {
	Signed4,
	Signed8,
	Signed16,
	Signed32,
	Signed64,
}

#[derive(Clone, Copy)]
enum RelativeInstructionKind {
	Jump,
	JumpIfZero,
	Call,
}

enum AssemblyInstruction {
	Noop,

	Push(IntegerLiteral),
	PushAddress(String),

	Add,
	Sub,
	Mul,
	DivUnsigned,
	DivSigned,
	RemUnsigned,
	RemSigned,

	BitAnd,
	BitOr,
	BitXor,
	BitNot,

	ShiftLeft,
	LogicShiftRight,
	ArithShiftRight,

	CompEqual,
	CompNotEqual,
	CompLessThanUnsigned,
	CompLessThanSigned,
	CompLessThanOrEqualUnsigned,
	CompLessThanOrEqualSigned,
	CompGreaterThanUnsigned,
	CompGreaterThanSigned,
	CompGreaterThanOrEqualUnsigned,
	CompGreaterThanOrEqualSigned,

	Dup,
	Drop,
	Swap,

	Halt,
	Return,
	JumpIndirect,
	CallIndirect,

	MemLoad8,
	MemLoad16,
	MemLoad32,
	MemLoad64,

	MemStore8,
	MemStore16,
	MemStore32,
	MemStore64,

	MemSize,
	MemGrow,
	MemCopy8,
	MemFill8,

	Relative {
		kind: RelativeInstructionKind,
		label_name: String,
		width: RelativeWidth,
	},
}

enum AssemblyItem {
	Instruction(AssemblyInstruction),
	Label(String),
}

pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
	let mut items = parse_source(source)?;

	relax_relative_instructions(&mut items)?;

	let labels = calculate_label_addresses(&items)?;
	let program = emit_program(&items, &labels)?;

	Ok(program)
}

fn emit_program(
	items: &[AssemblyItem],
	labels: &HashMap<String, usize>,
) -> Result<Vec<u8>, String> {
	let mut program = Vec::new();

	for item in items {
		match item {
			AssemblyItem::Label(_) => {}

			AssemblyItem::Instruction(instruction) => {
				emit_instruction(&mut program, instruction, labels)?;
			}
		}
	}

	Ok(program)
}

fn parse_source(source: &str) -> Result<Vec<AssemblyItem>, String> {
	let mut items = Vec::new();

	for line in source.lines() {
		let line = line
			.split_once(';')
			.map_or(line, |(before_comment, _)| before_comment)
			.trim();

		if line.is_empty() {
			continue;
		}

		if line.ends_with(':') {
			let label_name = parse_label(line)?;
			items.push(AssemblyItem::Label(label_name));
		} else {
			let instruction = parse_instruction(line)?;
			items.push(AssemblyItem::Instruction(instruction));
		}
	}

	Ok(items)
}

fn expect_operand_count(
	parts: &[&str],
	expected_operand_count: usize,
	mnemonic: &str,
) -> Result<(), String> {
	let actual_operand_count = parts.len() - 1;

	if actual_operand_count != expected_operand_count {
		return Err(match expected_operand_count {
			0 => format!("{} does not take any operands", mnemonic),
			1 => format!("{} expects exactly one operand", mnemonic),
			count => format!("{} expects exactly {} operands", mnemonic, count),
		});
	}

	Ok(())
}

fn parse_instruction(line: &str) -> Result<AssemblyInstruction, String> {
	let parts: Vec<&str> = line.split_whitespace().collect();

	if parts.is_empty() {
		return Err("expected instruction".to_string());
	}

	let mnemonic = parts[0].to_ascii_uppercase();

	match mnemonic.as_str() {
		"NOOP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Noop)
		}

		"MUL" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Mul)
		}

		"DUP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Dup)
		}

		"DROP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Drop)
		}

		"SWAP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Swap)
		}

		"PUSH" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let integer_literal = parse_integer_literal(parts[1])?;

			Ok(AssemblyInstruction::Push(integer_literal))
		}

		"PADDR" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let label_name = parts[1];

			validate_label_name(label_name)?;

			Ok(AssemblyInstruction::PushAddress(label_name.to_string()))
		}

		"ADD" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Add)
		}

		"SUB" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Sub)
		}

		"DIVU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::DivUnsigned)
		}

		"DIVS" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::DivSigned)
		}

		"REMU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::RemUnsigned)
		}

		"REMS" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::RemSigned)
		}

		"AND" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::BitAnd)
		}

		"OR" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::BitOr)
		}

		"XOR" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::BitXor)
		}

		"NOT" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::BitNot)
		}

		"SL" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::ShiftLeft)
		}

		"LSR" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::LogicShiftRight)
		}

		"ASR" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::ArithShiftRight)
		}

		"CEQ" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompEqual)
		}

		"CNE" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompNotEqual)
		}

		"CLTU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompLessThanUnsigned)
		}

		"CLTS" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompLessThanSigned)
		}

		"CLEU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompLessThanOrEqualUnsigned)
		}

		"CLES" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompLessThanOrEqualSigned)
		}

		"CGTU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompGreaterThanUnsigned)
		}

		"CGTS" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompGreaterThanSigned)
		}

		"CGEU" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompGreaterThanOrEqualUnsigned)
		}

		"CGES" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::CompGreaterThanOrEqualSigned)
		}

		"JMP" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let label_name = parts[1];

			validate_label_name(label_name)?;

			Ok(AssemblyInstruction::Relative {
				kind: RelativeInstructionKind::Jump,
				label_name: label_name.to_string(),
				width: RelativeWidth::Signed4,
			})
		}

		"JZ" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let label_name = parts[1];

			validate_label_name(label_name)?;

			Ok(AssemblyInstruction::Relative {
				kind: RelativeInstructionKind::JumpIfZero,
				label_name: label_name.to_string(),
				width: RelativeWidth::Signed4,
			})
		}

		"CALL" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let label_name = parts[1];

			validate_label_name(label_name)?;

			Ok(AssemblyInstruction::Relative {
				kind: RelativeInstructionKind::Call,
				label_name: label_name.to_string(),
				width: RelativeWidth::Signed4,
			})
		}

		"RET" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Return)
		}

		"JMPIND" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::JumpIndirect)
		}

		"CALLIND" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::CallIndirect)
		}

		"MEMLD8" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemLoad8)
		}

		"MEMLD16" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemLoad16)
		}

		"MEMLD32" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemLoad32)
		}

		"MEMLD64" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemLoad64)
		}

		"MEMST8" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore8)
		}

		"MEMST16" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore16)
		}

		"MEMST32" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore32)
		}

		"MEMST64" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore64)
		}

		"MEMSIZE" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemSize)
		}

		"MEMGROW" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemGrow)
		}

		"MEMCOPY8" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemCopy8)
		}

		"MEMFILL8" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemFill8)
		}

		"HALT" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Halt)
		}

		_ => Err(format!("unknown instruction: {}", parts[0])),
	}
}

fn parse_label(line: &str) -> Result<String, String> {
	let label_name = line
		.strip_suffix(':')
		.expect("parse_label should only be called for label statements");

	validate_label_name(label_name)?;

	Ok(label_name.to_string())
}

fn validate_label_name(label_name: &str) -> Result<(), String> {
	let mut characters = label_name.chars();

	let Some(first_character) = characters.next() else {
		return Err("label name cannot be empty".to_string());
	};

	if !first_character.is_ascii_alphabetic() && first_character != '_' {
		return Err(format!("invalid label name: {}", label_name));
	}

	if !characters.all(|character| character.is_ascii_alphanumeric() || character == '_') {
		return Err(format!("invalid label name: {}", label_name));
	}

	Ok(())
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

fn emit_instruction(
	program: &mut Vec<u8>,
	instruction: &AssemblyInstruction,
	labels: &HashMap<String, usize>,
) -> Result<(), String> {
	match instruction {
		AssemblyInstruction::Push(integer_literal) => {
			match integer_literal {
				IntegerLiteral::Unsigned(unsigned_value) => {
					let unsigned4_max = (PUSH_UNSIGNED4_END - PUSH_UNSIGNED4_START) as u64;

					if *unsigned_value <= unsigned4_max {
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

		AssemblyInstruction::PushAddress(label_name) => {
			let target_address = *labels
				.get(label_name)
				.ok_or_else(|| format!("unknown label: {}", label_name))?;

			let target_address = u64::try_from(target_address)
				.map_err(|_| format!("label address is too large: {}", label_name))?;

			program.push(OpcodeByte::Push64 as u8);
			program.extend_from_slice(&target_address.to_le_bytes());
		}

		AssemblyInstruction::Add => {
			program.push(OpcodeByte::Add as u8);
		}

		AssemblyInstruction::Sub => {
			program.push(OpcodeByte::Sub as u8);
		}

		AssemblyInstruction::DivUnsigned => {
			program.push(OpcodeByte::DivUnsigned as u8);
		}

		AssemblyInstruction::DivSigned => {
			program.push(OpcodeByte::DivSigned as u8);
		}

		AssemblyInstruction::RemUnsigned => {
			program.push(OpcodeByte::RemUnsigned as u8);
		}

		AssemblyInstruction::RemSigned => {
			program.push(OpcodeByte::RemSigned as u8);
		}

		AssemblyInstruction::BitAnd => {
			program.push(OpcodeByte::BitAnd as u8);
		}

		AssemblyInstruction::BitOr => {
			program.push(OpcodeByte::BitOr as u8);
		}

		AssemblyInstruction::BitXor => {
			program.push(OpcodeByte::BitXor as u8);
		}

		AssemblyInstruction::BitNot => {
			program.push(OpcodeByte::BitNot as u8);
		}

		AssemblyInstruction::ShiftLeft => {
			program.push(OpcodeByte::ShiftLeft as u8);
		}

		AssemblyInstruction::LogicShiftRight => {
			program.push(OpcodeByte::LogicShiftRight as u8);
		}

		AssemblyInstruction::ArithShiftRight => {
			program.push(OpcodeByte::ArithShiftRight as u8);
		}

		AssemblyInstruction::CompEqual => {
			program.push(OpcodeByte::CompEqual as u8);
		}

		AssemblyInstruction::CompNotEqual => {
			program.push(OpcodeByte::CompNotEqual as u8);
		}

		AssemblyInstruction::CompLessThanUnsigned => {
			program.push(OpcodeByte::CompLessThanUnsigned as u8);
		}

		AssemblyInstruction::CompLessThanSigned => {
			program.push(OpcodeByte::CompLessThanSigned as u8);
		}

		AssemblyInstruction::CompLessThanOrEqualUnsigned => {
			program.push(OpcodeByte::CompLessThanOrEqualUnsigned as u8);
		}

		AssemblyInstruction::CompLessThanOrEqualSigned => {
			program.push(OpcodeByte::CompLessThanOrEqualSigned as u8);
		}

		AssemblyInstruction::CompGreaterThanUnsigned => {
			program.push(OpcodeByte::CompGreaterThanUnsigned as u8);
		}

		AssemblyInstruction::CompGreaterThanSigned => {
			program.push(OpcodeByte::CompGreaterThanSigned as u8);
		}

		AssemblyInstruction::CompGreaterThanOrEqualUnsigned => {
			program.push(OpcodeByte::CompGreaterThanOrEqualUnsigned as u8);
		}

		AssemblyInstruction::CompGreaterThanOrEqualSigned => {
			program.push(OpcodeByte::CompGreaterThanOrEqualSigned as u8);
		}

		AssemblyInstruction::Return => {
			program.push(OpcodeByte::Return as u8);
		}

		AssemblyInstruction::JumpIndirect => {
			program.push(OpcodeByte::JumpAbsIndirect as u8);
		}

		AssemblyInstruction::CallIndirect => {
			program.push(OpcodeByte::CallAbsIndirect as u8);
		}

		AssemblyInstruction::Noop => {
			program.push(OpcodeByte::Noop as u8);
		}

		AssemblyInstruction::Mul => {
			program.push(OpcodeByte::Mul as u8);
		}

		AssemblyInstruction::Dup => {
			program.push(OpcodeByte::Dup as u8);
		}

		AssemblyInstruction::Drop => {
			program.push(OpcodeByte::Drop as u8);
		}

		AssemblyInstruction::Swap => {
			program.push(OpcodeByte::Swap as u8);
		}

		AssemblyInstruction::MemLoad8 => {
			program.push(OpcodeByte::MemLoad8 as u8);
		}

		AssemblyInstruction::MemLoad16 => {
			program.push(OpcodeByte::MemLoad16 as u8);
		}

		AssemblyInstruction::MemLoad32 => {
			program.push(OpcodeByte::MemLoad32 as u8);
		}

		AssemblyInstruction::MemLoad64 => {
			program.push(OpcodeByte::MemLoad64 as u8);
		}

		AssemblyInstruction::MemStore8 => {
			program.push(OpcodeByte::MemStore8 as u8);
		}

		AssemblyInstruction::MemStore16 => {
			program.push(OpcodeByte::MemStore16 as u8);
		}

		AssemblyInstruction::MemStore32 => {
			program.push(OpcodeByte::MemStore32 as u8);
		}

		AssemblyInstruction::MemStore64 => {
			program.push(OpcodeByte::MemStore64 as u8);
		}

		AssemblyInstruction::MemSize => {
			program.push(OpcodeByte::MemSize as u8);
		}

		AssemblyInstruction::MemGrow => {
			program.push(OpcodeByte::MemGrow as u8);
		}

		AssemblyInstruction::MemCopy8 => {
			program.push(OpcodeByte::MemCopy8 as u8);
		}

		AssemblyInstruction::MemFill8 => {
			program.push(OpcodeByte::MemFill8 as u8);
		}

		AssemblyInstruction::Relative {
			kind,
			label_name,
			width,
		} => {
			let instruction_address = program.len();

			let target_address = *labels
				.get(label_name)
				.ok_or_else(|| format!("unknown label: {}", label_name))?;

			let instruction_size = relative_width_instruction_size(*width);
			let next_instruction_address = instruction_address + instruction_size;

			let offset = target_address as i128 - next_instruction_address as i128;

			match *width {
				RelativeWidth::Signed4 => {
					let offset = i8::try_from(offset)
						.expect("relaxed signed4 relative offset should fit in i8");

					let (opcode_start, opcode_end) = match *kind {
						RelativeInstructionKind::Jump => {
							(JUMP_REL_SIGNED4_START, JUMP_REL_SIGNED4_END)
						}

						RelativeInstructionKind::JumpIfZero => {
							(JUMP_IF_ZERO_REL_SIGNED4_START, JUMP_IF_ZERO_REL_SIGNED4_END)
						}

						RelativeInstructionKind::Call => {
							(CALL_REL_SIGNED4_START, CALL_REL_SIGNED4_END)
						}
					};

					let signed4_mask = opcode_end - opcode_start;
					let encoded_offset = (offset as u8) & signed4_mask;

					program.push(opcode_start + encoded_offset);
				}

				RelativeWidth::Signed8 => {
					let offset = i8::try_from(offset)
						.expect("relaxed signed8 relative offset should fit in i8");

					let opcode = match *kind {
						RelativeInstructionKind::Jump => OpcodeByte::JumpRelSigned8,
						RelativeInstructionKind::JumpIfZero => OpcodeByte::JumpIfZeroRelSigned8,
						RelativeInstructionKind::Call => OpcodeByte::CallRelSigned8,
					};

					program.push(opcode as u8);
					program.push(offset as u8);
				}

				RelativeWidth::Signed16 => {
					let offset = i16::try_from(offset)
						.expect("relaxed signed16 relative offset should fit in i16");

					let opcode = match *kind {
						RelativeInstructionKind::Jump => OpcodeByte::JumpRelSigned16,
						RelativeInstructionKind::JumpIfZero => OpcodeByte::JumpIfZeroRelSigned16,
						RelativeInstructionKind::Call => OpcodeByte::CallRelSigned16,
					};

					program.push(opcode as u8);
					program.extend_from_slice(&offset.to_le_bytes());
				}

				RelativeWidth::Signed32 => {
					let offset = i32::try_from(offset)
						.expect("relaxed signed32 relative offset should fit in i32");

					let opcode = match *kind {
						RelativeInstructionKind::Jump => OpcodeByte::JumpRelSigned32,
						RelativeInstructionKind::JumpIfZero => OpcodeByte::JumpIfZeroRelSigned32,
						RelativeInstructionKind::Call => OpcodeByte::CallRelSigned32,
					};

					program.push(opcode as u8);
					program.extend_from_slice(&offset.to_le_bytes());
				}

				RelativeWidth::Signed64 => {
					let offset = i64::try_from(offset)
						.expect("relaxed signed64 relative offset should fit in i64");

					let opcode = match *kind {
						RelativeInstructionKind::Jump => OpcodeByte::JumpRelSigned64,
						RelativeInstructionKind::JumpIfZero => OpcodeByte::JumpIfZeroRelSigned64,
						RelativeInstructionKind::Call => OpcodeByte::CallRelSigned64,
					};

					program.push(opcode as u8);
					program.extend_from_slice(&offset.to_le_bytes());
				}
			}
		}

		AssemblyInstruction::Halt => {
			program.push(OpcodeByte::Halt as u8);
		}
	}

	Ok(())
}

fn instruction_size(instruction: &AssemblyInstruction) -> usize {
	match instruction {
		AssemblyInstruction::Push(integer_literal) => {
			integer_literal_push_size(integer_literal)
		}

		AssemblyInstruction::Noop
			| AssemblyInstruction::Add
			| AssemblyInstruction::Sub
			| AssemblyInstruction::Mul
			| AssemblyInstruction::DivUnsigned
			| AssemblyInstruction::DivSigned
			| AssemblyInstruction::RemUnsigned
			| AssemblyInstruction::RemSigned
			| AssemblyInstruction::BitAnd
			| AssemblyInstruction::BitOr
			| AssemblyInstruction::BitXor
			| AssemblyInstruction::BitNot
			| AssemblyInstruction::ShiftLeft
			| AssemblyInstruction::LogicShiftRight
			| AssemblyInstruction::ArithShiftRight
			| AssemblyInstruction::Dup
			| AssemblyInstruction::Drop
			| AssemblyInstruction::Swap
			| AssemblyInstruction::Halt
			| AssemblyInstruction::CompEqual
			| AssemblyInstruction::CompNotEqual
			| AssemblyInstruction::CompLessThanUnsigned
			| AssemblyInstruction::CompLessThanSigned
			| AssemblyInstruction::CompLessThanOrEqualUnsigned
			| AssemblyInstruction::CompLessThanOrEqualSigned
			| AssemblyInstruction::CompGreaterThanUnsigned
			| AssemblyInstruction::CompGreaterThanSigned
			| AssemblyInstruction::CompGreaterThanOrEqualUnsigned
			| AssemblyInstruction::CompGreaterThanOrEqualSigned
			| AssemblyInstruction::MemLoad8
			| AssemblyInstruction::MemLoad16
			| AssemblyInstruction::MemLoad32
			| AssemblyInstruction::MemLoad64
			| AssemblyInstruction::MemStore8
			| AssemblyInstruction::MemStore16
			| AssemblyInstruction::MemStore32
			| AssemblyInstruction::MemStore64
			| AssemblyInstruction::MemSize
			| AssemblyInstruction::MemGrow
			| AssemblyInstruction::MemCopy8
			| AssemblyInstruction::MemFill8
			| AssemblyInstruction::Return
			| AssemblyInstruction::JumpIndirect
			| AssemblyInstruction::CallIndirect => 1,
		
		AssemblyInstruction::PushAddress(_) => 9,

		AssemblyInstruction::Relative { width, .. } => {
			relative_width_instruction_size(*width)
		}
	}
}

fn relative_width_instruction_size(width: RelativeWidth) -> usize {
	match width {
		RelativeWidth::Signed4 => 1,
		RelativeWidth::Signed8 => 2,
		RelativeWidth::Signed16 => 3,
		RelativeWidth::Signed32 => 5,
		RelativeWidth::Signed64 => 9,
	}
}

fn relative_width_offset_fits(width: RelativeWidth, offset: i128) -> bool {
	match width {
		RelativeWidth::Signed4 => offset >= -8 && offset <= 7,
		RelativeWidth::Signed8 => offset >= i8::MIN as i128 && offset <= i8::MAX as i128,
		RelativeWidth::Signed16 => offset >= i16::MIN as i128 && offset <= i16::MAX as i128,
		RelativeWidth::Signed32 => offset >= i32::MIN as i128 && offset <= i32::MAX as i128,
		RelativeWidth::Signed64 => offset >= i64::MIN as i128 && offset <= i64::MAX as i128,
	}
}

fn next_relative_width(width: RelativeWidth) -> Option<RelativeWidth> {
	match width {
		RelativeWidth::Signed4 => Some(RelativeWidth::Signed8),
		RelativeWidth::Signed8 => Some(RelativeWidth::Signed16),
		RelativeWidth::Signed16 => Some(RelativeWidth::Signed32),
		RelativeWidth::Signed32 => Some(RelativeWidth::Signed64),
		RelativeWidth::Signed64 => None,
	}
}

fn relax_relative_instructions(items: &mut [AssemblyItem]) -> Result<(), String> {
	loop {
		let labels = calculate_label_addresses(items)?;
		let mut address = 0usize;
		let mut changed = false;

		for item in items.iter_mut() {
			match item {
				AssemblyItem::Label(_) => {}

				AssemblyItem::Instruction(instruction) => {
					let instruction_size = instruction_size(&*instruction);

					if let AssemblyInstruction::Relative {
						label_name,
						width,
						..
					} = instruction
					{
						let target_address = *labels
							.get(label_name)
							.ok_or_else(|| format!("unknown label: {}", label_name))?;

						let next_instruction_address = address + instruction_size;

						let offset = target_address as i128
							- next_instruction_address as i128;

						if !relative_width_offset_fits(*width, offset) {
							let wider_width = next_relative_width(*width)
								.ok_or_else(|| {
									format!(
										"jump target is too far away: {}",
										label_name
									)
								})?;

							*width = wider_width;
							changed = true;
						}
					}

					address += instruction_size;
				}
			}
		}

		if !changed {
			break;
		}
	}

	Ok(())
}

fn integer_literal_push_size(integer_literal: &IntegerLiteral) -> usize {
	match integer_literal {
		IntegerLiteral::Unsigned(unsigned_value) => {
			let unsigned4_max = (PUSH_UNSIGNED4_END - PUSH_UNSIGNED4_START) as u64;

			if *unsigned_value <= unsigned4_max {
				1
			} else if *unsigned_value <= u8::MAX as u64 {
				2
			} else if *unsigned_value <= u16::MAX as u64 {
				3
			} else if *unsigned_value <= u32::MAX as u64 {
				5
			} else {
				9
			}
		}

		IntegerLiteral::Signed(signed_value) => {
			let signed4_count = (PUSH_SIGNED4_END - PUSH_SIGNED4_START + 1) as i64;
			let signed4_half = signed4_count / 2;

			if *signed_value >= -signed4_half && *signed_value < signed4_half {
				1
			} else if *signed_value >= i8::MIN as i64 && *signed_value <= i8::MAX as i64 {
				2
			} else if *signed_value >= i16::MIN as i64 && *signed_value <= i16::MAX as i64 {
				3
			} else if *signed_value >= i32::MIN as i64 && *signed_value <= i32::MAX as i64 {
				5
			} else {
				9
			}
		}
	}
}

fn calculate_label_addresses(
	items: &[AssemblyItem],
) -> Result<HashMap<String, usize>, String> {
	let mut labels = HashMap::new();
	let mut address = 0usize;

	for item in items {
		match item {
			AssemblyItem::Label(label_name) => {
				if labels.insert(label_name.clone(), address).is_some() {
					return Err(format!("duplicate label: {}", label_name));
				}
			}

			AssemblyItem::Instruction(instruction) => {
				address += instruction_size(instruction);
			}
		}
	}

	Ok(labels)
}
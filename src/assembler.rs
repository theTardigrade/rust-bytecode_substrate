use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, VecDeque};

use crate::data_structures::fenwick::FenwickTree;
use crate::data_structures::segment_tree::SegmentTree;

use crate::host_calls::HostCallId;
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
	PICK_UNSIGNED4_START,
	REGISTER_COUNT,
	REGISTER_GET_START,
	REGISTER_SET_START,
	HOST_CALL_DIRECT_START,
	HOST_CALL_DIRECT_END,
};

enum IntegerLiteral {
	Unsigned(u64),
	Signed(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct LabelId(usize);

#[derive(Clone, Copy)]
struct DatumInfo {
	label: LabelId,
	length: u64,
}

struct Label {
	name: String,
	instruction_boundary: Option<usize>,
}

struct LabelTable {
	labels: Vec<Label>,
	ids_by_name: HashMap<String, LabelId>,
}

impl LabelTable {
	fn new() -> Self {
		Self {
			labels: Vec::new(),
			ids_by_name: HashMap::new(),
		}
	}

	fn get_or_create(&mut self, name: &str) -> LabelId {
		let key = name.to_ascii_lowercase();

		if let Some(label_id) = self.ids_by_name.get(&key) {
			return *label_id;
		}

		let label_id = LabelId(self.labels.len());

		self.labels.push(Label {
			name: name.to_string(),
			instruction_boundary: None,
		});

		self.ids_by_name.insert(key, label_id);

		label_id
	}

	fn define(
		&mut self,
		name: &str,
		instruction_boundary: usize,
	) -> Result<LabelId, String> {
		let label_id = self.get_or_create(name);
		let label = &mut self.labels[label_id.0];

		if label.instruction_boundary.is_some() {
			return Err(format!("duplicate label: {}", name));
		}

		label.instruction_boundary = Some(instruction_boundary);

		Ok(label_id)
	}

	fn instruction_boundary(&self, label_id: LabelId) -> Result<usize, String> {
		let label = &self.labels[label_id.0];

		label
			.instruction_boundary
			.ok_or_else(|| format!("unknown label: {}", label.name))
	}

	fn name(&self, label_id: LabelId) -> &str {
		&self.labels[label_id.0].name
	}
}

enum AssemblyInstruction {
	Noop,

	Push(IntegerLiteral),
	PushAddress(LabelId),

	DataHeader(usize),
	RawData(Vec<u8>),

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

	Pick(u64),
	Pop,
	Swap,

	Halt,
	Return,

	JumpIndirect,
	JumpIfZeroIndirect,
	CallIndirect,

	RegisterGet(u8),
	RegisterSet(u8),
	RegGetIndirect,
	RegSetIndirect,

	MemLoad8,
	MemLoad16,
	MemLoad32,
	MemLoad64,

	MemStore8,
	MemStore16,
	MemStore32,
	MemStore64,
	MemStore128,
	MemStore256,
	MemStore512,
	MemStore1024,

	MemSize,
	MemGrow,
	MemCopy,
	MemFill,
	MemBytecodeCopy,

	HostCall(IntegerLiteral),
	HostCallIndirect,

	Relative {
		kind: RelativeInstructionKind,
		label: LabelId,
		width: RelativeWidth,
	},
}

struct AssemblyProgram {
	instructions: Vec<AssemblyInstruction>,
	relative_instruction_indices: Vec<usize>,
	labels: LabelTable,
}

pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
	let mut assembly_program = parse_source(source)?;

	let instruction_sizes: Vec<usize> = assembly_program
		.instructions
		.iter()
		.map(instruction_size)
		.collect();

	let mut layout = FenwickTree::from_slice(&instruction_sizes);

	relax_relative_instructions(&mut assembly_program, &mut layout)?;

	emit_program(&assembly_program, &layout)
}

pub fn assemble_file(path: &Path) -> Result<Vec<u8>, String> {
	let mut expanded_source = String::new();
	let mut active_files = Vec::new();

	expand_includes(
		path,
		&mut active_files,
		&mut expanded_source,
	)?;

	assemble(&expanded_source)
}

fn expand_includes(
	path: &Path,
	active_files: &mut Vec<PathBuf>,
	output: &mut String,
) -> Result<(), String> {
	const MAX_INCLUDE_DEPTH: usize = 64;

	// Resolve the actual filesystem path. This also lets us
	// identify the same file when referenced through different paths.
	let canonical_path = fs::canonicalize(path)
		.map_err(|error| {
			format!(
				"failed to resolve assembly file {}: {}",
				path.display(),
				error,
			)
		})?;

	// Reject direct and indirect include cycles.
	if active_files.contains(&canonical_path) {
		return Err(format!(
			"circular INC detected: {}",
			canonical_path.display(),
		));
	}

	// Prevent excessively deep chains of nested includes.
	if active_files.len() >= MAX_INCLUDE_DEPTH {
		return Err(format!(
			"INC nesting exceeds {} files",
			MAX_INCLUDE_DEPTH,
		));
	}

	let source = fs::read_to_string(&canonical_path)
		.map_err(|error| {
			format!(
				"failed to read assembly file {}: {}",
				canonical_path.display(),
				error,
			)
		})?;

	active_files.push(canonical_path.clone());

	for (line_index, line) in source.lines().enumerate() {
		let stripped = strip_assembly_comment(line).trim();

		let mut parts = stripped.splitn(2, char::is_whitespace);
		let mnemonic = parts.next().unwrap_or("");

		if mnemonic.eq_ignore_ascii_case("INC") {
			let operand = parts.next().unwrap_or("").trim();

			let path_bytes = parse_string_literal(operand)
				.map_err(|error| {
					format!(
						"{}:{}: INC: {}",
						canonical_path.display(),
						line_index + 1,
						error,
					)
				})?;

			let include_name = String::from_utf8(path_bytes)
				.map_err(|_| {
					format!(
						"{}:{}: INC path must be valid UTF-8",
						canonical_path.display(),
						line_index + 1,
					)
				})?;

			if include_name.is_empty() {
				return Err(format!(
					"{}:{}: INC path cannot be empty",
					canonical_path.display(),
					line_index + 1,
				));
			}

			// Relative paths are resolved against the directory
			// containing the file with this INC directive.
			let include_path = canonical_path
				.parent()
				.unwrap_or(Path::new("."))
				.join(include_name);

			expand_includes(
				&include_path,
				active_files,
				output,
			)
			.map_err(|error| {
				format!(
					"{}:{}: {}",
					canonical_path.display(),
					line_index + 1,
					error,
				)
			})?;
		} else {
			// Preserve ordinary assembly lines verbatim.
			// The newline keeps adjacent files from merging lines.
			output.push_str(line);
			output.push('\n');
		}
	}

	active_files.pop();

	Ok(())
}

fn emit_program(
	assembly_program: &AssemblyProgram,
	layout: &FenwickTree,
) -> Result<Vec<u8>, String> {
	let program_size =
		layout.prefix_sum(assembly_program.instructions.len());

	let mut program = Vec::with_capacity(program_size);

	for (instruction_index, instruction) in
		assembly_program.instructions.iter().enumerate()
	{
		emit_instruction(
			&mut program,
			instruction,
			instruction_index,
			&assembly_program.labels,
			layout,
		)?;
	}

	Ok(program)
}

fn parse_source(source: &str) -> Result<AssemblyProgram, String> {
	let mut instructions = Vec::new();
	let mut relative_instruction_indices = Vec::new();
	let mut labels = LabelTable::new();
	let mut data_symbols: HashMap<String, DatumInfo> = HashMap::new();

	for line in source.lines() {
		let line = strip_assembly_comment(line).trim();

		if line.is_empty() {
			continue;
		}

		if line.ends_with(':') {
			let label_name = parse_label(line)?;
			labels.define(&label_name, instructions.len())?;
			continue;
		}

		let mut parts = line.splitn(2, char::is_whitespace);
		let mnemonic = parts.next().unwrap();
		let operands = parts.next().unwrap_or("").trim();

		if mnemonic.eq_ignore_ascii_case("DATA") {
			let mut data_parts =
				operands.splitn(2, char::is_whitespace);

			let name = data_parts.next().unwrap_or("");
			let literal = data_parts.next().unwrap_or("").trim();

			validate_label_name(name)?;

			let bytes = parse_string_literal(literal)
				.map_err(|error| format!("DATA {}: {}", name, error))?;

			let length = u64::try_from(bytes.len())
				.map_err(|_| "DATA payload is too large".to_string())?;

			// The symbol points past the header, to the payload.
			let label = labels.define(name, instructions.len() + 1)?;

			data_symbols.insert(
				name.to_ascii_lowercase(),
				DatumInfo { label, length },
			);

			instructions.push(
				AssemblyInstruction::DataHeader(bytes.len()),
			);

			instructions.push(
				AssemblyInstruction::RawData(bytes),
			);

			continue;
		}

		if mnemonic.eq_ignore_ascii_case("MEMDTCP") {
			if operands.starts_with('"') {
				let bytes = parse_string_literal(operands)
					.map_err(|error| format!("MEMDTCP: {}", error))?;

				instructions.extend(
					expand_inline_data_copy(&bytes)?,
				);
			} else {
				validate_label_name(operands)?;

				let key = operands.to_ascii_lowercase();

				let datum = data_symbols.get(&key)
					.ok_or_else(|| {
						format!(
							"unknown or not yet declared DATA: {}",
							operands,
						)
					})?;

				instructions.push(
					AssemblyInstruction::Pick(0),
				);

				instructions.push(
					AssemblyInstruction::PushAddress(datum.label),
				);

				instructions.push(
					AssemblyInstruction::Push(
						IntegerLiteral::Unsigned(datum.length),
					),
				);

				instructions.push(
					AssemblyInstruction::MemBytecodeCopy,
				);

				instructions.push(
					AssemblyInstruction::Push(
						IntegerLiteral::Unsigned(datum.length),
					),
				);
			}

			continue;
		}

		let instruction = parse_instruction(line, &mut labels)?;

		if matches!(instruction, AssemblyInstruction::Relative { .. }) {
			relative_instruction_indices.push(instructions.len());
		}

		instructions.push(instruction);
	}

	Ok(AssemblyProgram {
		instructions,
		relative_instruction_indices,
		labels,
	})
}

fn strip_assembly_comment(line: &str) -> &str {
	let mut inside_string = false;
	let mut escaped = false;

	for (index, character) in line.char_indices() {
		if inside_string {
			if escaped {
				escaped = false;
			} else if character == '\\' {
				escaped = true;
			} else if character == '"' {
				inside_string = false;
			}
		} else if character == '"' {
			inside_string = true;
		} else if character == ';' {
			return &line[..index];
		}
	}

	line
}

fn parse_string_literal(operand: &str) -> Result<Vec<u8>, String> {
	let Some(contents) = operand.strip_prefix('"') else {
		return Err("expected a double-quoted string literal".to_string());
	};

	let mut bytes = Vec::new();
	let mut characters = contents.chars();

	while let Some(character) = characters.next() {
		if character == '"' {
			if characters.as_str().trim().is_empty() {
				return Ok(bytes);
			}

			return Err("unexpected text after string literal".to_string());
		}

		if character == '\\' {
			let escaped = characters.next()
				.ok_or_else(|| "unfinished string escape sequence".to_string())?;

			match escaped {
				'n' => bytes.push(b'\n'),
				'r' => bytes.push(b'\r'),
				't' => bytes.push(b'\t'),
				'0' => bytes.push(0),
				'\\' => bytes.push(b'\\'),
				'"' => bytes.push(b'"'),
				'x' => {
					let high = characters.next()
						.and_then(|digit| digit.to_digit(16))
						.ok_or_else(|| "\\x escape expects two hexadecimal digits".to_string())?;

					let low = characters.next()
						.and_then(|digit| digit.to_digit(16))
						.ok_or_else(|| "\\x escape expects two hexadecimal digits".to_string())?;

					bytes.push(((high << 4) | low) as u8);
				}
				_ => return Err(format!(
					"unsupported string escape sequence: \\{}",
					escaped
				)),
			}
		} else {
			let mut encoded = [0; 4];
			bytes.extend_from_slice(
				character.encode_utf8(&mut encoded).as_bytes()
			);
		}
	}

	Err("unterminated string literal".to_string())
}

fn expand_inline_data_copy(bytes: &[u8]) -> Result<Vec<AssemblyInstruction>, String> {
	let length = u64::try_from(bytes.len())
		.map_err(|_| "string is too large".to_string())?;

	let mut instructions = Vec::new();
	let mut offset = 0;

	while offset < bytes.len() {
		let remaining = bytes.len() - offset;

		let width = [128, 64, 32, 16, 8, 4, 2, 1]
			.iter()
			.copied()
			.find(|&width| width <= remaining)
			.unwrap();

		// Keep the original destination below the temporary address.
		instructions.push(AssemblyInstruction::Pick(0));

		if offset != 0 {
			instructions.push(AssemblyInstruction::Push(
				IntegerLiteral::Unsigned(offset as u64),
			));
			instructions.push(AssemblyInstruction::Add);
		}

		for word_bytes in bytes[offset..offset + width].chunks(8) {
			let mut packed = [0u8; 8];
			packed[..word_bytes.len()].copy_from_slice(word_bytes);

			instructions.push(AssemblyInstruction::Push(
				IntegerLiteral::Unsigned(u64::from_le_bytes(packed)),
			));
		}

		instructions.push(match width {
			1 => AssemblyInstruction::MemStore8,
			2 => AssemblyInstruction::MemStore16,
			4 => AssemblyInstruction::MemStore32,
			8 => AssemblyInstruction::MemStore64,
			16 => AssemblyInstruction::MemStore128,
			32 => AssemblyInstruction::MemStore256,
			64 => AssemblyInstruction::MemStore512,
			128 => AssemblyInstruction::MemStore1024,
			_ => unreachable!(),
		});

		offset += width;
	}

	instructions.push(AssemblyInstruction::Push(
		IntegerLiteral::Unsigned(length),
	));

	Ok(instructions)
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

fn parse_instruction(
	line: &str,
	labels: &mut LabelTable,
) -> Result<AssemblyInstruction, String> {
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

			Ok(AssemblyInstruction::Pick(0))
		}

		"PICK" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let depth = match parse_integer_literal(parts[1])? {
				IntegerLiteral::Unsigned(depth) => depth,
				IntegerLiteral::Signed(_) => return Err("PICK depth cannot be negative".to_string()),
			};

			Ok(AssemblyInstruction::Pick(depth))
		}

		"POP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Pop)
		}

		"SWP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Swap)
		}

		"PUSH" => {
			expect_operand_count(&parts, 1, mnemonic.as_str())?;

			let operand = parts[1];

			if let Some(integer_literal) = parse_named_constant(operand) {
				Ok(AssemblyInstruction::Push(integer_literal))
			} else if validate_label_name(operand).is_ok() {
				let label = labels.get_or_create(operand);

				Ok(AssemblyInstruction::PushAddress(label))
			} else {
				let integer_literal = parse_integer_literal(operand)?;

				Ok(AssemblyInstruction::Push(integer_literal))
			}
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
			if parts.len() == 1 {
				Ok(AssemblyInstruction::JumpIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				let label_name = parts[1];
				validate_label_name(label_name)?;

				let label = labels.get_or_create(label_name);

				Ok(AssemblyInstruction::Relative {
					kind: RelativeInstructionKind::Jump,
					label,
					width: RelativeWidth::Signed4,
				})
			}
		}

		"JZ" => {
			if parts.len() == 1 {
				Ok(AssemblyInstruction::JumpIfZeroIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				let label_name = parts[1];
				validate_label_name(label_name)?;

				let label = labels.get_or_create(label_name);

				Ok(AssemblyInstruction::Relative {
					kind: RelativeInstructionKind::JumpIfZero,
					label,
					width: RelativeWidth::Signed4,
				})
			}
		}

		"CALL" => {
			if parts.len() == 1 {
				Ok(AssemblyInstruction::CallIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				let label_name = parts[1];
				validate_label_name(label_name)?;

				let label = labels.get_or_create(label_name);

				Ok(AssemblyInstruction::Relative {
					kind: RelativeInstructionKind::Call,
					label,
					width: RelativeWidth::Signed4,
				})
			}
		}

		"RT" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;

			Ok(AssemblyInstruction::Return)
		}

		"RGET" => {
			if parts.len() == 1 {
				Ok(AssemblyInstruction::RegGetIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				Ok(AssemblyInstruction::RegisterGet(
					parse_register_index(parts[1])?,
				))
			}
		}

		"RSET" => {
			if parts.len() == 1 {
				Ok(AssemblyInstruction::RegSetIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				Ok(AssemblyInstruction::RegisterSet(
					parse_register_index(parts[1])?,
				))
			}
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

		"MEMST128" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore128)
		}

		"MEMST256" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore256)
		}

		"MEMST512" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore512)
		}

		"MEMST1024" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemStore1024)
		}

		"MEMSZE" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemSize)
		}

		"MEMGRW" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemGrow)
		}

		"MEMCP" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemCopy)
		}

		"MEMFL" => {
			expect_operand_count(&parts, 0, mnemonic.as_str())?;
			Ok(AssemblyInstruction::MemFill)
		}

		"HOST" => {
			if parts.len() == 1 {
				Ok(AssemblyInstruction::HostCallIndirect)
			} else {
				expect_operand_count(&parts, 1, mnemonic.as_str())?;

				let host_call_id = parse_host_call_id(parts[1])?;

				Ok(AssemblyInstruction::HostCall(host_call_id))
			}
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

	if HostCallId::from_assembly_name(label_name).is_some() {
		return Err(format!("reserved label name: {}", label_name));
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

fn parse_named_constant(name: &str) -> Option<IntegerLiteral> {
	HostCallId::from_assembly_name(name)
		.map(|id| IntegerLiteral::Unsigned(id as u64))
}

fn parse_host_call_id(text: &str) -> Result<IntegerLiteral, String> {
	if let Some(integer_literal) = parse_named_constant(text) {
		Ok(integer_literal)
	} else {
		parse_integer_literal(text)
	}
}

fn direct_host_call_opcode(host_call_id: &IntegerLiteral) -> Option<u8> {
	let id = match host_call_id {
		IntegerLiteral::Unsigned(id) => *id,
		IntegerLiteral::Signed(id) if *id >= 0 => *id as u64,
		IntegerLiteral::Signed(_) => return None,
	};

	let maximum_direct_id = (HOST_CALL_DIRECT_END - HOST_CALL_DIRECT_START) as u64;

	(id <= maximum_direct_id).then(|| HOST_CALL_DIRECT_START + id as u8)
}

fn data_header_size(length: usize) -> usize {
	if (1..=12).contains(&length) {
		1
	} else if length <= u8::MAX as usize {
		2
	} else if length <= u16::MAX as usize {
		3
	} else if length <= u32::MAX as usize {
		5
	} else {
		9
	}
}

fn emit_data_header(program: &mut Vec<u8>, length: usize) {
	if (1..=12).contains(&length) {
		program.push(OpcodeByte::DataLength1 as u8 + (length as u8 - 1));
	} else if length <= u8::MAX as usize {
		program.push(OpcodeByte::DataUnsigned8 as u8);
		program.push(length as u8);
	} else if length <= u16::MAX as usize {
		program.push(OpcodeByte::DataUnsigned16 as u8);
		program.extend_from_slice(&(length as u16).to_le_bytes());
	} else if length <= u32::MAX as usize {
		program.push(OpcodeByte::DataUnsigned32 as u8);
		program.extend_from_slice(&(length as u32).to_le_bytes());
	} else {
		program.push(OpcodeByte::DataUnsigned64 as u8);
		program.extend_from_slice(&(length as u64).to_le_bytes());
	}
}

fn emit_instruction(
	program: &mut Vec<u8>,
	instruction: &AssemblyInstruction,
	instruction_index: usize,
	labels: &LabelTable,
	layout: &FenwickTree,
) -> Result<(), String> {
	match instruction {
		AssemblyInstruction::Push(integer_literal) => {
			emit_integer_literal_push(
				program,
				integer_literal,
			);
		}

		AssemblyInstruction::PushAddress(label) => {
			let target_boundary = labels.instruction_boundary(*label)?;
			let target_address = layout.prefix_sum(target_boundary);

			let target_address = u64::try_from(target_address)
				.map_err(|_| {
					format!(
						"label address is too large: {}",
						labels.name(*label),
					)
				})?;

			program.push(OpcodeByte::Push64 as u8);
			program.extend_from_slice(&target_address.to_le_bytes());
		}

		AssemblyInstruction::Pick(depth) => {
			if *depth <= 15 {
				program.push(PICK_UNSIGNED4_START + *depth as u8);
			} else if *depth <= u8::MAX as u64 {
				program.push(OpcodeByte::PickUnsigned8 as u8);
				program.push(*depth as u8);
			} else if *depth <= u16::MAX as u64 {
				program.push(OpcodeByte::PickUnsigned16 as u8);
				program.extend_from_slice(&(*depth as u16).to_le_bytes());
			} else if *depth <= u32::MAX as u64 {
				program.push(OpcodeByte::PickUnsigned32 as u8);
				program.extend_from_slice(&(*depth as u32).to_le_bytes());
			} else {
				program.push(OpcodeByte::PickUnsigned64 as u8);
				program.extend_from_slice(&depth.to_le_bytes());
			}
		}

		AssemblyInstruction::DataHeader(length) => {
			emit_data_header(program, *length);
		}

		AssemblyInstruction::RawData(bytes) => {
			program.extend_from_slice(bytes);
		}

		AssemblyInstruction::RegGetIndirect => {
			program.push(OpcodeByte::RegGetIndirect as u8);
		}

		AssemblyInstruction::RegSetIndirect => {
			program.push(OpcodeByte::RegSetIndirect as u8);
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

		AssemblyInstruction::JumpIfZeroIndirect => {
			program.push(OpcodeByte::JumpIfZeroAbsIndirect as u8);
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

		AssemblyInstruction::Pop => {
			program.push(OpcodeByte::Pop as u8);
		}

		AssemblyInstruction::Swap => {
			program.push(OpcodeByte::Swap as u8);
		}

		AssemblyInstruction::RegisterGet(register_index) => {
			program.push(REGISTER_GET_START + register_index);
		}

		AssemblyInstruction::RegisterSet(register_index) => {
			program.push(REGISTER_SET_START + register_index);
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

		AssemblyInstruction::MemStore128 => {
			program.push(OpcodeByte::MemStore128 as u8);
		}

		AssemblyInstruction::MemStore256 => {
			program.push(OpcodeByte::MemStore256 as u8);
		}

		AssemblyInstruction::MemStore512 => {
			program.push(OpcodeByte::MemStore512 as u8);
		}

		AssemblyInstruction::MemStore1024 => {
			program.push(OpcodeByte::MemStore1024 as u8);
		}

		AssemblyInstruction::MemSize => {
			program.push(OpcodeByte::MemSize as u8);
		}

		AssemblyInstruction::MemGrow => {
			program.push(OpcodeByte::MemGrow as u8);
		}

		AssemblyInstruction::MemCopy => {
			program.push(OpcodeByte::MemCopy as u8);
		}

		AssemblyInstruction::MemBytecodeCopy => {
			program.push(OpcodeByte::MemBytecodeCopy as u8);
		}

		AssemblyInstruction::MemFill => {
			program.push(OpcodeByte::MemFill as u8);
		}

		AssemblyInstruction::HostCall(host_call_id) => {
			if let Some(opcode) = direct_host_call_opcode(host_call_id) {
				program.push(opcode);
			} else {
				emit_integer_literal_push(program, host_call_id);
				program.push(OpcodeByte::HostCallIndirect as u8);
			}
		}

		AssemblyInstruction::HostCallIndirect => {
			program.push(OpcodeByte::HostCallIndirect as u8);
		}

		AssemblyInstruction::Relative {
			kind,
			label,
			width,
		} => {
			let instruction_address =
				layout.prefix_sum(instruction_index);

			let target_boundary =
				labels.instruction_boundary(*label)?;

			let target_address =
				layout.prefix_sum(target_boundary);

			let instruction_size =
				relative_width_instruction_size(*width);

			let next_instruction_address =
				instruction_address + instruction_size;

			let offset =
				target_address as i128 - next_instruction_address as i128;

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

fn emit_integer_literal_push(
	program: &mut Vec<u8>,
	integer_literal: &IntegerLiteral,
) {
	match integer_literal {
		IntegerLiteral::Unsigned(unsigned_value) => {
			let unsigned4_max =
				(PUSH_UNSIGNED4_END - PUSH_UNSIGNED4_START) as u64;

			if *unsigned_value <= unsigned4_max {
				program.push(
					PUSH_UNSIGNED4_START + *unsigned_value as u8
				);
			} else if *unsigned_value <= u8::MAX as u64 {
				program.push(OpcodeByte::PushUnsigned8 as u8);
				program.push(*unsigned_value as u8);
			} else if *unsigned_value <= u16::MAX as u64 {
				program.push(OpcodeByte::PushUnsigned16 as u8);
				program.extend_from_slice(
					&(*unsigned_value as u16).to_le_bytes()
				);
			} else if *unsigned_value <= u32::MAX as u64 {
				program.push(OpcodeByte::PushUnsigned32 as u8);
				program.extend_from_slice(
					&(*unsigned_value as u32).to_le_bytes()
				);
			} else {
				program.push(OpcodeByte::Push64 as u8);
				program.extend_from_slice(
					&unsigned_value.to_le_bytes()
				);
			}
		}

		IntegerLiteral::Signed(signed_value) => {
			let signed4_mask =
				PUSH_SIGNED4_END - PUSH_SIGNED4_START;

			let signed4_count =
				(signed4_mask + 1) as i64;

			let signed4_half =
				signed4_count / 2;

			if *signed_value >= -signed4_half
				&& *signed_value < signed4_half
			{
				let encoded_value =
					(*signed_value as i8 as u8) & signed4_mask;

				program.push(
					PUSH_SIGNED4_START + encoded_value
				);
			} else if *signed_value >= i8::MIN as i64
				&& *signed_value <= i8::MAX as i64
			{
				program.push(OpcodeByte::PushSigned8 as u8);
				program.push(*signed_value as i8 as u8);
			} else if *signed_value >= i16::MIN as i64
				&& *signed_value <= i16::MAX as i64
			{
				program.push(OpcodeByte::PushSigned16 as u8);
				program.extend_from_slice(
					&(*signed_value as i16).to_le_bytes()
				);
			} else if *signed_value >= i32::MIN as i64
				&& *signed_value <= i32::MAX as i64
			{
				program.push(OpcodeByte::PushSigned32 as u8);
				program.extend_from_slice(
					&(*signed_value as i32).to_le_bytes()
				);
			} else {
				program.push(OpcodeByte::Push64 as u8);
				program.extend_from_slice(
					&signed_value.to_le_bytes()
				);
			}
		}
	}
}

fn pick_instruction_size(depth: u64) -> usize {
	if depth <= 15 {
		1
	} else if depth <= u8::MAX as u64 {
		2
	} else if depth <= u16::MAX as u64 {
		3
	} else if depth <= u32::MAX as u64 {
		5
	} else {
		9
	}
}

fn instruction_size(instruction: &AssemblyInstruction) -> usize {
	match instruction {
		AssemblyInstruction::Push(integer_literal) => {
			integer_literal_push_size(integer_literal)
		}

		AssemblyInstruction::Pick(depth) => pick_instruction_size(*depth),

		AssemblyInstruction::DataHeader(length) => {
			data_header_size(*length)
		}

		AssemblyInstruction::RawData(bytes) => bytes.len(),

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
			| AssemblyInstruction::Pop
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
			| AssemblyInstruction::RegisterGet(_)
			| AssemblyInstruction::RegisterSet(_)
			| AssemblyInstruction::RegGetIndirect
			| AssemblyInstruction::RegSetIndirect
			| AssemblyInstruction::MemLoad8
			| AssemblyInstruction::MemLoad16
			| AssemblyInstruction::MemLoad32
			| AssemblyInstruction::MemLoad64
			| AssemblyInstruction::MemStore8
			| AssemblyInstruction::MemStore16
			| AssemblyInstruction::MemStore32
			| AssemblyInstruction::MemStore64
			| AssemblyInstruction::MemStore128
			| AssemblyInstruction::MemStore256
			| AssemblyInstruction::MemStore512
			| AssemblyInstruction::MemStore1024
			| AssemblyInstruction::MemSize
			| AssemblyInstruction::MemGrow
			| AssemblyInstruction::MemCopy
			| AssemblyInstruction::MemFill
			| AssemblyInstruction::MemBytecodeCopy
			| AssemblyInstruction::Return
			| AssemblyInstruction::JumpIndirect
			| AssemblyInstruction::JumpIfZeroIndirect
			| AssemblyInstruction::CallIndirect
			| AssemblyInstruction::HostCallIndirect => 1,

		AssemblyInstruction::HostCall(host_call_id) => {
			if direct_host_call_opcode(host_call_id).is_some() {
				1
			} else {
				integer_literal_push_size(host_call_id) + 1
			}
		}
		
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

fn relax_relative_instructions(
	assembly_program: &mut AssemblyProgram,
	layout: &mut FenwickTree,
) -> Result<(), String> {
	let dependencies = build_relative_dependency_tree(assembly_program)?;

	let mut queue = VecDeque::new();
	let mut queued = vec![false; assembly_program.instructions.len()];

	for &instruction_index in &assembly_program.relative_instruction_indices {
		queue.push_back(instruction_index);
		queued[instruction_index] = true;
	}

	let mut affected_indices = Vec::new();

	while let Some(instruction_index) = queue.pop_front() {
		queued[instruction_index] = false;

		let (label, width) =
			match &assembly_program.instructions[instruction_index] {
				AssemblyInstruction::Relative {
					label,
					width,
					..
				} => (*label, *width),

				_ => unreachable!(
					"relative instruction index should point to a relative instruction"
				),
			};

		let instruction_size =
			relative_width_instruction_size(width);

		let instruction_address =
			layout.prefix_sum(instruction_index);

		let target_boundary =
			assembly_program.labels.instruction_boundary(label)?;

		let target_address =
			layout.prefix_sum(target_boundary);

		let next_instruction_address =
			instruction_address + instruction_size;

		let offset =
			target_address as i128 - next_instruction_address as i128;

		if relative_width_offset_fits(width, offset) {
			continue;
		}

		let wider_width = next_relative_width(width)
			.ok_or_else(|| {
				format!(
					"relative target is too far away: {}",
					assembly_program.labels.name(label),
				)
			})?;

		let new_instruction_size =
			relative_width_instruction_size(wider_width);

		let AssemblyInstruction::Relative {
			width,
			..
		} = &mut assembly_program.instructions[instruction_index]
		else {
			unreachable!(
				"relative instruction index should point to a relative instruction"
			);
		};

		*width = wider_width;

		layout.add(
			instruction_index,
			new_instruction_size - instruction_size,
		);

		if !queued[instruction_index] {
			queue.push_back(instruction_index);
			queued[instruction_index] = true;
		}

		dependencies.query_point(
			instruction_index,
			&mut affected_indices,
		);

		for &affected_index in &affected_indices {
			if !queued[affected_index] {
				queue.push_back(affected_index);
				queued[affected_index] = true;
			}
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

fn parse_register_index(text: &str) -> Result<u8, String> {
	let literal = parse_integer_literal(text)?;

	let value = match literal {
		IntegerLiteral::Unsigned(value) => value,
		IntegerLiteral::Signed(value) if value >= 0 => value as u64,
		IntegerLiteral::Signed(_) => {
			return Err("register index cannot be negative".to_string());
		}
	};

	if value >= REGISTER_COUNT as u64 {
		return Err(format!(
			"register index must be between 0 and {}",
			REGISTER_COUNT - 1,
		));
	}

	Ok(value as u8)
}

fn build_relative_dependency_tree(
	assembly_program: &AssemblyProgram,
) -> Result<SegmentTree, String> {
	let mut dependencies =
		SegmentTree::new(assembly_program.instructions.len());

	for &instruction_index in &assembly_program.relative_instruction_indices {
		let label =
			match &assembly_program.instructions[instruction_index] {
				AssemblyInstruction::Relative {
					label,
					..
				} => *label,

				_ => unreachable!(
					"relative instruction index should point to a relative instruction"
				),
			};

		let target_boundary =
			assembly_program.labels.instruction_boundary(label)?;

		if target_boundary > instruction_index {
			dependencies.add_range(
				instruction_index + 1,
				target_boundary,
				instruction_index,
			);
		} else {
			dependencies.add_range(
				target_boundary,
				instruction_index + 1,
				instruction_index,
			);
		}
	}

	Ok(dependencies)
}

#[cfg(test)]
mod tests {
	use super::*;

	struct CapturingHost {
		copied: Vec<u8>,
	}

	impl crate::host::Host for CapturingHost {
		fn call(
			&mut self,
			id: u64,
			context: &mut crate::host::HostContext,
		) -> crate::host::HostCallOutcome {
			assert_eq!(id, 0);

			let length = context.pop_word() as usize;
			let address = context.pop_word() as usize;

			self.copied =
				context.memory()[address..address + length].to_vec();

			crate::host::HostCallOutcome::Continue
		}
	}

	fn relax_source(source: &str) -> AssemblyProgram {
		let mut assembly_program = parse_source(source).unwrap();

		let instruction_sizes: Vec<usize> = assembly_program
			.instructions
			.iter()
			.map(instruction_size)
			.collect();

		let mut layout = FenwickTree::from_slice(&instruction_sizes);

		relax_relative_instructions(
			&mut assembly_program,
			&mut layout,
		).unwrap();

		assembly_program
	}

	fn relative_width(
		assembly_program: &AssemblyProgram,
		instruction_index: usize,
	) -> RelativeWidth {
		match &assembly_program.instructions[instruction_index] {
			AssemblyInstruction::Relative { width, .. } => *width,
			_ => panic!("expected relative instruction"),
		}
	}

	#[test]
	fn forward_offset_7_uses_signed4() {
		let source = format!(
			"JMP target\n{}target:\nHALT",
			"NOOP\n".repeat(7),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 0),
			RelativeWidth::Signed4,
		);
	}

	#[test]
	fn forward_offset_8_uses_signed8() {
		let source = format!(
			"JMP target\n{}target:\nHALT",
			"NOOP\n".repeat(8),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 0),
			RelativeWidth::Signed8,
		);
	}

	#[test]
	fn backward_offset_negative_8_uses_signed4() {
		let source = format!(
			"target:\n{}JMP target",
			"NOOP\n".repeat(7),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 7),
			RelativeWidth::Signed4,
		);
	}

	#[test]
	fn backward_offset_negative_9_uses_signed8() {
		let source = format!(
			"target:\n{}JMP target",
			"NOOP\n".repeat(8),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 8),
			RelativeWidth::Signed8,
		);
	}

	#[test]
	fn widening_one_instruction_can_force_another_to_widen() {
		let source = format!(
			"JMP near\nJMP far\n{}near:\n{}far:\nHALT",
			"NOOP\n".repeat(6),
			"NOOP\n".repeat(8),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 0),
			RelativeWidth::Signed8,
		);

		assert_eq!(
			relative_width(&assembly_program, 1),
			RelativeWidth::Signed8,
		);
	}

	#[test]
	fn forward_offset_127_uses_signed8() {
		let source = format!(
			"JMP target\n{}target:\nHALT",
			"NOOP\n".repeat(127),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 0),
			RelativeWidth::Signed8,
		);
	}

	#[test]
	fn forward_offset_128_uses_signed16() {
		let source = format!(
			"JMP target\n{}target:\nHALT",
			"NOOP\n".repeat(128),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 0),
			RelativeWidth::Signed16,
		);
	}

	#[test]
	fn backward_offset_negative_128_uses_signed8() {
		let source = format!(
			"target:\n{}JMP target",
			"NOOP\n".repeat(126),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 126),
			RelativeWidth::Signed8,
		);
	}

	#[test]
	fn backward_offset_negative_129_uses_signed16() {
		let source = format!(
			"target:\n{}JMP target",
			"NOOP\n".repeat(127),
		);

		let assembly_program = relax_source(&source);

		assert_eq!(
			relative_width(&assembly_program, 127),
			RelativeWidth::Signed16,
		);
	}

	#[test]
	fn push_address_uses_relaxed_layout() {
		let source = format!(
			"PUSH target\nJMP target\n{}target:\nHALT",
			"NOOP\n".repeat(8),
		);

		let program = assemble(&source).unwrap();

		let pushed_address = u64::from_le_bytes(
			program[1..9]
				.try_into()
				.unwrap(),
		);

		assert_eq!(pushed_address, 19);
	}

	fn check_data_copy(literal: &str, expected: &[u8]) {
		let source = format!(
			"PUSH {}\nMEMGRW\nMEMDTCP {}\nHOST 0\nHALT\n",
			expected.len(),
			literal,
		);

		let program = assemble(&source).unwrap();
		let mut host = CapturingHost { copied: Vec::new() };

		let result = crate::vm::run_program(&program, &mut host);

		assert_eq!(host.copied, expected);
		assert!(result.stack.is_empty());
	}

	#[test]
	fn data_copy_simple_and_escaped_strings() {
		check_data_copy(r#""Hi!\n""#, b"Hi!\n");

		check_data_copy(
			r#""a;b" ; semicolon is part of the string"#,
			b"a;b",
		);

		check_data_copy(
			r#""A\0\t\r\\\"\xFF""#,
			b"A\0\t\r\\\"\xFF",
		);

		check_data_copy(r#""hi 😄""#, "hi 😄".as_bytes());

		check_data_copy(r#""""#, b"");
	}

	#[test]
	fn data_copy_updates_label_layout_before_relaxation() {
		let program = assemble(
			"PUSH after\nJMP after\nMEMDTCP \"ABCDEFGHIJKLMNOP\"\nafter:\nHALT",
		).unwrap();

		let address =
			u64::from_le_bytes(program[1..9].try_into().unwrap()) as usize;

		assert_eq!(program[address], OpcodeByte::Halt as u8);
	}

	#[test]
	fn data_copy_rejects_invalid_strings() {
		for source in [
			"MEMDTCP unquoted",
			"MEMDTCP \"unterminated",
			"MEMDTCP \"bad\\q\"",
			"MEMDTCP \"bad\\x0\"",
			"MEMDTCP \"fine\" extra",
			"MEMDTCP",
		] {
			assert!(assemble(source).is_err(), "accepted: {}", source);
		}
	}

	#[test]
	fn data_copy_all_store_widths_and_unaligned_destination() {
		for length in [1usize, 2, 4, 8, 16, 32, 64, 128, 129, 255, 256] {
			let text: String = (0..length)
				.map(|index| (b'A' + (index % 26) as u8) as char)
				.collect();

			let literal = format!("\"{}\"", text);

			check_data_copy(&literal, text.as_bytes());
		}

		let source =
			"PUSH 19\nMEMGRW\nPUSH 1\nMEMDTCP \"ABCDEFGH\"\nHOST 0\nHALT";

		let program = assemble(source).unwrap();
		let mut host = CapturingHost { copied: Vec::new() };

		let result = crate::vm::run_program(&program, &mut host);

		assert_eq!(host.copied, b"ABCDEFGH");
		assert_eq!(result.stack, vec![0]);
	}

	#[test]
	fn label_references_are_case_insensitive() {
		assert_eq!(
			assemble("JMP FiNiSh\nfinish:\nHALT").unwrap(),
			assemble("JMP finish\nfinish:\nHALT").unwrap(),
		);

		assert_eq!(
			assemble("CALL SuBrOuTiNe\nsubroutine:\nRT").unwrap(),
			assemble("CALL subroutine\nsubroutine:\nRT").unwrap(),
		);

		assert_eq!(
			assemble("PUSH TaRgEt\ntarget:\nHALT").unwrap(),
			assemble("PUSH target\ntarget:\nHALT").unwrap(),
		);
	}

	#[test]
	fn label_definitions_cannot_differ_only_by_case() {
		let result = assemble(
			"Start:\nNOOP\nSTART:\nHALT",
		);

		assert!(result.is_err());
		assert!(result.unwrap_err().contains("duplicate label"));
	}

	#[test]
	fn data_uses_correct_header_encodings() {
		for length in [
			0usize, 1, 2, 12, 13, 255, 256, 65535, 65536,
		] {
			let text = "A".repeat(length);

			let source = format!(
				"DATA Example \"{}\"\nPUSH eXaMpLe\nHALT",
				text,
			);

			let program = assemble(&source).unwrap();

			let (expected_opcode, payload_start) = if length == 0 {
				(OpcodeByte::DataUnsigned8 as u8, 2)
			} else if length <= 12 {
				(
					OpcodeByte::DataLength1 as u8 + length as u8 - 1,
					1,
				)
			} else if length <= 255 {
				(OpcodeByte::DataUnsigned8 as u8, 2)
			} else if length <= 65535 {
				(OpcodeByte::DataUnsigned16 as u8, 3)
			} else {
				(OpcodeByte::DataUnsigned32 as u8, 5)
			};

			assert_eq!(program[0], expected_opcode);

			assert_eq!(
				&program[payload_start..payload_start + length],
				text.as_bytes(),
			);

			let push_start = payload_start + length;

			assert_eq!(
				program[push_start],
				OpcodeByte::Push64 as u8,
			);

			let address = u64::from_le_bytes(
				program[push_start + 1..push_start + 9]
					.try_into()
					.unwrap(),
			);

			assert_eq!(address as usize, payload_start);
		}
	}

	#[test]
	fn named_data_can_be_copied_into_linear_memory() {
		let expected = "Hello; world! 😄\n".as_bytes();

		let source = format!(
			"DATA Greeting \"Hello; world! 😄\\n\"\n\
			PUSH {}\n\
			MEMGRW\n\
			MEMDTCP gReEtInG\n\
			HOST WROUT\n\
			HALT",
			expected.len(),
		);

		let program = assemble(&source).unwrap();

		let mut host = CapturingHost {
			copied: Vec::new(),
		};

		let result = crate::vm::run_program(&program, &mut host);

		assert_eq!(host.copied, expected);
		assert!(result.stack.is_empty());
	}

	#[test]
	fn data_names_must_be_unique_and_declared_before_copying() {
		assert!(
			assemble(
				"MEMDTCP greeting\nDATA greeting \"Hello\"",
			).is_err()
		);

		assert!(
			assemble(
				"DATA Greeting \"Hello\"\nDATA greeting \"World\"",
			).is_err()
		);

		assert!(
			assemble(
				"DATA greeting \"Hello\"\nGREETING:\nHALT",
			).is_err()
		);

		assert!(
			assemble(
				"Greeting:\nHALT\nDATA greeting \"Hello\"",
			).is_err()
		);
	}

	#[test]
	fn execution_skips_embedded_data() {
		let program = assemble(
			"PUSH 4\n\
			DATA unused \"\\xFF\\xFF\\xFF\"\n\
			PUSH 5\n\
			ADD\n\
			HALT",
		).unwrap();

		let mut host = CapturingHost {
			copied: Vec::new(),
		};

		let result = crate::vm::run_program(&program, &mut host);

		assert_eq!(result.stack, vec![9]);
	}

	#[test]
	fn data_addresses_survive_branch_relaxation() {
		let source = format!(
			"DATA message \"{}\"\n\
			PUSH MESSAGE\n\
			JMP finish\n\
			{}\
			finish:\n\
			HALT",
			"A".repeat(200),
			"NOOP\n".repeat(140),
		);

		let program = assemble(&source).unwrap();

		assert_eq!(
			program[0],
			OpcodeByte::DataUnsigned8 as u8,
		);

		assert_eq!(program[1], 200);

		// The DATA header occupies two bytes.
		let push_start = 202;

		assert_eq!(
			program[push_start],
			OpcodeByte::Push64 as u8,
		);

		let address = u64::from_le_bytes(
			program[push_start + 1..push_start + 9]
				.try_into()
				.unwrap(),
		);

		assert_eq!(address, 2);

		assert_eq!(
			program[push_start + 9],
			OpcodeByte::JumpRelSigned16 as u8,
		);
	}

	fn make_include_test_directory() -> PathBuf {
		use std::sync::atomic::{AtomicU64, Ordering};

		static NEXT_ID: AtomicU64 = AtomicU64::new(0);

		let directory = std::env::temp_dir().join(format!(
			"bytecode_substrate_inc_{}_{}",
			std::process::id(),
			NEXT_ID.fetch_add(1, Ordering::Relaxed),
		));

		fs::create_dir_all(&directory).unwrap();

		directory
	}

	#[test]
	fn include_supports_nested_files_and_cross_file_labels() {
		let directory = make_include_test_directory();
		let library_directory = directory.join("lib");

		fs::create_dir_all(&library_directory).unwrap();

		let main_file = directory.join("main.sub.asm");

		fs::write(
			&main_file,
			"JMP main\n\
			INC \"lib/outer.sub.asm\" ; nested library\n\
			main:\n\
			PUSH 21\n\
			CALL double\n\
			HALT\n",
		).unwrap();

		fs::write(
			library_directory.join("outer.sub.asm"),
			"INC \"inner.sub.asm\"\n",
		).unwrap();

		fs::write(
			library_directory.join("inner.sub.asm"),
			"double:\nDUP\nADD\nRT\n",
		).unwrap();

		let program = assemble_file(&main_file).unwrap();

		let mut host = CapturingHost {
			copied: Vec::new(),
		};

		let result = crate::vm::run_program(&program, &mut host);

		assert_eq!(result.stack, vec![42]);

		fs::remove_dir_all(directory).unwrap();
	}

	#[test]
	fn include_rejects_cycles() {
		let directory = make_include_test_directory();

		let first = directory.join("first.sub.asm");
		let second = directory.join("second.sub.asm");

		fs::write(
			&first,
			"INC \"second.sub.asm\"\n",
		).unwrap();

		fs::write(
			&second,
			"INC \"first.sub.asm\"\n",
		).unwrap();

		let error = assemble_file(&first).unwrap_err();

		assert!(error.contains("circular INC"));

		fs::remove_dir_all(directory).unwrap();
	}
}
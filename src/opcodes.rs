use num_enum::TryFromPrimitive;

pub const PUSH_UNSIGNED4_START: u8 = 0x90;
pub const PUSH_UNSIGNED4_END: u8 = 0x9F;

pub const PUSH_SIGNED4_START: u8 = 0xA0;
pub const PUSH_SIGNED4_END: u8 = 0xAF;

pub const CALL_REL_SIGNED4_START: u8 = 0xB0;
pub const CALL_REL_SIGNED4_END: u8 = 0xBF;

pub const JUMP_IF_ZERO_REL_SIGNED4_START: u8 = 0xC0;
pub const JUMP_IF_ZERO_REL_SIGNED4_END: u8 = 0xCF;

pub const JUMP_REL_SIGNED4_START: u8 = 0xD0;
pub const JUMP_REL_SIGNED4_END: u8 = 0xDF;

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeByte {
	Noop                           = 0x00,

	Add                            = 0x01,
	Sub                            = 0x02,
	Mul                            = 0x03,

	DivUnsigned                    = 0x04,
	DivSigned                      = 0x05,
	RemUnsigned                    = 0x06,
	RemSigned                      = 0x07,

	BitAnd                         = 0x08,
	BitOr                          = 0x09,
	BitXor                         = 0x0A,
	BitNot                         = 0x0B,

	ShiftLeft                      = 0x0C,
	LogicShiftRight                = 0x0D,
	ArithShiftRight                = 0x0E,

	Dup                            = 0x0F,
	Pop                            = 0x10,
	Swap                           = 0x11,

	PushUnsigned8                  = 0x12,
	PushSigned8                    = 0x13,
	PushUnsigned16                 = 0x14,
	PushSigned16                   = 0x15,
	PushUnsigned32                 = 0x16,
	PushSigned32                   = 0x17,
	Push64                         = 0x18,


	CompEqual                      = 0x19,
	CompNotEqual                   = 0x1A,
	CompLessThanUnsigned           = 0x1B,
	CompLessThanSigned             = 0x1C,
	CompLessThanOrEqualUnsigned    = 0x1D,
	CompLessThanOrEqualSigned      = 0x1E,
	CompGreaterThanUnsigned        = 0x1F,
	CompGreaterThanSigned          = 0x20,
	CompGreaterThanOrEqualUnsigned = 0x21,
	CompGreaterThanOrEqualSigned   = 0x22,

	JumpRelSigned8                 = 0x23,
	JumpRelSigned16                = 0x24,
	JumpRelSigned32                = 0x25,
	JumpRelSigned64                = 0x26,

	JumpIfZeroRelSigned8           = 0x27,
	JumpIfZeroRelSigned16          = 0x28,
	JumpIfZeroRelSigned32          = 0x29,
	JumpIfZeroRelSigned64          = 0x2A,

	CallRelSigned8                 = 0x2B,
	CallRelSigned16                = 0x2C,
	CallRelSigned32                = 0x2D,
	CallRelSigned64                = 0x2E,

	JumpAbsIndirect                = 0x2F,
	CallAbsIndirect                = 0x30,
	Return                         = 0x31,

	MemLoad8                       = 0x32,
	MemLoad16                      = 0x33,
	MemLoad32                      = 0x34,
	MemLoad64                      = 0x35,

	MemStore8                      = 0x36,
	MemStore16                     = 0x37,
	MemStore32                     = 0x38,
	MemStore64                     = 0x39,

	MemSize                        = 0x3A,
	MemGrow                        = 0x3B,
	MemCopy8                       = 0x3C,
	MemFill8                       = 0x3D, // maybe add MemZeroFill later

	// 0x3E–0x8F currently free

	// 0x90–0x9F = PushUnsigned4
	// Unsigned 4-bit value stored in the low nibble.

	// 0xA0–0xAF = PushSigned4
	// Signed 4-bit value stored in the low nibble.

	// 0xB0–0xBF = CallRelSigned4
	// Signed 4-bit relative offset stored in the low nibble.

	// 0xC0–0xCF = JumpIfZeroRelSigned4
	// Signed 4-bit relative offset stored in the low nibble.

	// 0xD0–0xDF = JumpRelSigned4
	// Signed 4-bit relative offset stored in the low nibble.

	// 0xE0–0xFE currently free

	Halt                           = 0xFF,
}
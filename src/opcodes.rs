use num_enum::TryFromPrimitive;

pub const CALL_REL_SIGNED4_START: u8 = 0xB0;
pub const CALL_REL_SIGNED4_END: u8 = 0xBF;

pub const JUMP_IF_ZERO_REL_SIGNED4_START: u8 = 0xC0;
pub const JUMP_IF_ZERO_REL_SIGNED4_END: u8 = 0xCF;

pub const JUMP_REL_SIGNED4_START: u8 = 0xD0;
pub const JUMP_REL_SIGNED4_END: u8 = 0xDF;

pub const PUSH_UNSIGNED4_START: u8 = 0xE0;
pub const PUSH_UNSIGNED4_END: u8 = 0xEF;

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
	Drop                           = 0x10,
	Swap                           = 0x11,

	PushUnsigned8                  = 0x12,
	PushUnsigned16                 = 0x13,
	PushUnsigned32                 = 0x14,
	Push64                         = 0x15,

	CompEqual                      = 0x16,
	CompNotEqual                   = 0x17,
	CompLessThanUnsigned           = 0x18,
	CompLessThanSigned             = 0x19,
	CompLessThanOrEqualUnsigned    = 0x1A,
	CompLessThanOrEqualSigned      = 0x1B,
	CompGreaterThanUnsigned        = 0x1C,
	CompGreaterThanSigned          = 0x1D,
	CompGreaterThanOrEqualUnsigned = 0x1E,
	CompGreaterThanOrEqualSigned   = 0x1F,

	JumpRelSigned8                 = 0x20,
	JumpRelSigned16                = 0x21,
	JumpRelSigned32                = 0x22,
	JumpRelSigned64                = 0x23,

	JumpIfZeroRelSigned8           = 0x24,
	JumpIfZeroRelSigned16          = 0x25,
	JumpIfZeroRelSigned32          = 0x26,
	JumpIfZeroRelSigned64          = 0x27,

	CallRelSigned8                 = 0x28,
	CallRelSigned16                = 0x29,
	CallRelSigned32                = 0x2A,
	CallRelSigned64                = 0x2B,

	JumpAbsIndirect                = 0x2C,
	CallAbsIndirect                = 0x2D,
	Return                         = 0x2E,

	MemLoad8                       = 0x2F,
	MemLoad16                      = 0x30,
	MemLoad32                      = 0x31,
	MemLoad64                      = 0x32,

	MemStore8                      = 0x33,
	MemStore16                     = 0x34,
	MemStore32                     = 0x35,
	MemStore64                     = 0x36,

	MemSize                        = 0x37,
	MemGrow                        = 0x38,

    MemCopy8                       = 0x39,
	MemFill8                       = 0x3A,

	// 0x3B–0xAF currently free

	// 0xB0–0xBF = CallRelSigned4, with signed 4-bit relative offset encoded in low nibble
	// 0xC0–0xCF = JumpIfZeroRelSigned4, with signed 4-bit relative offset encoded in low nibble
	// 0xD0–0xDF = JumpRelSigned4, with signed 4-bit relative offset encoded in low nibble
	// 0xE0–0xEF = PushUnsigned4, with unsigned 4-bit value encoded in low nibble

	// 0xF0–0xFE currently free

	Halt                           = 0xFF,
}
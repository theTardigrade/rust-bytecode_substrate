use num_enum::TryFromPrimitive;

pub const PICK_UNSIGNED4_START: u8 = 0x40;
pub const PICK_UNSIGNED4_END: u8 = 0x4F;

pub const REGISTER_GET_START: u8 = 0x60;
pub const REGISTER_GET_END: u8 = 0x6F;

pub const REGISTER_SET_START: u8 = 0x70;
pub const REGISTER_SET_END: u8 = 0x7F;

pub const REGISTER_COUNT: usize = (REGISTER_GET_END - REGISTER_GET_START + 1) as usize;

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

pub const HOST_CALL_DIRECT_START: u8 = 0xE0;
pub const HOST_CALL_DIRECT_END: u8 = 0xFD;

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

	Pop                            = 0x0F,
	Swap                           = 0x10,

	PushUnsigned8                  = 0x11,
	PushSigned8                    = 0x12,
	PushUnsigned16                 = 0x13,
	PushSigned16                   = 0x14,
	PushUnsigned32                 = 0x15,
	PushSigned32                   = 0x16,
	Push64                         = 0x17,

	CompEqual                      = 0x18,
	CompNotEqual                   = 0x19,
	CompLessThanUnsigned           = 0x1A,
	CompLessThanSigned             = 0x1B,
	CompLessThanOrEqualUnsigned    = 0x1C,
	CompLessThanOrEqualSigned      = 0x1D,
	CompGreaterThanUnsigned        = 0x1E,
	CompGreaterThanSigned          = 0x1F,
	CompGreaterThanOrEqualUnsigned = 0x20,
	CompGreaterThanOrEqualSigned   = 0x21,

	JumpRelSigned8                 = 0x22,
	JumpRelSigned16                = 0x23,
	JumpRelSigned32                = 0x24,
	JumpRelSigned64                = 0x25,

	JumpIfZeroRelSigned8           = 0x26,
	JumpIfZeroRelSigned16          = 0x27,
	JumpIfZeroRelSigned32          = 0x28,
	JumpIfZeroRelSigned64          = 0x29,

	CallRelSigned8                 = 0x2A,
	CallRelSigned16                = 0x2B,
	CallRelSigned32                = 0x2C,
	CallRelSigned64                = 0x2D,

	JumpAbsIndirect                = 0x2E,
	CallAbsIndirect                = 0x2F,
	Return                         = 0x30,

	MemLoad8                       = 0x31,
	MemLoad16                      = 0x32,
	MemLoad32                      = 0x33,
	MemLoad64                      = 0x34,

	MemStore8                      = 0x35,
	MemStore16                     = 0x36,
	MemStore32                     = 0x37,
	MemStore64                     = 0x38,

	MemSize                        = 0x39,
	MemGrow                        = 0x3A,
	MemCopy8                       = 0x3B,
	MemFill8                       = 0x3C,

	JumpIfZeroAbsIndirect          = 0x3D,

	// 0x3E–0x3F currently free

	// 0x40–0x4F = PickUnsigned4
	// Unsigned 4-bit stack depth stored in the low nibble. (Depth 0 duplicates the top stack value.)

	PickUnsigned8                  = 0x50,
	PickUnsigned16                 = 0x51,
	PickUnsigned32                 = 0x52,
	PickUnsigned64                 = 0x53,

	// 0x54–0x5F currently free

	// 0x60–0x6F = RegisterGet
	// Pushes the value of the register identified by the low nibble.

	// 0x70–0x7F = RegisterSet
	// Pops a value and stores it in the register identified by the low nibble.

	RegGetIndirect                 = 0x80,
	RegSetIndirect                 = 0x81,

	// 0x82–0x8F currently free

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

	// 0xE0–0xFD = direct host calls (IDs 0–29)

	HostCallIndirect = 0xFE,
	Halt = 0xFF,
}

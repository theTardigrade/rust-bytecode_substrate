use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeNibble {
	Noop            = 0x0,
	Push4           = 0x1,
	Add             = 0x2,
	Sub             = 0x3,
	Mul             = 0x4,
	BitAnd          = 0x5,
	BitOr           = 0x6,
	BitXor          = 0x7,
	BitNot          = 0x8,
	ShiftLeft       = 0x9,
	LogicShiftRight = 0xA,
	ArithShiftRight = 0xB,
	Dup             = 0xC,
	Drop            = 0xD,
	Ext1            = 0xE,
	Ext2            = 0xF,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Ext1OpcodeNibble {
	Push8       = 0x0,
	Push16      = 0x1,
	Push32      = 0x2,
	Push64      = 0x3,
	DivUnsigned = 0x4,
	DivSigned   = 0x5,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Ext2OpcodeNibble {
	CompEqual                      = 0x0,
	CompNotEqual                   = 0x1,
	CompLessThanUnsigned           = 0x2,
	CompLessThanSigned             = 0x3,
	CompLessThanOrEqualUnsigned    = 0x4,
	CompLessThanOrEqualSigned      = 0x5,
	CompGreaterThanUnsigned        = 0x6,
	CompGreaterThanSigned          = 0x7,
	CompGreaterThanOrEqualUnsigned = 0x8,
	CompGreaterThanOrEqualSigned   = 0x9,
	Halt                           = 0xF,
}
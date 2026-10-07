mod memory;
use memory::Memory;
mod register_file;
use register_file::RegisterFile;
mod cpu;

pub enum ExecutionError {
    ReadFromUnbackedMemory,
    ReadFromUninitializedRegister,
    UnalignedRead,
    WrongModeBits,
    Breakpoint(u8),
}

impl From<register_file::RegisterError> for ExecutionError {
    fn from(e: register_file::RegisterError) -> Self {
        match e {
            register_file::RegisterError::UninitializedRegister => {
                Self::ReadFromUninitializedRegister
            }
            register_file::RegisterError::WrongModeBits => Self::WrongModeBits,
        }
    }
}

impl From<memory::FetchError> for ExecutionError {
    fn from(e: memory::FetchError) -> Self {
        match e {
            memory::FetchError::UnalignedAccess => Self::UnalignedRead,
        }
    }
}

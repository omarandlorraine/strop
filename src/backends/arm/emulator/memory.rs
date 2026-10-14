pub struct Memory {}

pub enum FetchError {
    UnalignedAccess,
}

impl Memory {
    pub fn get8(&self, addr: u32) -> Result<u8, FetchError> {
        todo!();
    }
    pub fn get16(&self, addr: u32) -> Result<u8, FetchError> {
        todo!();
    }
    pub fn get32(&self, addr: u32) -> Result<u32, FetchError> {
        todo!();
    }
}

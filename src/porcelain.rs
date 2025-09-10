//! This module contains functions and things that make strop a little more interoperable with the
//! conveniences offered by the standard library and other conventions. These are intended for
//! consumption by clients that want to abstract over different architectures and search
//! strategies, and which can rely on strop to do backend-specific static analysis and things.

use crate::Callable;
use crate::triplets::Triplet;

use crate::bruteforce::ToBruteForce;
use crate::mips::O32;

pub struct DisassemblyIterator(Box<dyn Iterator<Item = String>>);

impl std::fmt::Debug for DisassemblyIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "DisassemblyIterator{{...}}")
    }
}

#[derive(Debug)]
pub enum CannotSearchError {
    DoesNotSupportFloatingPoint,
}

impl Triplet {
    pub fn bruteforce_search_u16_to_u16<
        TargetFunction: Callable<u16, u16>,
    >(
        self,
        name: &str,
        target: TargetFunction,
    ) -> Result<DisassemblyIterator, CannotSearchError> {

        match self {
            Triplet::MipsUnknownLinuxGnu => Ok(DisassemblyIterator(Box::new(O32::default().to_bruteforce(target)))),
            Triplet::MipsUnknownLinuxMusl => todo!(),
            Triplet::MipsUnknownLinuxUclibc => todo!(),
            Triplet::Armv4tUnknownLinuxGnueabi => todo!(),
        }
    }
}

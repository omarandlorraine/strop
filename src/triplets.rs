//! Module dealing with strop's inventory of target triplets.
/*
 * #[cfg(feature = "armv4t")]
pub mod armv4t;
#[cfg(feature = "m6502")]
pub mod m6502;
#[cfg(feature = "m6809")]
pub mod m6809;
#[cfg(feature = "mips")]
pub mod mips;
#[cfg(feature = "z80")]
pub mod z80;
*/

/// Enumerates all target triplets known by strop. Since backends are gated behind Cargo features,
/// the enum's variants may vary from build to build.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum Triplet {
    #[cfg(feature = "mips")]
    MipsUnknownLinuxGnu,
    #[cfg(feature = "mips")]
    MipsUnknownLinuxMusl,
    #[cfg(feature = "mips")]
    MipsUnknownLinuxUclibc,
    #[cfg(feature = "armv4t")]
    Armv4tUnknownLinuxGnueabi,
}

impl Triplet {
    /// Returns a `Vec<Triplet>` containing all the target triplets that strop knows about.
    pub fn all() -> Vec<Self> {
        vec![
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxGnu,
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxMusl,
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxUclibc,
            #[cfg(feature = "armv4t")]
            Self::Armv4tUnknownLinuxGnueabi,
        ]
    }

    /// Returns the triplet specified by the string
    pub fn search(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|triplet| triplet.to_string() == name)
            .copied()
    }
}

impl std::fmt::Debug for Triplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self}")
    }
}

impl std::fmt::Display for Triplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::MipsUnknownLinuxGnu => write!(f, "mips-unknown-linux-gnu"),
            Self::MipsUnknownLinuxMusl => write!(f, "mips-unknown-linux-musl"),
            Self::MipsUnknownLinuxUclibc => write!(f, "mips-unknown-linux-uclibc"),
            Self::Armv4tUnknownLinuxGnueabi => write!(f, "armv4t-unknown-linux-gnueabi"),
        }
    }
}

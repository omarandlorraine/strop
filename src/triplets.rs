//! Module dealing with strop's inventory of target triplets.

/// Enumerates all target triplets known by strop. Since backends are gated behind Cargo features,
/// the enum's variants may vary from build to build.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug)]
pub enum Triplet {
    #[cfg(feature = "sm83")]
    Sm83UnknownSdcc,
    #[cfg(feature = "z80")]
    Z80UnknownSdcc,
    #[cfg(feature = "i8080")]
    I8080UnknownSdcc,
    #[cfg(feature = "mips")]
    MipsUnknownLinuxGnu,
    #[cfg(feature = "mips")]
    MipsUnknownLinuxMusl,
    #[cfg(feature = "mips")]
    MipsUnknownLinuxUclibc,
    #[cfg(feature = "armv4t")]
    Armv4tNoneEabi,
    #[cfg(feature = "armv4t")]
    Armv4tUnknownLinuxGnueabi,
}

impl Triplet {
    /// Returns a `Vec<Triplet>` containing all the target triplets that strop knows about.
    pub fn all() -> Vec<Self> {
        vec![
            #[cfg(feature = "i8080")]
            Self::I8080UnknownSdcc,
            #[cfg(feature = "sm83")]
            Self::Sm83UnknownSdcc,
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxGnu,
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxMusl,
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxUclibc,
            #[cfg(feature = "armv4t")]
            Self::Armv4tNoneEabi,
            #[cfg(feature = "armv4t")]
            Self::Armv4tUnknownLinuxGnueabi,
            #[cfg(feature = "z80")]
            Self::Z80UnknownSdcc,
        ]
    }

    /// Returns the triplet specified by the string
    pub fn search(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|triplet| triplet.to_string() == name)
            .copied()
    }

    /// Returns a Searchable
    pub fn pure_leaf_function_search<
        Input: 'static + crate::test::Parameters,
        Output: 'static + crate::test::ReturnValue,
        C: 'static + crate::Callable<Input, Output>,
    >(
        &self,
        target: C,
    ) -> Box<dyn crate::Testable> {
        match self {
            #[cfg(feature = "sm83")]
            Self::Sm83UnknownSdcc => Box::new(crate::search::Searcher::new(
                crate::backends::x80::SdccCall1::<crate::backends::sm83::Instruction>::default(),
                crate::test::FuzzTest::new(target),
            )),

            #[cfg(feature = "i8080")]
            Self::I8080UnknownSdcc => Box::new(crate::search::Searcher::new(
                crate::backends::x80::SdccCall1::<crate::backends::i8080::Instruction>::default(),
                crate::test::FuzzTest::new(target),
            )),

            #[cfg(feature = "z80")]
            Self::Z80UnknownSdcc => Box::new(crate::search::Searcher::new(
                crate::backends::x80::SdccCall1::<crate::backends::z80::Instruction>::default(),
                crate::test::FuzzTest::new(target),
            )),

            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxGnu
            | Self::MipsUnknownLinuxMusl
            | Self::MipsUnknownLinuxUclibc => Box::new(crate::search::Searcher::new(
                crate::backends::mips::o32::O32::<Input, Output>::default(),
                crate::test::FuzzTest::new(target),
            )),

            #[cfg(feature = "armv4t")]
            Self::Armv4tNoneEabi | Self::Armv4tUnknownLinuxGnueabi => {
                Box::new(crate::search::Searcher::new(
                    crate::backends::armv4t::Aapcs32::<Input, Output>::default(),
                    crate::test::FuzzTest::new(target),
                ))
            }
        }
    }
}

impl std::fmt::Display for Triplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            #[cfg(feature = "i8080")]
            Self::I8080UnknownSdcc => write!(f, "8080-unknown-sdcc"),
            #[cfg(feature = "sm83")]
            Self::Sm83UnknownSdcc => write!(f, "sm83-unknown-sdcc"),
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxGnu => write!(f, "mips-unknown-linux-gnu"),
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxMusl => write!(f, "mips-unknown-linux-musl"),
            #[cfg(feature = "mips")]
            Self::MipsUnknownLinuxUclibc => write!(f, "mips-unknown-linux-uclibc"),
            #[cfg(feature = "armv4t")]
            Self::Armv4tNoneEabi => write!(f, "armv4t-none-eabi"),
            #[cfg(feature = "armv4t")]
            Self::Armv4tUnknownLinuxGnueabi => write!(f, "armv4t-unknown-linux-gnueabi"),
            #[cfg(feature = "z80")]
            Self::Z80UnknownSdcc => write!(f, "z80-unknown-sdcc"),
        }
    }
}

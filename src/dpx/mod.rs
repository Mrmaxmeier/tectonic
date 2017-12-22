
pub mod error {
    use untrusted;

    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Unspecified;

    // This is required for the implementation of `std::error::Error`.
    impl Display for Unspecified {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            f.write_str("corrode::error::Unspecified")
        }
    }


    impl Error for Unspecified {
        #[inline]
        fn cause(&self) -> Option<&Error> { None }

        #[inline]
        fn description(&self) -> &str { "corrode::error::Unspecified" }
    }

    impl From<untrusted::EndOfInput> for Unspecified {
        fn from(_: untrusted::EndOfInput) -> Self { Unspecified }
    }
}

pub mod tfm;

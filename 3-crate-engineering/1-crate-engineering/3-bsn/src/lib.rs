use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug)]
/// Error creating BSN
pub enum Error {
    /// Invalid BSN length, should consists of 8 or 9 digits.
    InvalidBsnLength(usize),
    /// Invalid digit, should be 0 - 9.
    InvalidBsnDigit { position: usize, digit: u8 },
    /// 11 check failed
    Failed11Check,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidBsnLength(len) => {
                write!(f, "Invalid BSN number: expect 8 or 9 digits, get {len}")
            }
            Error::InvalidBsnDigit { position, digit } => {
                write!(
                    f,
                    "Invalid BSN number: invalid digit at index of {position}: '{digit}'"
                )
            }
            Error::Failed11Check => write!(f, "Invalid BSN number: failed 11 check"),
        }
    }
}

/// A valid BSN (burgerservicenummer), a Dutch
/// personal identification number that is similar
/// to the US Social Security Number.
/// More info (Dutch): https://www.rvig.nl/bsn
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Bsn {
    inner: String,
}

impl Bsn {
    /// Try to create a new BSN. Returns `Err` if the passed string
    /// does not represent a valid BSN
    pub fn try_from_string<B: ToString>(bsn: B) -> Result<Self, Error> {
        let bsn = bsn.to_string();
        Bsn::validate(&bsn)?;
        Ok(Bsn { inner: bsn })
    }

    /// Check whether the passed string represents a valid BSN.
    //  Returns `Err` if the passed string does not represent a valid BSN
    pub fn validate(bsn: &str) -> Result<(), Error> {
        let bsn = bsn.as_bytes();

        if bsn.len() != 8 && bsn.len() != 9 {
            return Err(Error::InvalidBsnLength(bsn.len()));
        }

        let mut checksum: i32 = 0;
        for (pos, &d) in bsn.iter().enumerate() {
            match d {
                b'0'..=b'9' => {
                    let factor = if pos == 8 { -1 } else { 9 - pos as i32 };
                    checksum += ((d - b'0') as i32) * factor;
                }
                _ => {
                    return Err(Error::InvalidBsnDigit {
                        position: pos,
                        digit: d,
                    })
                }
            }
        }
        if checksum % 11 != 0 {
            return Err(Error::Failed11Check);
        }

        Ok(())
    }
}

impl Serialize for Bsn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl<'de> Deserialize<'de> for Bsn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// A visitor for deserializing strings into `Bns`
        struct BsnVisitor;

        impl<'d> Visitor<'d> for BsnVisitor {
            type Value = Bsn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A string representing a valid BSN")
            }

            // TODO: Override the correct `Visitor::visit_*` to validate the input and output a new `BSN`
            // if the input represents a valid BSN. Note that we do not need to override all default methods

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Bsn::try_from_string(v).map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(BsnVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::Bsn;

    #[test]
    fn test_validation() {
        let bsns = include_str!("../valid_bsns.in").lines();
        bsns.for_each(|bsn| {
            assert!(
                Bsn::validate(bsn).is_ok(),
                "BSN {bsn} is valid, but did not pass validation"
            )
        });

        let bsns = include_str!("../invalid_bsns.in").lines();
        bsns.for_each(|bsn| {
            assert!(
                Bsn::validate(bsn).is_err(),
                "BSN {bsn} invalid, but passed validation"
            )
        });
    }

    #[test]
    fn test_serde() {
        let json = serde_json::to_string(&Bsn::try_from_string("999998456").unwrap()).unwrap();
        assert_eq!(json, "\"999998456\"");
        let bsn: Bsn = serde_json::from_str("\"999998456\"").unwrap();
        assert_eq!(bsn, Bsn::try_from_string("999998456".to_string()).unwrap());

        serde_json::from_str::<Bsn>("\"1112223333\"").unwrap_err();
    }
}

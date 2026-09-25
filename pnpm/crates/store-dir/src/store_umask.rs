use derive_more::{Display, Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// The `storeUmask` setting: the permission bits cleared from every file
/// pnpm writes to the store, in place of the process umask.
///
/// Files in `node_modules` are hardlinks to store files, so they share
/// the store file's mode. Without this setting that mode comes from the
/// umask of whichever process first wrote the file.
///
/// Written as octal digits, like a shell `umask` (`"002"`, `"022"`, `"077"`),
/// with an optional `0o` prefix. The owner's bits cannot be masked: pnpm
/// must read what it writes to the store, and executables must stay
/// executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreUmask(u32);

impl StoreUmask {
    /// The mode of a new store file: `0o666`, or `0o777` for an
    /// executable, without the masked bits.
    #[must_use]
    pub fn file_mode(self, executable: bool) -> u32 {
        let base = if executable { 0o777 } else { 0o666 };
        base & !self.0
    }
}

/// Error type of [`StoreUmask::from_str`].
#[derive(Debug, Display, Error)]
#[display("storeUmask must be an octal number from 000 to 077, got {value:?}")]
pub struct ParseStoreUmaskError {
    #[error(not(source))]
    pub value: String,
}

impl FromStr for StoreUmask {
    type Err = ParseStoreUmaskError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let error = || ParseStoreUmaskError { value: value.to_string() };
        let digits = value.strip_prefix("0o").unwrap_or(value);
        if digits.is_empty()
            || !digits
                .bytes()
                .all(|digit| matches!(digit, b'0'..=b'7'))
        {
            return Err(error());
        }
        match u32::from_str_radix(digits, 8) {
            Ok(mask) if mask <= 0o077 => Ok(StoreUmask(mask)),
            _ => Err(error()),
        }
    }
}

impl fmt::Display for StoreUmask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:03o}", self.0)
    }
}

impl Serialize for StoreUmask {
    fn serialize<Target: Serializer>(
        &self,
        serializer: Target,
    ) -> Result<Target::Ok, Target::Error> {
        serializer.collect_str(self)
    }
}

/// Reads the value as a string, so that the digits of an unquoted YAML
/// `022` are read as octal, not decimal.
impl<'de> Deserialize<'de> for StoreUmask {
    fn deserialize<Source: Deserializer<'de>>(deserializer: Source) -> Result<Self, Source::Error> {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;

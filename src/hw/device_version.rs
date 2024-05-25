use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeviceVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub pre_release: bool,
}

impl Display for DeviceVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}{}",
            self.major,
            self.minor,
            self.patch,
            if self.pre_release { "-pre" } else { "" }
        )
    }
}

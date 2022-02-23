use core::fmt;

/// Contains procedure metadata (without procedure body)
#[derive(Clone, Debug, PartialEq)]
pub struct ProcInfo {
    pub name: String,
    pub num_locals: usize,
}

impl fmt::Display for ProcInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.num_locals)?;

        Ok(())
    }
}

use crate::utils::string::String;

// PROC MARKER
// ================================================================================================

/// Contains information corresponsing to a ProcMarker decorator
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcMarker {
    ProcStarted(String, u32),
    ProcEnded,
}

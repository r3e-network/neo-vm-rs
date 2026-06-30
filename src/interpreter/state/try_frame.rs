#[derive(Debug, Clone)]
pub(in crate::interpreter) struct TryFrame {
    pub(in crate::interpreter) catch_ip: usize,
    pub(in crate::interpreter) finally_ip: usize,
    /// Whether this frame HAS a catch / finally block. Canonical NeoVM uses a
    /// `-1` sentinel in `CatchPointer`/`FinallyPointer` and tests
    /// `HasCatch = CatchPointer >= 0`; a block whose computed target is IP 0 (a
    /// backward TRY offset of `-ip`) is PRESENT, not absent. Storing presence as
    /// an explicit flag — rather than testing `catch_ip != 0` — keeps "present
    /// at IP 0" distinct from "absent", matching canonical and avoiding a
    /// HALT-vs-FAULT divergence on a crafted backward catch/finally target.
    pub(in crate::interpreter) has_catch: bool,
    pub(in crate::interpreter) has_finally: bool,
    pub(in crate::interpreter) call_depth: usize,
    pub(in crate::interpreter) caught: bool,
    pub(in crate::interpreter) in_finally: bool,
    pub(in crate::interpreter) end_ip: usize,
}

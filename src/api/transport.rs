#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonError { InvalidReplay, InvalidStateContinuity }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct State(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeConfig { valid: bool }

impl Default for RuntimeConfig { fn default() -> Self { Self { valid: true } } }
impl RuntimeConfig { pub const fn invalid() -> Self { Self { valid: false } } pub const fn is_structurally_valid(&self) -> bool { self.valid } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlEvent { pub state_after: State, pub valid: bool }

pub type TLog = Vec<ControlEvent>;

pub fn verify_tlog(tlog: &TLog) -> Result<(), CanonError> { if tlog.iter().all(|e| e.valid) { Ok(()) } else { Err(CanonError::InvalidReplay) } }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiTransportSession {
    pub cfg: RuntimeConfig,
    pub state: State,
    pub tlog: TLog,
}

impl ApiTransportSession {
    pub fn from_parts(cfg: RuntimeConfig, state: State, tlog: TLog) -> Result<Self, CanonError> {
        if !cfg.is_structurally_valid() {
            return Err(CanonError::InvalidReplay);
        }
        verify_tlog(&tlog)?;
        if let Some(event) = tlog.last() {
            if event.state_after != state {
                return Err(CanonError::InvalidStateContinuity);
            }
        }
        Ok(Self { cfg, state, tlog })
    }
}

pub fn verify_api_transport_receipts(tlog: &TLog) -> Result<(), CanonError> {
    verify_tlog(tlog)
}

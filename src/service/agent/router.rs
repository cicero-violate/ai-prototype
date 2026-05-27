//! Re-export shim — RouterClient lives in `capability::llm::browser_router`.
pub use crate::capability::llm::browser_router::{
    close_first_tab_with_timeout, close_tab_for_target_id_with_timeout,
    close_tab_for_url_with_timeout, RouterClient, RouterStreamingResult, RouterTabCloseOutcome,
    RouterTurnResult,
};

//! Re-export shim — RouterClient lives in `capability::llm::browser_router`.
pub use crate::capability::llm::browser_router::{
    close_all_tabs_with_timeout, close_tab_for_target_id_with_timeout,
    close_tab_for_url_with_timeout, RouterClient, RouterStreamingResult, RouterTabCloseOutcome,
    RouterTurnResult,
};

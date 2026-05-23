//! Agent and mailbox native tooling capability.

use serde_json::Value;

use super::ActionHost;
use crate::api::action::{result_with_warning, tool_error};
use crate::api::protocol::Command as KernelCommand;
use crate::capability::execution::action::{canon_read_mailbox, canon_send_agent_message};

pub async fn execute<H: ActionHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        "canon_spawn_agent"
        | "canon_runtime_state"
        | "canon_supervisor_health"
        | "canon_supervisor_reload_worker"
        | "canon_supervisor_restart"
        | "canon_workspace_get"
        | "canon_workspace_set"
        | "canon_browser_list_tabs"
        | "canon_browser_close_tab"
        | "canon_browser_upload"
        | "canon_browser_group_chat" => host
            .run_host_tool(name, args)
            .await
            .unwrap_or_else(|| tool_error(format!("Unknown host tool: {name}"))),
        "canon_send_agent_message" => run_authorized_mailbox_send(args, host).await,
        "canon_read_mailbox" => {
            let root = host.workspace().root;
            canon_read_mailbox::run(args, &root)
        }
        _ => tool_error(format!("Unknown agent tool: {name}")),
    }
}

async fn run_authorized_mailbox_send<H: ActionHost>(args: &Value, host: &H) -> Value {
    let parsed = match canon_send_agent_message::parse_args(args) {
        Ok(parsed) => parsed,
        Err(error) => return tool_error(error),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeMailboxMessage(parsed.request))
        .await
    {
        return tool_error(format!(
            "mailbox message denied before append by ai worker: {error}"
        ));
    }

    let root = host.workspace().root;
    match canon_send_agent_message::append_authorized(&parsed, &root) {
        Ok((receipt, result)) => {
            if let Err(error) = host
                .submit_kernel_command(KernelCommand::SubmitMailboxMessageReceipt(receipt))
                .await
            {
                return result_with_warning(
                    result,
                    format!("mailbox receipt recording failed in ai worker: {error}"),
                );
            }
            result
        }
        Err(error) => tool_error(error),
    }
}

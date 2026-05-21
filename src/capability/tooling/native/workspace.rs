//! Workspace native tooling capability: patch application and shell execution.

use serde_json::Value;

use super::NativeToolHost;
use crate::api::mcp::{result_with_warning, tool_error};
use crate::api::protocol::Command as KernelCommand;
use crate::capability::tooling::mcp_tools::{apply_patch, shell};

pub async fn execute<H: NativeToolHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        "apply_patch" => {
            let workspace = host.workspace();
            apply_patch::run(args, &workspace).await
        }
        "shell" => {
            let workspace = host.workspace();
            shell::run_unrecorded(args, &workspace).await
        }
        _ => tool_error(format!("Unknown workspace tool: {name}")),
    }
}

pub async fn execute_recorded_shell<H: NativeToolHost>(args: &Value, host: &H) -> Value {
    let workspace = host.workspace();
    let request = match shell::recorded_process_request(args, &workspace) {
        Ok(request) => request,
        Err(error) => return tool_error(error),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeProcessCall(request))
        .await
    {
        return tool_error(format!(
            "process execution denied before execution by ai worker: {error}"
        ));
    }

    let args_owned = args.clone();
    let run_result =
        tokio::task::spawn_blocking(move || shell::run_recorded_process(&args_owned, &workspace))
            .await;
    match run_result {
        Err(join_err) => tool_error(format!("shell task panicked: {join_err}")),
        Ok(Err(error)) => tool_error(error),
        Ok(Ok((receipt, stdout, stderr))) => {
            let result = shell::render_recorded_response(&receipt, &stdout, &stderr);
            if let Err(error) = host
                .submit_kernel_command(KernelCommand::SubmitProcessReceipt(receipt))
                .await
            {
                return result_with_warning(
                    result,
                    format!("process receipt recording failed in ai worker: {error}"),
                );
            }
            result
        }
    }
}

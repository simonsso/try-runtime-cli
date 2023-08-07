use std::{fmt::Debug, str::FromStr};

use frame_try_runtime::UpgradeCheckSelect;
use parity_scale_codec::{Decode, Encode};
use sc_executor::sp_wasm_interface::HostFunctions;
use sp_runtime::traits::{Block as BlockT, NumberFor};
use sp_weights::Weight;

use crate::{
    build_executor, state::State, state_machine_call_with_proof, SharedParams, LOG_TARGET,
};

/// Configuration for [`run`].
#[derive(Debug, Clone, clap::Parser)]
pub struct Command {
    /// The state type to use.
    #[command(subcommand)]
    pub state: State,

    /// Select which optional checks to perform. Selects all when no value is given.
    ///
    /// - `none`: Perform no checks.
    /// - `all`: Perform all checks (default when --checks is present with no value).
    /// - `pre-and-post`: Perform pre- and post-upgrade checks (default when the arg is not
    ///   present).
    /// - `try-state`: Perform the try-state checks.
    ///
    /// Performing any checks will potentially invalidate the measured PoV/Weight.
    // NOTE: The clap attributes make it backwards compatible with the previous `--checks` flag.
    #[clap(long,
		default_value = "pre-and-post",
		default_missing_value = "all",
		num_args = 0..=1,
		require_equals = true,
		verbatim_doc_comment)]
    pub checks: UpgradeCheckSelect,
}

// Runs the `on-runtime-upgrade` command.
pub(crate) async fn run<Block, HostFns>(
    shared: SharedParams,
    command: Command,
) -> sc_cli::Result<()>
where
    Block: BlockT + serde::de::DeserializeOwned,
    <Block::Hash as FromStr>::Err: Debug,
    Block::Header: serde::de::DeserializeOwned,
    NumberFor<Block>: FromStr,
    <NumberFor<Block> as FromStr>::Err: Debug,
    HostFns: HostFunctions,
{
    let executor = build_executor(&shared);
    let ext = command
        .state
        .into_ext::<Block, HostFns>(&shared, &executor, None, true)
        .await?;

    let (_, encoded_result) = state_machine_call_with_proof::<Block, HostFns>(
        &ext,
        &executor,
        "TryRuntime_on_runtime_upgrade",
        command.checks.encode().as_ref(),
        Default::default(), // we don't really need any extensions here.
        shared.export_proof,
    )?;

    let (weight, total_weight) = <(Weight, Weight) as Decode>::decode(&mut &*encoded_result)
        .map_err(|e| format!("failed to decode weight: {:?}", e))?;

    log::info!(
        target: LOG_TARGET,
        "TryRuntime_on_runtime_upgrade executed without errors. Consumed weight = ({} ps, {} byte), total weight = ({} ps, {} byte) ({:.2} %, {:.2} %).",
        weight.ref_time(), weight.proof_size(),
        total_weight.ref_time(), total_weight.proof_size(),
        (weight.ref_time() as f64 / total_weight.ref_time().max(1) as f64) * 100.0,
        (weight.proof_size() as f64 / total_weight.proof_size().max(1) as f64) * 100.0,
    );

    Ok(())
}
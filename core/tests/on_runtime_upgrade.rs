// This file is part of try-runtime-cli.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(unix)]

use std::time::Duration;

use assert_cmd::cargo::cargo_bin;
use substrate_cli_test_utils as common;
use tokio::process::Command;

#[tokio::test]
async fn on_runtime_upgrade_works() {
    common::run_with_timeout(Duration::from_secs(60), async move {
        let project_root = env!("CARGO_MANIFEST_DIR");
        fn on_runtime_upgrade(
            snap_path: &str,
            runtime_path: &str,
            extra_args: &[&str],
        ) -> tokio::process::Child {
            Command::new(cargo_bin("try-runtime"))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .arg(format!("--runtime={}", runtime_path))
                .arg("on-runtime-upgrade")
                .args(extra_args)
                .args(&["snap", format!("--path={}", snap_path).as_str()])
                .kill_on_drop(true)
                .spawn()
                .unwrap()
        }

        println!("Checking OK migration");
        let snap_path = format!("{}/tests/snaps/rococo-bridge-hub.snap", project_root);
        let runtime_path = format!(
            "{}/tests/runtimes/bridge_hub_rococo_runtime_OK.compact.compressed.wasm",
            project_root
        );
        let child = on_runtime_upgrade(snap_path.as_str(), runtime_path.as_str(), &[]);
        let out = child.wait_with_output().await.unwrap();
        assert!(out.status.success());

        println!("Checking migration with weight issue fails");
        let runtime_path = format!(
            "{}/tests/runtimes/bridge_hub_rococo_runtime_WEIGHT_ISSUE.compact.compressed.wasm",
            project_root
        );
        let child = on_runtime_upgrade(snap_path.as_str(), runtime_path.as_str(), &[]);
        let out = child.wait_with_output().await.unwrap();
        assert!(!out.status.success());

        println!("Checking weight issues can be ignored with an arg");
        let child = on_runtime_upgrade(
            snap_path.as_str(),
            runtime_path.as_str(),
            &["--no-weight-warnings"],
        );
        let out = child.wait_with_output().await.unwrap();
        assert!(out.status.success());

        println!("Checking migration with idempotency execution issue fails");
        let runtime_path = format!(
            "{}/tests/runtimes/bridge_hub_rococo_runtime_NOT_IDEMPOTENT_EXECUTION.compact.compressed.wasm",
            project_root
        );
        let child = on_runtime_upgrade(snap_path.as_str(), runtime_path.as_str(), &[]);
        let out = child.wait_with_output().await.unwrap();
        assert!(!out.status.success());

        println!("Checking idempotency execution issue can be ignored with an arg");
        let child = on_runtime_upgrade(
            snap_path.as_str(),
            runtime_path.as_str(),
            &["--no-idempotency-checks"],
        );
        let out = child.wait_with_output().await.unwrap();
        assert!(out.status.success());

        println!("Checking migration with idempotency state root issue fails");
        let runtime_path = format!(
            "{}/tests/runtimes/bridge_hub_rococo_runtime_NOT_IDEMPOTENT_STATE_ROOT.compact.compressed.wasm",
            project_root
        );
        let child = on_runtime_upgrade(snap_path.as_str(), runtime_path.as_str(), &[]);
        let out = child.wait_with_output().await.unwrap();
        assert!(!out.status.success());

        println!("Checking idempotency state root issue can be ignored with an arg");
        let child = on_runtime_upgrade(
            snap_path.as_str(),
            runtime_path.as_str(),
            &["--no-idempotency-checks"],
        );
        let out = child.wait_with_output().await.unwrap();
        assert!(out.status.success());
    })
    .await;
}
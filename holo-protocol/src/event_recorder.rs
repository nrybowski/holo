//
// Copyright (c) The Holo Core Contributors
//
// SPDX-License-Identifier: MIT
//

#![cfg_attr(feature = "testing", allow(dead_code, unused_variables))]

use std::io::Write;
use std::path::Path;

use holo_utils::protocol::Protocol;
use tracing::warn;

use crate::{InstanceMsg, ProtocolInstance};

pub struct EventRecorder(std::fs::File);

// ===== impl EventRecorder =====

impl EventRecorder {
    // Creates new event recorder.
    pub(crate) fn new(
        protocol: Protocol,
        instance: &str,
        dir: &Path,
    ) -> Option<EventRecorder> {
        // Get full file path.
        let path = dir.join(format!("holo-events-{protocol}-{instance}.jsonl"));

        // Create event recorder.
        match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
        {
            Ok(file) => Some(EventRecorder(file)),
            Err(error) => {
                warn!(%error, "couldn't write to file");
                None
            }
        }
    }

    // Records the protocol instance event.
    pub(crate) fn record<P>(&mut self, msg: &InstanceMsg<P>)
    where
        P: ProtocolInstance,
    {
        let event = serde_json::to_string(msg).unwrap();
        if let Err(error) = writeln!(self.0, "{event}") {
            warn!(%error, "couldn't write to file");
        }
    }
}

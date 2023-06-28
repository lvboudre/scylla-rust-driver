pub mod auth_response;
pub mod batch;
pub mod execute;
pub mod options;
pub mod prepare;
pub mod query;
pub mod register;
pub mod startup;

use crate::frame::frame_errors::ParseError;
use bytes::{BufMut, Bytes};
use num_enum::TryFromPrimitive;

pub use auth_response::AuthResponse;
pub use batch::Batch;
pub use options::Options;
pub use prepare::Prepare;
pub use query::Query;
pub use startup::Startup;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
#[repr(u8)]
pub enum RequestOpcode {
    Startup = 0x01,
    Options = 0x05,
    Query = 0x07,
    Prepare = 0x09,
    Execute = 0x0A,
    Register = 0x0B,
    Batch = 0x0D,
    AuthResponse = 0x0F,
}

pub trait Request {
    const OPCODE: RequestOpcode;

    fn serialize(&self, buf: &mut impl BufMut) -> Result<(), ParseError>;

    fn to_bytes(&self) -> Result<Bytes, ParseError> {
        let mut v = Vec::new();
        self.serialize(&mut v)?;
        Ok(v.into())
    }
}

/// Not intended for driver's direct usage (as driver has no interest in deserialising CQL requests),
/// but very useful for testing (e.g. asserting that the sent requests have proper parameters set).
pub trait DeserializableRequest: Request + Sized {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, ParseError>;
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, ops::Deref};

    use bytes::Bytes;

    use crate::{
        frame::{
            request::{
                batch::{Batch, BatchStatement, BatchType},
                execute::Execute,
                query::{Query, QueryParameters},
                DeserializableRequest, Request,
            },
            types::{self, LegacyConsistency, SerialConsistency},
            value::SerializedValues,
        },
        Consistency,
    };

    #[test]
    fn request_ser_de_identity() {
        // Query
        let contents = Cow::Borrowed("SELECT host_id from system.peers");
        let parameters = QueryParameters {
            consistency: Consistency::All,
            serial_consistency: Some(SerialConsistency::Serial),
            timestamp: None,
            page_size: Some(323),
            paging_state: Some(vec![2, 1, 3, 7].into()),
            values: {
                let mut vals = SerializedValues::new();
                vals.add_value(&2137).unwrap();
                Cow::Owned(vals)
            },
        };
        let query = Query {
            contents,
            parameters,
        };

        {
            let mut buf = Vec::new();
            query.serialize(&mut buf).unwrap();

            let query_deserialized = Query::deserialize(&mut &buf[..]).unwrap();
            assert_eq!(&query_deserialized, &query);
        }

        // Execute
        let id: Bytes = vec![2, 4, 5, 2, 6, 7, 3, 1].into();
        let parameters = QueryParameters {
            consistency: Consistency::Any,
            serial_consistency: None,
            timestamp: Some(3423434),
            page_size: None,
            paging_state: None,
            values: {
                let mut vals = SerializedValues::new();
                vals.add_named_value("the_answer", &42).unwrap();
                vals.add_named_value("really?", &2137).unwrap();
                Cow::Owned(vals)
            },
        };
        let execute = Execute { id, parameters };
        {
            let mut buf = Vec::new();
            execute.serialize(&mut buf).unwrap();

            let execute_deserialized = Execute::deserialize(&mut &buf[..]).unwrap();
            assert_eq!(&execute_deserialized, &execute);
        }

        // Batch
        let statements = vec![
            BatchStatement::Query {
                text: query.contents,
            },
            BatchStatement::Prepared {
                id: Cow::Borrowed(&execute.id),
            },
        ];
        let batch = Batch {
            statements: Cow::Owned(statements),
            batch_type: BatchType::Logged,
            consistency: Consistency::EachQuorum,
            serial_consistency: Some(SerialConsistency::LocalSerial),
            timestamp: Some(32432),

            // Not execute's values, because named values are not supported in batches.
            values: vec![
                query.parameters.values.deref().clone(),
                query.parameters.values.deref().clone(),
            ],
        };
        {
            let mut buf = Vec::new();
            batch.serialize(&mut buf).unwrap();

            let batch_deserialized = Batch::deserialize(&mut &buf[..]).unwrap();
            assert_eq!(&batch_deserialized, &batch);
        }
    }

    #[test]
    fn deser_rejects_unknown_flags() {
        // Query
        let contents = Cow::Borrowed("SELECT host_id from system.peers");
        let parameters = QueryParameters {
            consistency: Default::default(),
            serial_consistency: Some(SerialConsistency::LocalSerial),
            timestamp: None,
            page_size: None,
            paging_state: None,
            values: Cow::Owned(SerializedValues::new()),
        };
        let query = Query {
            contents: contents.clone(),
            parameters,
        };

        {
            let mut buf = Vec::new();
            query.serialize(&mut buf).unwrap();

            // Sanity check: query deserializes to the equivalent.
            let query_deserialized = Query::deserialize(&mut &buf[..]).unwrap();
            assert_eq!(&query_deserialized.contents, &query.contents);
            assert_eq!(&query_deserialized.parameters, &query.parameters);

            // Now modify flags by adding an unknown one.
            // Find flags in buffer:
            let mut buf_ptr = buf.as_slice();
            let serialised_contents = types::read_long_string(&mut buf_ptr).unwrap();
            assert_eq!(serialised_contents, contents);

            // Now buf_ptr points at consistency.
            let consistency = types::read_consistency(&mut buf_ptr).unwrap();
            assert_eq!(
                consistency,
                LegacyConsistency::Regular(Consistency::default())
            );

            // Now buf_ptr points at flags, but it is immutable. Get mutable reference into the buffer.
            let flags_idx = buf.len() - buf_ptr.len();
            let flags_mut = &mut buf[flags_idx];

            // This assumes that the following flag is unknown, which is true at the time of writing this test.
            *flags_mut |= 0x80;

            // Unknown flag should lead to frame rejection, as unknown flags can be new protocol extensions
            // leading to different semantics.
            let _parse_error = Query::deserialize(&mut &buf[..]).unwrap_err();
        }

        // Batch
        let statements = vec![BatchStatement::Query {
            text: query.contents,
        }];
        let batch = Batch {
            statements: Cow::Owned(statements),
            batch_type: BatchType::Logged,
            consistency: Consistency::EachQuorum,
            serial_consistency: None,
            timestamp: None,

            values: vec![query.parameters.values.deref().clone()],
        };
        {
            let mut buf = Vec::new();
            batch.serialize(&mut buf).unwrap();

            // Sanity check: batch deserializes to the equivalent.
            let batch_deserialized = Batch::deserialize(&mut &buf[..]).unwrap();
            assert_eq!(batch, batch_deserialized);

            // Now modify flags by adding an unknown one.
            // There are no timestamp nor serial consistency, so flags are the last byte in the buf.
            let buf_len = buf.len();
            let flags_mut = &mut buf[buf_len - 1];
            // This assumes that the following flag is unknown, which is true at the time of writing this test.
            *flags_mut |= 0x80;

            // Unknown flag should lead to frame rejection, as unknown flags can be new protocol extensions
            // leading to different semantics.
            let _parse_error = Batch::deserialize(&mut &buf[..]).unwrap_err();
        }
    }
}

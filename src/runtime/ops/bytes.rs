//! Runtime-level byte string and buffer opcode adapters.

use crate::runtime::RuntimeStack;
use crate::{
    byte_sequence_bytes, byte_sequence_len, concat_byte_sequences, slice_byte_sequence, StackValue,
};

pub fn cat<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let right = runtime.pop_value();
    let left = runtime.pop_value();
    match concat_byte_sequences(left, right) {
        Some(value) => runtime.push_value(value),
        None => runtime.fault("CAT: operands must be ByteString or Buffer"),
    }
}

pub fn substr<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let count = runtime.pop_i64();
    let index = runtime.pop_i64();
    let value = runtime.pop_value();

    let Some(len) = byte_sequence_len(&value) else {
        runtime.fault("SUBSTR: not a ByteString or Buffer");
        return;
    };

    if index < 0 || count < 0 {
        runtime.fault("SUBSTR: negative index or count");
        return;
    }

    #[allow(clippy::cast_sign_loss)]
    let (index, count) = (index as usize, count as usize);
    let Some(end) = index.checked_add(count) else {
        runtime.fault("SUBSTR: range out of bounds");
        return;
    };
    if end > len {
        runtime.fault("SUBSTR: range out of bounds");
        return;
    }

    match slice_byte_sequence(value, index, count) {
        Some(value) => runtime.push_value(value),
        None => runtime.fault("SUBSTR: range out of bounds"),
    }
}

pub fn left<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let count = runtime.pop_i64();
    let value = runtime.pop_value();

    let Some(len) = byte_sequence_len(&value) else {
        runtime.fault("LEFT: not a ByteString or Buffer");
        return;
    };

    if count < 0 {
        runtime.fault("LEFT: negative count");
        return;
    }

    #[allow(clippy::cast_sign_loss)]
    let count = count as usize;
    if count > len {
        runtime.fault("LEFT: count exceeds length");
        return;
    }

    match slice_byte_sequence(value, 0, count) {
        Some(value) => runtime.push_value(value),
        None => runtime.fault("LEFT: count exceeds length"),
    }
}

pub fn right<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let count = runtime.pop_i64();
    let value = runtime.pop_value();

    let Some(len) = byte_sequence_len(&value) else {
        runtime.fault("RIGHT: not a ByteString or Buffer");
        return;
    };

    if count < 0 {
        runtime.fault("RIGHT: negative count");
        return;
    }

    #[allow(clippy::cast_sign_loss)]
    let count = count as usize;
    if count > len {
        runtime.fault("RIGHT: count exceeds length");
        return;
    }

    match slice_byte_sequence(value, len - count, count) {
        Some(value) => runtime.push_value(value),
        None => runtime.fault("RIGHT: count exceeds length"),
    }
}

pub fn memcpy<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let count = runtime.pop_i64();
    let source_index = runtime.pop_i64();
    let source = runtime.pop_value();
    let destination_index = runtime.pop_i64();

    let Some(source_bytes) = byte_sequence_bytes(&source).map(<[u8]>::to_vec) else {
        runtime.fault("MEMCPY: source is not a ByteString or Buffer");
        return;
    };

    if count < 0 || source_index < 0 || destination_index < 0 {
        runtime.fault("MEMCPY: negative argument");
        return;
    }

    #[allow(clippy::cast_sign_loss)]
    let (count, source_index, destination_index) = (
        count as usize,
        source_index as usize,
        destination_index as usize,
    );

    let Some(source_end) = source_index.checked_add(count) else {
        runtime.fault("MEMCPY: source range out of bounds");
        return;
    };
    if source_end > source_bytes.len() {
        runtime.fault("MEMCPY: source range out of bounds");
        return;
    }

    let source_slice = source_bytes[source_index..source_end].to_vec();
    let result = match runtime.top_value_mut() {
        Some(StackValue::Buffer(buffer)) => {
            let Some(destination_end) = destination_index.checked_add(count) else {
                return runtime.fault("MEMCPY: destination range out of bounds");
            };
            if destination_end > buffer.len() {
                return runtime.fault("MEMCPY: destination range out of bounds");
            }
            buffer[destination_index..destination_end].copy_from_slice(&source_slice);
            Ok(())
        }
        _ => Err("MEMCPY: destination is not a Buffer"),
    };

    if let Err(message) = result {
        runtime.fault(message);
    }
}

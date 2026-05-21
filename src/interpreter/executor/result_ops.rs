use super::super::runtime_types::StackValue;
use alloc::vec::Vec;

#[inline]
pub(super) fn trim_halt_stack_for_result_limit(
    stack: &mut Vec<StackValue>,
    result_stack_limit: Option<usize>,
) {
    let Some(keep) = result_stack_limit else {
        return;
    };

    // The guest uses a per-execution bump allocator. Forgetting discarded
    // return-stack debris avoids recursive Drop on deep historical compound
    // values; the whole arena is reset before the next execution.
    let old_stack = core::mem::take(stack);
    if keep == 0 {
        core::mem::forget(old_stack);
    } else if old_stack.len() > keep {
        let start = old_stack.len() - keep;
        let mut kept = Vec::with_capacity(keep);
        for item in old_stack.iter().skip(start) {
            kept.push(item.clone());
        }
        *stack = kept;
        core::mem::forget(old_stack);
    } else {
        *stack = old_stack;
    }
}

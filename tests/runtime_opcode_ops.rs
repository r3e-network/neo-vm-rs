use neo_vm_rs::{
    runtime::{ops, RuntimeStack},
    StackValue, VmContext, VmState,
};

#[derive(Default)]
struct TestRuntime {
    stack: Vec<StackValue>,
    fault: Option<String>,
}

impl RuntimeStack for TestRuntime {
    fn pop_value(&mut self) -> StackValue {
        self.stack.pop().expect("test stack underflow")
    }

    fn push_value(&mut self, value: StackValue) {
        self.stack.push(value);
    }

    fn top_value_mut(&mut self) -> Option<&mut StackValue> {
        self.stack.last_mut()
    }

    fn stack_values(&self) -> &[StackValue] {
        &self.stack
    }

    fn stack_values_mut(&mut self) -> &mut Vec<StackValue> {
        &mut self.stack
    }

    fn fault(&mut self, message: &str) {
        self.fault = Some(message.to_string());
    }
}

#[test]
fn shared_vm_context_owns_common_state_slots_and_results() {
    let mut ctx = VmContext::from_stack(vec![StackValue::Integer(10), StackValue::Integer(20)]);

    ctx.init_slot(1, 2);
    ctx.load_arg(0);
    ctx.store_local(0);
    ctx.load_local(0);

    let result = ctx.into_execution_result(99);

    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.fee_consumed_pico, 99);
    assert_eq!(result.stack, vec![StackValue::Integer(10)]);
}

#[test]
fn shared_stack_and_byte_opcode_apis_do_not_require_context_methods() {
    let mut ctx = VmContext::from_stack(vec![]);

    ctx.push_value(StackValue::Integer(1));
    ctx.push_value(StackValue::Integer(2));
    ops::stack::dup(&mut ctx);
    ops::stack::swap(&mut ctx);
    ops::stack::depth(&mut ctx);

    assert_eq!(ctx.pop_value(), StackValue::Integer(3));
    assert_eq!(ctx.pop_value(), StackValue::Integer(2));
    assert_eq!(ctx.pop_value(), StackValue::Integer(2));
    assert_eq!(ctx.pop_value(), StackValue::Integer(1));

    ctx.push_value(StackValue::ByteString(b"neo".to_vec()));
    ctx.push_value(StackValue::Buffer(b"n4".to_vec()));
    ops::bytes::cat(&mut ctx);

    assert_eq!(ctx.pop_value(), StackValue::ByteString(b"neon4".to_vec()));
}

#[test]
fn runtime_arithmetic_ops_own_stack_pop_push_shape() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Integer(10), StackValue::Integer(3)],
        fault: None,
    };

    ops::arithmetic::div(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Integer(3)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_collection_ops_preserve_in_place_mutation_shape() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Array(vec![]), StackValue::Integer(1)],
        fault: None,
    };

    ops::collections::append(&mut rt);
    rt.stack.push(StackValue::Integer(2));
    ops::collections::append(&mut rt);
    ops::collections::size(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Integer(2)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_conversion_and_comparison_ops_share_vm_rules() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Boolean(true)],
        fault: None,
    };

    ops::conversion::convert_to(&mut rt, 0x21);
    ops::comparison::is_null(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Boolean(false)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_ops_report_faults_through_adapter() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Integer(10), StackValue::Integer(0)],
        fault: None,
    };

    ops::arithmetic::div(&mut rt);

    assert_eq!(rt.stack, Vec::<StackValue>::new());
    assert_eq!(rt.fault.as_deref(), Some("division by zero for DIV"));
}

use neo_vm_rs::{semantics::runtime, StackValue};

#[derive(Default)]
struct TestRuntime {
    stack: Vec<StackValue>,
    fault: Option<String>,
}

impl runtime::RuntimeStack for TestRuntime {
    fn pop_value(&mut self) -> StackValue {
        self.stack.pop().expect("test stack underflow")
    }

    fn push_value(&mut self, value: StackValue) {
        self.stack.push(value);
    }

    fn top_value_mut(&mut self) -> Option<&mut StackValue> {
        self.stack.last_mut()
    }

    fn fault(&mut self, message: &str) {
        self.fault = Some(message.to_string());
    }
}

#[test]
fn runtime_arithmetic_ops_own_stack_pop_push_shape() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Integer(10), StackValue::Integer(3)],
        fault: None,
    };

    runtime::arithmetic::div(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Integer(3)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_collection_ops_preserve_in_place_mutation_shape() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Array(vec![]), StackValue::Integer(1)],
        fault: None,
    };

    runtime::collections::append(&mut rt);
    rt.stack.push(StackValue::Integer(2));
    runtime::collections::append(&mut rt);
    runtime::collections::size(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Integer(2)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_conversion_and_comparison_ops_share_vm_rules() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Boolean(true)],
        fault: None,
    };

    runtime::conversion::convert_to(&mut rt, 0x21);
    runtime::comparison::is_null(&mut rt);

    assert_eq!(rt.stack, vec![StackValue::Boolean(false)]);
    assert_eq!(rt.fault, None);
}

#[test]
fn runtime_ops_report_faults_through_adapter() {
    let mut rt = TestRuntime {
        stack: vec![StackValue::Integer(10), StackValue::Integer(0)],
        fault: None,
    };

    runtime::arithmetic::div(&mut rt);

    assert_eq!(rt.stack, Vec::<StackValue>::new());
    assert_eq!(rt.fault.as_deref(), Some("DIV: division by zero"));
}

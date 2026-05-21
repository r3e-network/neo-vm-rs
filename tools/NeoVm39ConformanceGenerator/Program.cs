using System.Numerics;
using System.Text;
using System.Text.Json;
using Neo.VM;
using Neo.VM.Types;
using ArrayItem = Neo.VM.Types.Array;
using StructItem = Neo.VM.Types.Struct;

internal sealed record Case(string Name, string Description, byte[] Script);

internal static class Program
{
    private static byte[] Bytes(params byte[] bytes) => bytes;

    private static string Hex(byte[] bytes) => Convert.ToHexString(bytes).ToLowerInvariant();

    private static object ToJsonStackValue(StackItem item)
    {
        return item.Type switch
        {
            StackItemType.Any => "Null",
            StackItemType.Boolean => new Dictionary<string, object> { ["Boolean"] = item.GetBoolean() },
            StackItemType.Integer => ToJsonInteger(item.GetInteger()),
            StackItemType.ByteString => new Dictionary<string, object> { ["ByteString"] = BytesForJson(item) },
            StackItemType.Buffer => new Dictionary<string, object> { ["Buffer"] = BytesForJson(item) },
            StackItemType.Array => new Dictionary<string, object> { ["Array"] = ((ArrayItem)item).Select(ToJsonStackValue).ToArray() },
            StackItemType.Struct => new Dictionary<string, object> { ["Struct"] = ((StructItem)item).Select(ToJsonStackValue).ToArray() },
            _ => throw new NotSupportedException($"Unsupported stack item type for fixture: {item.Type}")
        };
    }

    private static object ToJsonInteger(BigInteger value)
    {
        if (value >= long.MinValue && value <= long.MaxValue)
        {
            return new Dictionary<string, object> { ["Integer"] = (long)value };
        }
        return new Dictionary<string, object> { ["BigInteger"] = value.ToByteArray().Select(b => (int)b).ToArray() };
    }

    private static int[] BytesForJson(StackItem item) =>
        item.GetSpan().ToArray().Select(b => (int)b).ToArray();

    private static object Run(Case testCase)
    {
        using var engine = new ExecutionEngine(new JumpTable());
        engine.LoadScript(new Script(testCase.Script));
        var state = engine.Execute();

        return new Dictionary<string, object?>
        {
            ["name"] = testCase.Name,
            ["description"] = testCase.Description,
            ["script_hex"] = Hex(testCase.Script),
            ["expected_state"] = state.HasFlag(VMState.FAULT) ? "Fault" : "Halt",
            ["expected_stack"] = engine.ResultStack.Select(ToJsonStackValue).ToArray(),
        };
    }

    public static void Main(string[] args)
    {
        var cases = new[]
        {
            new Case("add_small_integers", "PUSH2 PUSH3 ADD RET", Bytes(0x12, 0x13, 0x9e, 0x40)),
            new Case("pushint16_little_endian", "PUSHINT16 0x1234 RET", Bytes(0x01, 0x34, 0x12, 0x40)),
            new Case("negate_then_abs", "PUSHINT8 -7 ABS RET", Bytes(0x00, 0xf9, 0x9a, 0x40)),
            new Case("integer_division_truncates", "PUSH10 PUSH3 DIV RET", Bytes(0x1a, 0x13, 0xa1, 0x40)),
            new Case("integer_modulo", "PUSH10 PUSH3 MOD RET", Bytes(0x1a, 0x13, 0xa2, 0x40)),
            new Case("boolean_and_false", "PUSHT PUSHF BOOLAND RET", Bytes(0x08, 0x09, 0xab, 0x40)),
            new Case("boolean_or_true", "PUSHT PUSHF BOOLOR RET", Bytes(0x08, 0x09, 0xac, 0x40)),
            new Case("numeric_less_than", "PUSH2 PUSH3 LT RET", Bytes(0x12, 0x13, 0xb5, 0x40)),
            new Case("numeric_within", "PUSH5 PUSH0 PUSH10 WITHIN RET", Bytes(0x15, 0x10, 0x1a, 0xbb, 0x40)),
            new Case("bitwise_and", "PUSHINT8 0x0f PUSHINT8 0x33 AND RET", Bytes(0x00, 0x0f, 0x00, 0x33, 0x91, 0x40)),
            new Case("bytes_cat", "PUSHDATA1 0102 PUSHDATA1 0304 CAT RET", Bytes(0x0c, 0x02, 0x01, 0x02, 0x0c, 0x02, 0x03, 0x04, 0x8b, 0x40)),
            new Case("bytes_left", "PUSHDATA1 abcdef PUSH3 LEFT RET", Bytes(0x0c, 0x06, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x13, 0x8d, 0x40)),
            new Case("bytes_right", "PUSHDATA1 abcdef PUSH2 RIGHT RET", Bytes(0x0c, 0x06, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x12, 0x8e, 0x40)),
            new Case("bytes_substr", "PUSHDATA1 abcdef PUSH2 PUSH3 SUBSTR RET", Bytes(0x0c, 0x06, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x12, 0x13, 0x8c, 0x40)),
            new Case("pack_array_order", "PUSH1 PUSH2 PUSH2 PACK RET", Bytes(0x11, 0x12, 0x12, 0xc0, 0x40)),
            new Case("newarray_nulls", "PUSH3 NEWARRAY RET", Bytes(0x13, 0xc3, 0x40)),
            new Case("array_pickitem", "PUSH10 PUSH20 PUSH2 PACK PUSH1 PICKITEM RET", Bytes(0x1a, 0x00, 0x14, 0x12, 0xc0, 0x11, 0xce, 0x40)),
            new Case("local_slot_roundtrip", "INITSLOT STLOC0 LDLOC0 RET", Bytes(0x57, 0x01, 0x00, 0x15, 0x70, 0x68, 0x40)),
            new Case("static_slot_roundtrip", "INITSSLOT STSFLD0 LDSFLD0 RET", Bytes(0x56, 0x01, 0x16, 0x60, 0x58, 0x40)),
            new Case("jmpif_true", "PUSHT JMPIF skips PUSH1 and returns PUSH2", Bytes(0x08, 0x24, 0x04, 0x11, 0x40, 0x12, 0x40)),
        };

        var doc = new Dictionary<string, object?>
        {
            ["neo_node_tag"] = "v3.9.2",
            ["neo_package_version"] = "Neo 3.9.1",
            ["neo_vm_package_version"] = "Neo.VM 3.9.0",
            ["source"] = "Generated with NuGet Neo.VM 3.9.0. neo-node v3.9.2 depends on Neo 3.9.1, which depends on Neo.VM 3.9.0.",
            ["vectors"] = cases.Select(Run).ToArray(),
        };

        var json = JsonSerializer
            .Serialize(doc, new JsonSerializerOptions { WriteIndented = true })
            .Replace("\r\n", "\n");
        if (args.Length > 0)
        {
            File.WriteAllText(args[0], json + "\n", new UTF8Encoding(false));
            return;
        }

        Console.WriteLine(json);
    }
}

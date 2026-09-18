using Neo.VM;

// Bytes for the NeoVM 3.9 opcodes exercised here, aligned with
// external/neo-vm-rs/src/interpreter/opcodes.rs (OpCode::<name>.byte()).
internal static class Op
{
    internal const byte PUSHINT8 = 0x00;
    internal const byte PUSHDATA1 = 0x0c;
    internal const byte RET = 0x40;

    internal const byte NEGATE = 0x99;
    internal const byte ABS = 0x9a;
    internal const byte SUB = 0x9f;
    internal const byte ADD = 0x9e;
    internal const byte DIV = 0xa1;
    internal const byte MOD = 0xa2;

    internal const byte CAT = 0x8b;
    internal const byte SUBSTR = 0x8c;
    internal const byte LEFT = 0x8d;
    internal const byte RIGHT = 0x8e;
}

internal static class RandomOpsCases
{
    private static readonly string[] Families =
    {
        "negate", "abs", "sub", "div", "mod", "cat",
        "left", "right", "substr", "fault_empty_add", "fault_empty_abs", "fault_underflow_sub",
    };

    internal static Case[] Generate(uint seed, int count)
    {
        if (seed == 0 || count < 1 || count > 4096)
            throw new ArgumentOutOfRangeException(nameof(count), "Nonzero seed and 1..4096 cases required.");
        var state = seed;
        uint Next()
        {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            return state;
        }

        static byte[] Data(uint next) => Enumerable.Range(0, (int)(next % 9)).Select(_ => (byte)next).ToArray();

        var cases = new Case[count];
        for (var i = 0; i < count; i++)
        {
            var family = Families[i % Families.Length];
            var script = new List<byte>();
            var description = family;
            switch (family)
            {
                case "negate":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.NEGATE, Op.RET]);
                    description = "PUSHINT8 x NEGATE RET";
                    break;
                case "abs":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.ABS, Op.RET]);
                    description = "PUSHINT8 x ABS RET";
                    break;
                case "sub":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.PUSHINT8, (byte)(Next() & 255), Op.SUB, Op.RET]);
                    description = "PUSHINT8 a PUSHINT8 b SUB RET";
                    break;
                case "div":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.PUSHINT8, (byte)(Next() & 255), Op.DIV, Op.RET]);
                    description = "PUSHINT8 a PUSHINT8 b DIV RET (FAULT if b==0)";
                    break;
                case "mod":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.PUSHINT8, (byte)(Next() & 255), Op.MOD, Op.RET]);
                    description = "PUSHINT8 a PUSHINT8 b MOD RET (FAULT if b==0)";
                    break;
                case "cat":
                {
                    var d1 = Data(Next());
                    var d2 = Data(Next());
                    script.AddRange([Op.PUSHDATA1, (byte)d1.Length, .. d1, Op.PUSHDATA1, (byte)d2.Length, .. d2, Op.CAT, Op.RET]);
                    description = "PUSHDATA1 CAT RET";
                    break;
                }
                case "left":
                {
                    var data = Data(Next());
                    var n = (int)(Next() % 4) switch { 0 => -1, 1 => 0, 2 => data.Length, _ => data.Length + 1 };
                    script.AddRange([Op.PUSHDATA1, (byte)data.Length, .. data, Op.PUSHINT8, (byte)n, Op.LEFT, Op.RET]);
                    description = "PUSHDATA1 PUSHINT8 n LEFT RET (FAULT if n out of [0,len])";
                    break;
                }
                case "right":
                {
                    var data = Data(Next());
                    var n = (int)(Next() % 4) switch { 0 => -1, 1 => 0, 2 => data.Length, _ => data.Length + 1 };
                    script.AddRange([Op.PUSHDATA1, (byte)data.Length, .. data, Op.PUSHINT8, (byte)n, Op.RIGHT, Op.RET]);
                    description = "PUSHDATA1 PUSHINT8 n RIGHT RET (FAULT if n out of [0,len])";
                    break;
                }
                case "substr":
                {
                    var data = Data(Next());
                    var index = Next() % (data.Length + 1);
                    var blen = Next() % 8;
                    script.AddRange([Op.PUSHDATA1, (byte)data.Length, .. data, Op.PUSHINT8, (byte)index, Op.PUSHINT8, (byte)blen, Op.SUBSTR, Op.RET]);
                    description = "PUSHDATA1 PUSHINT8 i PUSHINT8 n SUBSTR RET (FAULT if span out of range)";
                    break;
                }
                case "fault_empty_add":
                    script.AddRange([Op.ADD, Op.RET]);
                    description = "ADD RET (FAULT: empty operand stack)";
                    break;
                case "fault_empty_abs":
                    script.AddRange([Op.ABS, Op.RET]);
                    description = "ABS RET (FAULT: empty operand stack)";
                    break;
                case "fault_underflow_sub":
                    script.AddRange([Op.PUSHINT8, (byte)(Next() & 255), Op.SUB, Op.RET]);
                    description = "PUSHINT8 a SUB RET (FAULT: SUB needs two operands)";
                    break;
            }

            cases[i] = new Case($"ops_{family}_{seed}_{i}", $"{description} [seed={seed} index={i}]", script.ToArray());
        }
        return cases;
    }
}
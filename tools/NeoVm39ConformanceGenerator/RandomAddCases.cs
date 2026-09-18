using Neo.VM;

internal static class RandomAddCases
{
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

        var cases = new Case[count];
        for (var i = 0; i < count; i++)
        {
            var script = new List<byte>();
            var terms = 2 + (int)(Next() % 31);
            for (var term = 0; term < terms; term++)
            {
                script.Add((byte)OpCode.PUSHINT8);
                script.Add((byte)(Next() & 255));
                if (term > 0)
                    script.Add((byte)OpCode.ADD);
            }
            script.Add((byte)OpCode.RET);
            cases[i] = new Case($"random_add_{seed}_{i}", $"xorshift32 seed={seed}, index={i}, terms={terms}", script.ToArray());
        }
        return cases;
    }
}

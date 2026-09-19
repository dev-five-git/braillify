using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

internal static class Btl
{
    private const string Dll = "BrailleTransMain.dll";
    private const int BufferSize = 1 << 20;
    private const int CellCapacity = 1 << 16;

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
    private static extern int BrailleTransStringToBrailleWithOption(
        string input, byte[] output, int outputSize, int gradeMode, int koreanRuleVersion);

    [DllImport(Dll, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
    private static extern int BrailleTransBrailleToUnicodeBraille(
        byte[] braille, StringBuilder output, int outputSize);

    private static int Main(string[] args)
    {
        if (args.Length < 4)
        {
            Console.Error.WriteLine(
                "usage: btl.exe <in.txt> <out.txt> <gradeMode> <ruleVersion> [startIndex]");
            return 2;
        }

        var gradeMode = int.Parse(args[2]);
        var ruleVersion = int.Parse(args[3]);
        var startIndex = args.Length > 4 ? int.Parse(args[4]) : 0;
        var encoding = new UTF8Encoding(false);
        var buffer = new byte[BufferSize];
        var cells = new StringBuilder(CellCapacity);
        var watch = Stopwatch.StartNew();
        var count = 0;

        using (var reader = new StreamReader(args[0], encoding))
        using (var writer = new StreamWriter(args[1], startIndex > 0, encoding))
        {
            writer.AutoFlush = true;
            for (var i = 0; i < startIndex; i++)
            {
                if (reader.ReadLine() == null) break;
            }
            string line;
            while ((line = reader.ReadLine()) != null)
            {
                var input = line.Replace("\\n", "\n").Replace("\\\\", "\\");
                Array.Clear(buffer, 0, buffer.Length);
                var produced = BrailleTransStringToBrailleWithOption(
                    input, buffer, buffer.Length, gradeMode, ruleVersion);
                var result = string.Empty;
                if (produced > 0)
                {
                    // 이 DLL 은 유니코드 출력 버퍼를 NUL 로 끝맺지 않는다. 미리 NUL 로
                    // 채워 두어야 마샬러가 DLL 이 쓴 마지막 셀에서 멈추고, 버퍼에 남은
                    // 이전 호출의 잔여 셀을 결과로 잘못 읽지 않는다.
                    cells.Length = 0;
                    cells.Append('\0', CellCapacity);
                    BrailleTransBrailleToUnicodeBraille(buffer, cells, CellCapacity);
                    result = cells.ToString();
                }
                writer.WriteLine(produced + "\t" + result.Replace("\n", "\\n"));
                count++;
            }
        }

        Console.Error.WriteLine(
            "translated=" + count +
            " elapsed=" + watch.Elapsed.TotalSeconds.ToString("F1") + "s");
        return 0;
    }
}

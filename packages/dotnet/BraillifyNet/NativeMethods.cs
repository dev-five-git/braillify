#if NET7_0_OR_GREATER
// .NET 7+: LibraryImport (source generator) 사용 - 최고 성능

namespace Braillify;

using System;
using System.Runtime.InteropServices;

internal static partial class NativeMethods
{
    private const string LibraryName = "braillify_native";

    [LibraryImport(LibraryName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint braillify_encode(string text, out nuint outLen);

    [LibraryImport(LibraryName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint braillify_encode_to_unicode(string text);

    [LibraryImport(LibraryName, StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint braillify_encode_to_braille_font(string text);

    [LibraryImport(LibraryName)]
    internal static partial IntPtr braillify_encode_in_context(IntPtr text, IntPtr context, out UIntPtr outLen);

    [LibraryImport(LibraryName)]
    internal static partial IntPtr braillify_encode_to_unicode_in_context(IntPtr text, IntPtr context);

    [LibraryImport(LibraryName)]
    internal static partial IntPtr braillify_encode_to_braille_font_in_context(IntPtr text, IntPtr context);

    [LibraryImport(LibraryName)]
    internal static partial nint braillify_get_last_error();

    [LibraryImport(LibraryName)]
    internal static partial void braillify_free_string(nint ptr);

    [LibraryImport(LibraryName)]
    internal static partial void braillify_free_bytes(nint ptr, nuint len);
}

#elif NET5_0_OR_GREATER
// .NET 5-6: DllImport with nint/nuint

namespace Braillify;

using System;
using System.Runtime.InteropServices;

internal static class NativeMethods
{
    private const string LibraryName = "braillify_native";

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern nint braillify_encode(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text,
        out nuint outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern nint braillify_encode_to_unicode(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern nint braillify_encode_to_braille_font(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_in_context(IntPtr text, IntPtr context, out UIntPtr outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_unicode_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_braille_font_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern nint braillify_get_last_error();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_string(nint ptr);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_bytes(nint ptr, nuint len);
}

#elif NETCOREAPP3_0_OR_GREATER
// .NET Core 3.x: DllImport with IntPtr/UIntPtr and LPUTF8Str

namespace Braillify;

using System;
using System.Runtime.InteropServices;

internal static class NativeMethods
{
    private const string LibraryName = "braillify_native";

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr braillify_encode(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text,
        out UIntPtr outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr braillify_encode_to_unicode(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    internal static extern IntPtr braillify_encode_to_braille_font(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_in_context(IntPtr text, IntPtr context, out UIntPtr outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_unicode_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_braille_font_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_get_last_error();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_string(IntPtr ptr);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_bytes(IntPtr ptr, UIntPtr len);
}

#else
// .NET Standard 2.0: 수동 UTF-8 마샬링이 필요한 DllImport (IntPtr/UIntPtr)

namespace Braillify;

using System;
using System.Runtime.InteropServices;

internal static class NativeMethods
{
    private const string LibraryName = "braillify_native";

    // .NET Standard 2.0에서는 byte[] 포인터를 직접 전달
    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode(
        IntPtr text,
        out UIntPtr outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_unicode(IntPtr text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_braille_font(IntPtr text);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_in_context(IntPtr text, IntPtr context, out UIntPtr outLen);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_unicode_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_encode_to_braille_font_in_context(IntPtr text, IntPtr context);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr braillify_get_last_error();

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_string(IntPtr ptr);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void braillify_free_bytes(IntPtr ptr, UIntPtr len);
}

#endif

# Types real key events (SendInput, virtual-key codes) into Notepad, then reads the text back
# via Ctrl+A / Ctrl+C and compares with -Expected.
# Tokens in -Type: {BS} {ENTER} {TAB} {LEFT} {RIGHT} {UP} {DOWN} {CTRL} (tap) {CLICK} (click at centre of window).
# Chords (uppercase): {ALT+Z} = modifiers down, key tap, modifiers up. {CTRL+SHIFT} = modifier-only chord (press and release, no key).
# Keep this file ASCII-only (Windows PowerShell 5.1 reads BOM-less files as ANSI).
param(
    [string]$Type = 'Vieejt Nam, hoaf dduowcj.',
    [string]$Expected = ('Vi' + [char]0x1EC7 + 't Nam, h' + [char]0x00F2 + 'a ' + [char]0x0111 + [char]0x01B0 + [char]0x1EE3 + 'c.'),
    [switch]$KeepOpen
)
$ErrorActionPreference = 'Stop'
Add-Type -Language CSharp @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading;
public static class Native {
    [StructLayout(LayoutKind.Sequential)] struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Explicit)] struct INPUT {
        [FieldOffset(0)] public uint type;
        [FieldOffset(8)] public KEYBDINPUT ki;
        [FieldOffset(8)] public MOUSEINPUT mi;
    }
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] static extern ushort MapVirtualKey(uint code, uint map);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, IntPtr pid);
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint a, uint b, bool attach);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    // Windows refuses SetForegroundWindow from a background process; attaching to the foreground thread's input lifts that.
    public static void Focus(IntPtr hwnd) {
        uint fg = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero), me = GetCurrentThreadId();
        AttachThreadInput(me, fg, true);
        SetForegroundWindow(hwnd);
        AttachThreadInput(me, fg, false);
    }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT r);
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);

    static INPUT Key(ushort vk, bool up) {
        var i = new INPUT { type = 1 };
        i.ki.wVk = vk; i.ki.wScan = MapVirtualKey(vk, 0); i.ki.dwFlags = up ? 2u : 0u;
        return i;
    }
    static void Send(List<INPUT> seq) {
        var arr = seq.ToArray();
        if (SendInput((uint)arr.Length, arr, Marshal.SizeOf(typeof(INPUT))) != arr.Length)
            throw new Exception("SendInput failed: " + Marshal.GetLastWin32Error());
    }
    // Presses vk with optional Shift/Ctrl held; each event is a real key down/up pair.
    public static void Tap(ushort vk, bool shift, bool ctrl) {
        var seq = new List<INPUT>();
        if (shift) seq.Add(Key(0x10, false));
        if (ctrl) seq.Add(Key(0x11, false));
        seq.Add(Key(vk, false)); seq.Add(Key(vk, true));
        if (ctrl) seq.Add(Key(0x11, true));
        if (shift) seq.Add(Key(0x10, true));
        Send(seq);
        Thread.Sleep(40);
    }
    public static void Click(int x, int y) {
        SetCursorPos(x, y);
        var d = new INPUT { type = 0 }; d.mi.dwFlags = 0x02;
        var u = new INPUT { type = 0 }; u.mi.dwFlags = 0x04;
        Send(new List<INPUT> { d, u });
        Thread.Sleep(150);
    }
    // Chord: modifiers down, optional key tap, modifiers up. vk 0 = modifier-only (press and release, no key between).
    public static void Chord(ushort[] mods, ushort vk) {
        var seq = new List<INPUT>();
        foreach (var m in mods) seq.Add(Key(m, false));
        if (vk != 0) { seq.Add(Key(vk, false)); seq.Add(Key(vk, true)); }
        for (int i = mods.Length - 1; i >= 0; i--) seq.Add(Key(mods[i], true));
        Send(seq);
        Thread.Sleep(150);
    }
}
'@

$vkMap = @{ 'BS' = 0x08; 'ENTER' = 0x0D; 'TAB' = 0x09; 'LEFT' = 0x25; 'UP' = 0x26; 'RIGHT' = 0x27; 'DOWN' = 0x28; 'CTRL' = 0x11 }
$modVk = @{ 'CTRL' = 0x11; 'ALT' = 0x12; 'SHIFT' = 0x10; 'WIN' = 0x5B }
# {ALT+Z} = chord with a key; {CTRL+SHIFT} = modifier-only chord (no key between press and release).
$tokens = [regex]::Matches($Type, '\{(BS|ENTER|TAB|LEFT|UP|RIGHT|DOWN|CTRL|CLICK|(?:(?:CTRL|ALT|SHIFT|WIN)\+)+(?:[A-Z0-9]|F(?:1[0-2]|[1-9])|SPACE)|(?:CTRL|ALT|SHIFT|WIN)(?:\+(?:CTRL|ALT|SHIFT|WIN))*)\}|.')

# Win11 Notepad is packaged and single-instance: a new process may not appear, so open a uniquely named
# file and match the window by its title. Only that window receives the keys below.
$name = "evkey-smoke-$([guid]::NewGuid().ToString('N').Substring(0, 8)).txt"
$path = Join-Path $env:TEMP $name
New-Item -ItemType File -Path $path -Force | Out-Null
Start-Process notepad.exe -ArgumentList $path | Out-Null
$proc = $null; $hwnd = [IntPtr]::Zero
for ($i = 0; $i -lt 60 -and $hwnd -eq [IntPtr]::Zero; $i++) {
    Start-Sleep -Milliseconds 250
    $cand = Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 -and $_.MainWindowTitle -like "*$name*" } | Select-Object -First 1
    if ($cand) { $proc = $cand; $hwnd = $cand.MainWindowHandle }
}
if ($hwnd -eq [IntPtr]::Zero) { throw "Notepad window for $name not found" }
[Native]::Focus($hwnd)
Start-Sleep -Milliseconds 500
# Abort before any key goes out: keys sent to a non-target window would edit someone else's text.
if ([Native]::GetForegroundWindow() -ne $hwnd) { throw 'Notepad is not the foreground window; refusing to send keys' }

[Native]::Tap(0x41, $false, $true)  # Ctrl+A select all
[Native]::Tap(0x2E, $false, $false) # Delete
Start-Sleep -Milliseconds 200

foreach ($m in $tokens) {
    $t = $m.Value
    if ($t -eq '{CLICK}') {
        $r = New-Object Native+RECT
        [void][Native]::GetWindowRect($hwnd, [ref]$r)
        [Native]::Click([int](($r.Left + $r.Right) / 2), [int](($r.Top + $r.Bottom) / 2))
        continue
    }
    if ($t -match '^\{((?:CTRL|ALT|SHIFT|WIN)(?:\+(?:CTRL|ALT|SHIFT|WIN|SPACE|F(?:1[0-2]|[1-9])|[A-Z0-9]))*)\}$') {
        $parts = $Matches[1].Split('+')
        $last = $parts[-1]
        if ($modVk.ContainsKey($last)) { $mods = $parts; $vk = [uint16]0 }
        else {
            $mods = $parts[0..($parts.Length - 2)]
            $vk = if ($last -eq 'SPACE') { [uint16]0x20 } elseif ($last -match '^F(\d+)$') { [uint16](0x6F + [int]$Matches[1]) } else { [uint16][char]$last }
        }
        [Native]::Chord([uint16[]]@($mods | ForEach-Object { $modVk[$_] }), $vk)
        continue
    }
    if ($t -match '^\{(\w+)\}$') { [Native]::Tap([uint16]$vkMap[$Matches[1]], $false, $false); continue }
    $c = [int][char]$t
    if ($c -ge 0x61 -and $c -le 0x7A) { [Native]::Tap([uint16]($c - 0x20), $false, $false) }
    elseif ($c -ge 0x41 -and $c -le 0x5A) { [Native]::Tap([uint16]$c, $true, $false) }
    elseif ($c -ge 0x30 -and $c -le 0x39) { [Native]::Tap([uint16]$c, $false, $false) }
    elseif ($c -eq 0x20) { [Native]::Tap(0x20, $false, $false) }
    elseif ($c -eq 0x2C) { [Native]::Tap(0xBC, $false, $false) }
    elseif ($c -eq 0x2E) { [Native]::Tap(0xBE, $false, $false) }
    else { throw ("Unsupported character in -Type: '{0}' (U+{1:X4})" -f $t, $c) }
}
Start-Sleep -Milliseconds 300

Set-Clipboard -Value ' '  # sentinel: a copy that never happens must not read back as a stale result
[Native]::Tap(0x41, $false, $true)  # Ctrl+A
[Native]::Tap(0x43, $false, $true)  # Ctrl+C
Start-Sleep -Milliseconds 200
$actual = Get-Clipboard -Raw
if ($null -eq $actual) { $actual = '' }
$actual = $actual.TrimEnd("`r", "`n")

$pass = ($actual -ceq $Expected)
function Show([string]$s) { ($s.ToCharArray() | ForEach-Object { if ([int]$_ -lt 128) { [string]$_ } else { '\u{0:X4}' -f [int]$_ } }) -join '' }
Write-Host "Typed   : $Type"
Write-Host "Actual  : $(Show $actual)"
Write-Host "Expected: $(Show $Expected)"
Write-Host ($(if ($pass) { 'PASS' } else { 'FAIL' }))

if (-not $KeepOpen) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
exit $(if ($pass) { 0 } else { 1 })

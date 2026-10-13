# Generates the checked-in EVKey icons: tray-v.png, tray-e.png (32px) and icon.ico (16/32/48/256).
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File crates/app/icons/generate.ps1
Add-Type -AssemblyName System.Drawing
$out = Split-Path -Parent $MyInvocation.MyCommand.Path

function New-RoundedPath([single]$x, [single]$y, [single]$w, [single]$h, [single]$r) {
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $r * 2
    $p.AddArc($x, $y, $d, $d, 180, 90)
    $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $p.CloseFigure()
    return $p
}

function New-Glyph([int]$size, [string]$letter, [System.Drawing.Color]$background, [System.Drawing.Color]$foreground) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
    $g.Clear([System.Drawing.Color]::Transparent)
    $pad = [single]($size * 0.04)
    $path = New-RoundedPath $pad $pad ([single]($size - 2 * $pad)) ([single]($size - 2 * $pad)) ([single]($size * 0.22))
    $brush = New-Object System.Drawing.SolidBrush $background
    $g.FillPath($brush, $path)
    $font = New-Object System.Drawing.Font 'Segoe UI', ([single]($size * 0.62)), ([System.Drawing.FontStyle]::Bold), ([System.Drawing.GraphicsUnit]::Pixel)
    $fmt = New-Object System.Drawing.StringFormat
    $fmt.Alignment = [System.Drawing.StringAlignment]::Center
    $fmt.LineAlignment = [System.Drawing.StringAlignment]::Center
    $g.DrawString($letter, $font, (New-Object System.Drawing.SolidBrush $foreground), [System.Drawing.RectangleF]::new(0, 0, $size, $size), $fmt)
    $g.Dispose()
    return $bmp
}

function Get-PngBytes($bmp) {
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    return ,$ms.ToArray()
}

$red = [System.Drawing.Color]::FromArgb(0xDA, 0x25, 0x1D)
$yellow = [System.Drawing.Color]::FromArgb(0xFF, 0xCD, 0x00)
$slate = [System.Drawing.Color]::FromArgb(0x47, 0x55, 0x69)
$white = [System.Drawing.Color]::White

(New-Glyph 32 'V' $red $yellow).Save("$out/tray-v.png", [System.Drawing.Imaging.ImageFormat]::Png)
(New-Glyph 32 'E' $slate $white).Save("$out/tray-e.png", [System.Drawing.Imaging.ImageFormat]::Png)

$images = foreach ($s in 16, 32, 48, 256) { ,@($s, (Get-PngBytes (New-Glyph $s 'V' $red $yellow))) }
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter $ms
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$images.Count)
$offset = 6 + 16 * $images.Count
foreach ($img in $images) {
    $dim = if ($img[0] -ge 256) { 0 } else { $img[0] }
    $bw.Write([byte]$dim)
    $bw.Write([byte]$dim)
    $bw.Write([byte]0)
    $bw.Write([byte]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]32)
    $bw.Write([uint32]$img[1].Length)
    $bw.Write([uint32]$offset)
    $offset += $img[1].Length
}
foreach ($img in $images) { $bw.Write([byte[]]$img[1]) }
$bw.Flush()
[System.IO.File]::WriteAllBytes("$out/icon.ico", $ms.ToArray())
Write-Host "Wrote tray-v.png, tray-e.png, icon.ico to $out"

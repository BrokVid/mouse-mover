[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$Png,
    [Parameter(Mandatory)] [string]$Ico
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$pngSizes = @(16, 24, 32, 48, 64)

function New-ScaledBitmap {
    param([Parameter(Mandatory)] [int]$Size)
    $scaled = [System.Drawing.Bitmap]::new($Size, $Size)
    $g = [System.Drawing.Graphics]::FromImage($scaled)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.DrawImage([System.Drawing.Image]::FromFile($Png), 0, 0, $Size, $Size)
    $g.Dispose()
    return $scaled
}

$entries = @()
foreach ($size in $pngSizes) {
    $scaled = New-ScaledBitmap $size
    $ms = New-Object System.IO.MemoryStream
    $scaled.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $scaled.Dispose()
    $entries += @{ Size = $size; Data = $ms.ToArray() }
    $ms.Dispose()
}

$out = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($out)
$writer.Write([uint16]0)
$writer.Write([uint16]1)
$writer.Write([uint16]$entries.Count)

$offset = 6 + 16 * $entries.Count
foreach ($entry in $entries) {
    $byteSize = if ($entry.Size -ge 256) { 0 } else { $entry.Size }
    $writer.Write([byte]$byteSize)
    $writer.Write([byte]$byteSize)
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([uint16]1)
    $writer.Write([uint16]32)
    $writer.Write([uint32]$entry.Data.Length)
    $writer.Write([uint32]$offset)
    $offset += $entry.Data.Length
}
foreach ($entry in $entries) {
    $writer.Write($entry.Data)
}
$writer.Flush()
[System.IO.File]::WriteAllBytes($Ico, $out.ToArray())
$writer.Dispose()

$check = [System.Drawing.Icon]::new($Ico)
Write-Host "Wrote ${Ico}: $($entries.Count) images, Icon loads at $($check.Width)x$($check.Height)"
$check.Dispose()

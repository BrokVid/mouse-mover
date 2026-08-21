[CmdletBinding()]
param(
    [string]$Version
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
$cargoToml = Join-Path $repoRoot 'Cargo.toml'
$manifestTemplate = Join-Path $repoRoot 'msix\AppxManifest.xml'
$outDir = Join-Path $repoRoot 'msix-out'

if (-not $Version) {
    $cargoVersion = [regex]::Match(
        (Get-Content -Raw $cargoToml),
        '(?m)^version\s*=\s*"(?<version>[0-9]+\.[0-9]+\.[0-9]+)"'
    )
    if (-not $cargoVersion.Success) {
        throw "Could not read a three-part package version from $cargoToml."
    }
    $Version = "$($cargoVersion.Groups['version'].Value).0"
}

$versionParts = $Version -split '\.'
if ($versionParts.Count -ne 4 -or @($versionParts | Where-Object {
            $_ -notmatch '^\d+$' -or [uint32]$_ -gt 65535
        }).Count -ne 0) {
    throw 'MSIX version must have four numeric components from 0 through 65535.'
}

$sdkRoot = 'C:\Program Files (x86)\Windows Kits\10\bin'
$makeAppxCandidates = @(Get-ChildItem -Path $sdkRoot -Filter MakeAppx.exe -Recurse |
        Where-Object { $_.Directory.Name -eq 'x64' } |
        Sort-Object FullName -Descending)
if ($makeAppxCandidates.Count -eq 0) {
    throw 'MakeAppx.exe was not found. Install the Windows 10 or 11 SDK.'
}
$makeAppx = $makeAppxCandidates[0].FullName

Add-Type -AssemblyName System.Drawing

Remove-Item -Recurse -Force $outDir -ErrorAction Ignore
New-Item -ItemType Directory -Force $outDir | Out-Null

$template = Get-Content -Raw $manifestTemplate
$iconPath = Join-Path $repoRoot 'assets\icon.ico'
$icon = [System.Drawing.Icon]::new($iconPath, 256, 256)

function New-Logo {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [int]$Size
    )

    $bitmap = [System.Drawing.Bitmap]::new($Size, $Size)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([System.Drawing.Color]::Transparent)
        $graphics.DrawIcon($icon, [System.Drawing.Rectangle]::new(0, 0, $Size, $Size))
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

$packagePaths = @()
foreach ($architecture in @('x86', 'x64')) {
    $target = if ($architecture -eq 'x86') {
        Join-Path $repoRoot 'target\i686-pc-windows-msvc\release'
    }
    else {
        Join-Path $repoRoot 'target\release'
    }
    $exe = Join-Path $target 'mouse-mover.exe'
    $pdb = Join-Path $target 'mouse_mover.pdb'
    if (-not (Test-Path $exe) -or -not (Test-Path $pdb)) {
        throw "Missing release build for $architecture. Run the matching cargo build first."
    }

    $packageDir = Join-Path $outDir "package-$architecture"
    $assetsDir = Join-Path $packageDir 'Assets'
    New-Item -ItemType Directory -Force $assetsDir | Out-Null
    Copy-Item $exe (Join-Path $packageDir 'mouse-mover.exe')
    $manifest = $template.Replace('@VERSION@', $Version).Replace('@ARCHITECTURE@', $architecture)
    $manifest | Set-Content -NoNewline -Encoding utf8 (Join-Path $packageDir 'AppxManifest.xml')
    New-Logo -Path (Join-Path $assetsDir 'Square44x44Logo.png') -Size 44
    New-Logo -Path (Join-Path $assetsDir 'Square150x150Logo.png') -Size 150
    New-Logo -Path (Join-Path $assetsDir 'StoreLogo.png') -Size 50

    $packagePath = Join-Path $outDir "MouseMover_$Version`_$architecture.msix"
    & $makeAppx pack /d $packageDir /p $packagePath /o
    if ($LASTEXITCODE -ne 0) {
        throw "MakeAppx failed while packaging $architecture."
    }
    $packagePaths += $packagePath

    $symbolZip = Join-Path $outDir "mouse-mover-$architecture.zip"
    Compress-Archive -Path $pdb -DestinationPath $symbolZip -CompressionLevel Optimal
    Move-Item $symbolZip (Join-Path $outDir "mouse-mover-$architecture.appxsym")
}

$bundleDir = Join-Path $outDir 'bundle'
New-Item -ItemType Directory -Force $bundleDir | Out-Null
foreach ($packagePath in $packagePaths) {
    Copy-Item $packagePath $bundleDir
}
$bundlePath = Join-Path $outDir "MouseMover_$Version`_x86_x64.msixbundle"
& $makeAppx bundle /d $bundleDir /p $bundlePath /bv $Version /o
if ($LASTEXITCODE -ne 0) {
    throw 'MakeAppx failed while creating the MSIX bundle.'
}

$uploadDir = Join-Path $outDir 'upload'
New-Item -ItemType Directory -Force $uploadDir | Out-Null
Copy-Item $bundlePath $uploadDir
Get-ChildItem -Path $outDir -Filter '*.appxsym' | Copy-Item -Destination $uploadDir
$uploadZip = Join-Path $outDir "MouseMover_$Version`_x86_x64.zip"
Compress-Archive -Path (Join-Path $uploadDir '*') -DestinationPath $uploadZip -CompressionLevel Optimal
$uploadPath = [System.IO.Path]::ChangeExtension($uploadZip, '.msixupload')
Move-Item $uploadZip $uploadPath

$icon.Dispose()
Write-Host "Created Store upload package: $uploadPath"

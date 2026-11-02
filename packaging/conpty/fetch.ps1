param(
  [string[]]$Dest = @("target/release", "target/debug")
)

$ErrorActionPreference = "Stop"

$Version = "1.25.260930003"
$Sha256 = "02b07b349af66d801159bdf9e440d4a1ce78bb951f37fc8609731665afdae7ee"
$Url = "https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/$Version/microsoft.windows.console.conpty.$Version.nupkg"

$work = Join-Path ([System.IO.Path]::GetTempPath()) "nova-conpty-$Version"
$pkg = Join-Path $work "conpty.zip"
$out = Join-Path $work "pkg"

if (-not (Test-Path $out)) {
  New-Item -ItemType Directory -Force $work | Out-Null
  Invoke-WebRequest -Uri $Url -OutFile $pkg -UseBasicParsing
  $actual = (Get-FileHash -Algorithm SHA256 $pkg).Hash.ToLower()
  if ($actual -ne $Sha256) {
    Remove-Item -Force $pkg
    throw "ConPTY package hash mismatch: expected $Sha256, got $actual"
  }
  Expand-Archive -Path $pkg -DestinationPath $out -Force
}

$files = @(
  (Join-Path $out "runtimes/win-x64/native/conpty.dll"),
  (Join-Path $out "build/native/runtimes/x64/OpenConsole.exe")
)

foreach ($d in $Dest) {
  New-Item -ItemType Directory -Force $d | Out-Null
  foreach ($f in $files) {
    Copy-Item -Force $f $d
  }
  Write-Host "ConPTY $Version -> $d"
}

# Install

The Windows release ships as `milner.exe`.

## Install Script

From PowerShell:

```powershell
irm https://paredez.dev/install.ps1 | iex
```

## Manual Download

Download the latest release asset and verify its SHA-256 checksum:

```powershell
$asset = "milner-x86_64-pc-windows-msvc.zip"
$base = "https://github.com/AdrianParedez/milner/releases/latest/download"

Invoke-WebRequest "$base/$asset" -OutFile $asset
Invoke-WebRequest "$base/$asset.sha256" -OutFile "$asset.sha256"

$expected = (Get-Content "$asset.sha256").Split(" ")[0]
$actual = (Get-FileHash $asset -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "checksum mismatch" }

Expand-Archive $asset -DestinationPath .\milner
.\milner\milner.exe --no-config powershell -NoProfile -Command "exit 0"
```

## Install From Source

Milner requires Rust stable on Windows:

```powershell
cargo install --git https://github.com/AdrianParedez/milner --locked
milner --no-config powershell -NoProfile -Command "exit 0"
```

## Verify The Binary

A basic verification command should exit successfully and print no output:

```powershell
milner.exe --no-config powershell -NoProfile -Command "exit 0"
```

To verify child exit-code propagation:

```powershell
milner.exe --no-config powershell -NoProfile -Command "exit 7"
$LASTEXITCODE
```

The final command should print `7`.

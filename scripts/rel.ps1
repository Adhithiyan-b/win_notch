$env:PATH = "C:\Users\badhi\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20261020-ucrt-x86_64\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"
$o = cargo build --release 2>&1
if ($LASTEXITCODE -eq 0) { "BUILD OK" } else { $o | Select-String 'error' | Select-Object -First 20 | ForEach-Object { $_.Line } }

param(
    [string]$ArchivePath = (Join-Path $PSScriptRoot '..\NIKL_Korean-Korean_Braille_Parallel_Corpus_2025_v1.0.zip'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\test_cases\corpus'),
    [int]$ChunkSize = 25000
)

$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.IO.Compression.FileSystem

$pattern = ' a1b''k2l@cif/msp"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\0z7(_?w]#y)='
$existingWorld = @{}
$existingWorldByInput = @{}
$existingPaths = @()
$legacyPath = Join-Path $OutputDirectory 'sentence.json'
if (Test-Path -LiteralPath $legacyPath) {
    $existingPaths += $legacyPath
}
if (Test-Path -LiteralPath $OutputDirectory) {
    $existingPaths += @(Get-ChildItem -LiteralPath $OutputDirectory -Filter 'sentence_*.json' -File | ForEach-Object FullName)
}
foreach ($existingPath in $existingPaths) {
    $existingCases = Get-Content -LiteralPath $existingPath -Raw | ConvertFrom-Json
    foreach ($case in $existingCases) {
        if ($case.id -and $null -ne $case.world) {
            $existingWorld[[string]$case.id] = [string]$case.world
        }
        if ($case.input -and $null -ne $case.world) {
            $existingWorldByInput[[string]$case.input] = [string]$case.world
        }
    }
}

$archive = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $ArchivePath))
$cases = [System.Collections.Generic.List[object]]::new()

try {
    foreach ($entry in $archive.Entries | Where-Object Name -like '*.json') {
        $reader = [System.IO.StreamReader]::new($entry.Open(), [System.Text.Encoding]::UTF8)
        try {
            $document = $reader.ReadToEnd() | ConvertFrom-Json
        }
        finally {
            $reader.Dispose()
        }

        foreach ($record in $document.parallel) {
            # NIKL uses U+0020 between cells. braillify's Unicode API represents
            # the same blank cell as U+2800.
            $target = [string]$record.target
            $unicode = $target.Replace(' ', [string][char]0x2800)
            $internal = [System.Text.StringBuilder]::new($unicode.Length)
            $expected = [System.Text.StringBuilder]::new()
            foreach ($cell in $unicode.ToCharArray()) {
                $index = [int][char]$cell - 0x2800
                if ($index -lt 0 -or $index -ge $pattern.Length) {
                    throw "NIKL target contains a non-braille cell: U+$('{0:X4}' -f [int][char]$cell)"
                }
                [void]$internal.Append($pattern[$index])
                [void]$expected.Append($index)
            }

            $case = [ordered]@{
                input = [string]$record.source
                internal = $internal.ToString()
                expected = $expected.ToString()
                unicode = $unicode
            }
            $id = [string]$record.id
            if ($existingWorld.ContainsKey($id)) {
                $case.world = $existingWorld[$id]
            }
            elseif ($existingWorldByInput.ContainsKey($case.input)) {
                $case.world = $existingWorldByInput[$case.input]
            }
            $cases.Add([PSCustomObject]$case)
        }
    }
}
finally {
    $archive.Dispose()
}

[System.IO.Directory]::CreateDirectory($OutputDirectory) | Out-Null
$writtenPaths = @()
$shardCount = [Math]::Ceiling($cases.Count / $ChunkSize)
for ($shardIndex = 0; $shardIndex -lt $shardCount; $shardIndex++) {
    $offset = $shardIndex * $ChunkSize
    $count = [Math]::Min($ChunkSize, $cases.Count - $offset)
    $shardPath = Join-Path $OutputDirectory ('sentence_{0:D2}.json' -f ($shardIndex + 1))
    [System.IO.File]::WriteAllText(
        $shardPath,
        ($cases.GetRange($offset, $count) | ConvertTo-Json -Depth 3),
        [System.Text.UTF8Encoding]::new($false)
    )
    $writtenPaths += $shardPath
}

Get-ChildItem -LiteralPath $OutputDirectory -Filter 'sentence_*.json' -File |
    Where-Object FullName -NotIn $writtenPaths |
    ForEach-Object { Remove-Item -LiteralPath $_.FullName }

Write-Host "Imported $($cases.Count) NIKL parallel corpus records into $shardCount shards"

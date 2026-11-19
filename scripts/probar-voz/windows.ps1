# La prueba de voz de H3a con el reconocimiento de voz de Windows (el de
# SAPI, `System.Speech`, en castellano), para compararlo con Whisper (D2).
#
#     powershell -ExecutionPolicy Bypass -File scripts\probar-voz\windows.ps1 [-Carpeta <carpeta>]
#
# Lo prueba de dos maneras y deja cada una en `<carpeta>\resultados\`:
#
# - `windows-dictado.json`: dictado libre, como el de Windows.
# - `windows-vocabulario.json`: solo puede oír las palabras de la clase, los
#   números y las de unión («con», «coma», «ponle»…). Es lo más que se le
#   puede ayudar con la lista de alumnos, como el contexto de Whisper.
#
# Luego, `cargo run --release --example probar_voz -- --informe` los compara.
# El reconocimiento de Windows **desde un fichero** solo lo da SAPI; el
# moderno (WinRT) escucha solo del micrófono, y usa el mismo paquete de voz.

param(
    [string]$Carpeta = (Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'Gestor de Notas\prueba-voz')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech

$info = [System.Speech.Recognition.SpeechRecognitionEngine]::InstalledRecognizers() |
    Where-Object { $_.Culture.Name -eq 'es-ES' } | Select-Object -First 1
if (-not $info) { throw 'Windows no tiene el reconocimiento de voz en castellano (es-ES).' }

$datos = Join-Path $PSScriptRoot '..\..\src-tauri\tests\datos\voz'
$clase = Get-Content (Join-Path $datos 'clase.txt') -Encoding UTF8 | Where-Object { $_.Trim() }
$resultados = Join-Path $Carpeta 'resultados'
New-Item -ItemType Directory -Force $resultados | Out-Null
$grabaciones = Get-ChildItem $Carpeta -Filter '??.wav' | Sort-Object Name
if (-not $grabaciones) { throw "No hay grabaciones en $Carpeta" }

function New-GramaticaVocabulario {
    $numeros = 'cero uno dos tres cuatro cinco seis siete ocho nueve diez once doce trece catorce quince ' +
        'dieciséis diecisiete dieciocho diecinueve veinte veintiuno veintidós veintitrés veinticuatro ' +
        'veinticinco veintiséis veintisiete veintiocho veintinueve treinta cuarenta cincuenta sesenta ' +
        'setenta ochenta noventa'
    $union = 'con coma punto y medio media un una ponle pon le a para tiene nota de el la'
    $palabras = (($clase -join ' ') + ' ' + $numeros + ' ' + $union) -split '\s+' |
        Where-Object { $_ } | Sort-Object -Unique
    $opciones = New-Object System.Speech.Recognition.Choices -ArgumentList (, [string[]]$palabras)
    $gb = New-Object System.Speech.Recognition.GrammarBuilder
    $gb.Culture = $info.Culture
    $gb.Append($opciones, 1, 10)
    New-Object System.Speech.Recognition.Grammar -ArgumentList $gb
}

function Invoke-Prueba([string]$Motor, $Gramatica) {
    $sapi = New-Object System.Speech.Recognition.SpeechRecognitionEngine -ArgumentList $info
    $sapi.LoadGrammar($Gramatica)
    $frases = @()
    foreach ($g in $grabaciones) {
        $sapi.SetInputToWaveFile($g.FullName)
        $reloj = [Diagnostics.Stopwatch]::StartNew()
        # Una pausa en medio («Hugo Herrero, ocho») corta la frase en dos: se
        # sigue hasta el final del fichero, donde `Recognize` ya no devuelve
        # nada o se queja de que no le queda audio.
        $partes = @()
        while ($true) {
            try { $r = $sapi.Recognize() } catch { break }
            if (-not $r) { break }
            $partes += $r.Text
        }
        $reloj.Stop()
        $frases += [pscustomobject]@{ n = [int]$g.BaseName; texto = ($partes -join ' '); ms = $reloj.ElapsedMilliseconds }
        Write-Host -NoNewline "`r  ${Motor}: $($frases.Count)/$($grabaciones.Count)"
    }
    Write-Host ''
    $sapi.SetInputToNull()
    $sapi.Dispose()
    $json = [pscustomobject]@{ motor = $Motor; frases = $frases } | ConvertTo-Json -Depth 3
    # Sin BOM: PowerShell 5.1 lo pone con -Encoding utf8.
    [IO.File]::WriteAllText((Join-Path $resultados "$Motor.json"), $json, (New-Object Text.UTF8Encoding $false))
}

Invoke-Prueba 'windows-dictado' (New-Object System.Speech.Recognition.DictationGrammar)
Invoke-Prueba 'windows-vocabulario' (New-GramaticaVocabulario)

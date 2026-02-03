Add-Type -AssemblyName System.Drawing

$pngPath = "src-tauri/icons/icon.png"
$icoPath = "src-tauri/icons/icon.ico"
$path = Resolve-Path $pngPath

if (-not $path) {
    Write-Error "PNG not found"
    exit 1
}

try {
    Write-Host "Loading PNG..."
    $img = [System.Drawing.Image]::FromFile($path)
    
    Write-Host "Resizing to 256x256..."
    $resized = new-object System.Drawing.Bitmap(256, 256)
    $g = [System.Drawing.Graphics]::FromImage($resized)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.DrawImage($img, 0, 0, 256, 256)
    $g.Dispose()
    
    Write-Host "Creating Icon..."
    $hIcon = $resized.GetHicon()
    $icon = [System.Drawing.Icon]::FromHandle($hIcon)
    
    if (Test-Path $icoPath) {
        Remove-Item $icoPath -Force
    }
    
    $fs = [System.IO.File]::OpenWrite($icoPath)
    $icon.Save($fs)
    $fs.Close()
    
    # Cleanup
    $img.Dispose()
    $resized.Dispose()

    Write-Host "Success! ICO created."
} catch {
    Write-Error "Error: $_"
    exit 1
}

param([string]$Output = "docs/gallery-smoke.png")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class NativeWindowRect {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
}
"@
$process = Start-Process -FilePath (Resolve-Path "target/debug/acme-gallery.exe") -PassThru
try {
  Start-Sleep -Seconds 5
  $process.Refresh()
  if ($process.HasExited -or $process.MainWindowHandle -eq 0) { throw "Gallery window did not remain available" }
  $rect = New-Object NativeWindowRect+RECT
  if (-not [NativeWindowRect]::GetWindowRect($process.MainWindowHandle, [ref]$rect)) { throw "GetWindowRect failed" }
  $width = $rect.Right - $rect.Left
  $height = $rect.Bottom - $rect.Top
  $bitmap = New-Object System.Drawing.Bitmap($width, $height)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try { $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size) } finally { $graphics.Dispose() }
  $destination = Join-Path (Resolve-Path ".") $Output
  $bitmap.Save($destination, [System.Drawing.Imaging.ImageFormat]::Png)
  $bitmap.Dispose()
  Write-Output $destination
} finally {
  if (-not $process.HasExited) { Stop-Process -Id $process.Id }
}

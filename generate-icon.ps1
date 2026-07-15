Add-Type -AssemblyName System.Drawing
$size = 1024
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.Clear([System.Drawing.Color]::Transparent)

# 蓝紫渐变圆形背景
$bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
  (New-Object System.Drawing.Point(0, 0)),
  (New-Object System.Drawing.Point($size, $size)),
  [System.Drawing.Color]::FromArgb(255, 59, 130, 246),
  [System.Drawing.Color]::FromArgb(255, 139, 92, 246))
$g.FillEllipse($bg, 0, 0, $size, $size)

# 白色闪电
$pts = @(
  (New-Object System.Drawing.PointF(590, 130)),
  (New-Object System.Drawing.PointF(320, 570)),
  (New-Object System.Drawing.PointF(480, 570)),
  (New-Object System.Drawing.PointF(380, 890)),
  (New-Object System.Drawing.PointF(710, 440)),
  (New-Object System.Drawing.PointF(545, 440)),
  (New-Object System.Drawing.PointF(645, 130))
)
$g.FillPolygon([System.Drawing.Brushes]::White, $pts)

$out = Join-Path $PSScriptRoot "app-icon.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Host "saved $out"

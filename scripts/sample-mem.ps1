$out = 'F:/projects/game2-perf/mem_samples.csv'
't,rss_mb,cpu_s' | Out-File $out
while ($true) {
    $p = Get-Process perf_probe -ErrorAction SilentlyContinue
    if (-not $p) { break }
    '{0},{1:N1},{2:N1}' -f [DateTime]::Now.ToString('HH:mm:ss.fff'), ($p.WorkingSet64/1MB), $p.CPU | Out-File -Append $out
    Start-Sleep -Milliseconds 500
}

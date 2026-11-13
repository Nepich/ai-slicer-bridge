function Invoke-SlicerMcp {
 param([Parameter(Mandatory)][string]$Exe,[Parameter(Mandatory)][string[]]$Requests)
 $info=[System.Diagnostics.ProcessStartInfo]::new()
 $info.FileName=$Exe;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
 $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $agent=[System.Diagnostics.Process]::Start($info)
 try {
  $output=$agent.StandardOutput.ReadToEndAsync()
  $errors=$agent.StandardError.ReadToEndAsync()
  foreach($request in $Requests){$agent.StandardInput.WriteLine($request)}
  $agent.StandardInput.Close()
  if(!$agent.WaitForExit(15000)){$agent.Kill();throw 'MCP smoke timed out'}
  if($agent.ExitCode -ne 0){throw "MCP server failed: $($errors.GetAwaiter().GetResult())"}
  $output.GetAwaiter().GetResult() -split '\r?\n' | Where-Object {$_}
 } finally {$agent.Dispose()}
}

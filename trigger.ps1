$exePath = "..\intruder_alert_system\target\release\security-cam.exe"
$userId  = (whoami)

$xml = @"
<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Principals>
    <Principal id="Author">
      <UserId>$userId</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT1M</ExecutionTimeLimit>
    <MultipleInstancesPolicy>Parallel</MultipleInstancesPolicy>
    <Enabled>true</Enabled>
  </Settings>
  <Triggers>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id="0" Path="Security"&gt;&lt;Select Path="Security"&gt;*[System[EventID=4625]]&lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
    </EventTrigger>
  </Triggers>
  <Actions Context="Author">
    <Exec>
      <Command>$exePath</Command>
      <WorkingDirectory>..\intruder_alert_system</WorkingDirectory> 
    </Exec>
  </Actions>
</Task>
"@

$xml | Out-File "$env:TEMP\intruder_alert_system.xml" -Encoding Unicode
schtasks /create /tn "intruder_alert_system" /xml "$env:TEMP\intruder_alert_system.xml" /f
Write-Host "Done!" -ForegroundColor Green
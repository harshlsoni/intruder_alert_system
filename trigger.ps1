$base   = "intruder_alert_system\target\release"
$userId = (whoami)

function Register-SecTask($name, $exe, $eventId) {
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
    <ExecutionTimeLimit>PT10M</ExecutionTimeLimit>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <Hidden>true</Hidden>
    <Enabled>true</Enabled>
  </Settings>
  <Triggers>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id="0" Path="Security"&gt;&lt;Select Path="Security"&gt;*[System[EventID=$eventId]]&lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
    </EventTrigger>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id="0" Path="System"&gt;&lt;Select Path="System"&gt;*[System[EventID=507]]&lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
    </EventTrigger>
  </Triggers>
  <Actions Context="Author">
    <Exec>
      <Command>$base\$exe</Command>
      <WorkingDirectory>..\intruder_alert_system</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"@
    $xml | Out-File "$env:TEMP\$name.xml" -Encoding Unicode
    schtasks /create /tn $name /xml "$env:TEMP\$name.xml" /f
    Write-Host "$name registered!" -ForegroundColor Green
}

Register-SecTask "Intruder_alert_system-Warm"    "warm.exe"    "4800"
Register-SecTask "intruder_alert_system-Capture" "capture.exe" "4625"
Register-SecTask "intruder_alert_system-Release" "release.exe" "4801"
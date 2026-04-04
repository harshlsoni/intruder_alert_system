
Get-Content ".env" | ForEach-Object {
    if ($_ -match "^\s*([^#][^=]*)=(.*)$") {
        $name = $matches[1].Trim()
        $value = $matches[2].Trim()
        [System.Environment]::SetEnvironmentVariable($name, $value)
    }
}


$ProjectPath = $env:root_dir

$WarmTask     = "intruder_alert_system-Warm"
$CaptureTask  = "intruder_alert_system-Capture"
$ReleaseTask  = "intruder_alert_system-Release"


function Register-TaskFromXml {
    param (
        [string]$TaskName,
        [string]$ExeName,
        [string]$SubscriptionXml
    )

    $TaskXml = @"
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription><![CDATA[
$SubscriptionXml
      ]]></Subscription>
    </EventTrigger>
  </Triggers>

  <Principals>
    <Principal id="Author">
      <UserId>SYSTEM</UserId>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>

  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <StartWhenAvailable>true</StartWhenAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
  </Settings>

  <Actions Context="Author">
    <Exec>
      <Command>$ProjectPath\$ExeName</Command>
      <WorkingDirectory>$ProjectPath</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"@

    $TempFile = "$env:TEMP\$TaskName.xml"

    
    $Utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($TempFile, $TaskXml, $Utf8NoBom)

    
    schtasks /Create /TN $TaskName /XML $TempFile /F | Out-Null

    Remove-Item $TempFile
}


Write-Host "Cleaning old tasks..."

$tasks = @($WarmTask, $CaptureTask, $ReleaseTask)

foreach ($task in $tasks) {
    schtasks /Delete /TN $task /F 2>$null | Out-Null
}



$WarmSubscription = @"
<QueryList>
  <Query Id="0" Path="Security">
    <Select Path="Security">
      *[System[(EventID=4800 or EventID=4624)]]
    </Select>
  </Query>
  <Query Id="1" Path="System">
    <Select Path="System">
      *[System[(EventID=1)]]
    </Select>
  </Query>
</QueryList>
"@


$CaptureSubscription = @"
<QueryList>
  <Query Id="0" Path="Security">
    <Select Path="Security">
      *[System[(EventID=4625)]]
    </Select>
  </Query>
</QueryList>
"@


$ReleaseSubscription = @"
<QueryList>
  <Query Id="0" Path="Security">
    <Select Path="Security">
      *[System[(EventID=4801)]]
    </Select>
  </Query>
</QueryList>
"@


Write-Host "Registering tasks..."

Register-TaskFromXml $WarmTask "warm.exe" $WarmSubscription
Register-TaskFromXml $CaptureTask "capture.exe" $CaptureSubscription
Register-TaskFromXml $ReleaseTask "release.exe" $ReleaseSubscription

Write-Host "All tasks registered successfully ✅"

# ================================
# Verify
# ================================
Write-Host "`nRegistered Tasks:"
schtasks /Query | findstr "intruder_alert_system"

$StartupPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$TargetPath  = "$ProjectPath\controller.exe"
$ShortcutPath = "$StartupPath\intruder_alert_system_Controller.lnk"

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)

$Shortcut.TargetPath = $TargetPath
$Shortcut.WorkingDirectory = $ProjectPath
$Shortcut.WindowStyle = 1
$Shortcut.Save()
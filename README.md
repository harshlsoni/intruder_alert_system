# Security Cam

A lightweight Windows security tool written in Rust that silently captures a photo of anyone who enters the wrong password on your laptop and sends an email alert with the image attached.

---

## How It Works

```
Lock screen appears (Event 4800)
        ↓
warm.exe launches → camera initializes silently
        ↓
Wrong password entered (Event 4625)
        ↓
capture.exe → camera already warm → instant photo
        ↓
Email alert sent via SMTP with photo attached
        ↓
PC unlocked (Event 4801)
        ↓
release.exe → camera released → battery saved
```

---

## Features

- Silent background operation — no visible windows
- Near-instant capture — camera pre-warmed on lock screen
- Email alert with photo attached via SMTP
- Fallback direct capture if PC was not locked first
- Automatic logging to file
- Battery friendly — camera only active during lock screen
- Single `.exe` binaries — no runtime dependencies

---

## Project Structure

```
security-cam/
│
├── Cargo.toml
│
├── src/
│   ├── logger.rs        ← file + console logging
│   ├── mailer.rs        ← SMTP email with attachment
│   ├── warm.rs          ← opens camera on lock (Event 4800)
│   ├── capture.rs       ← grabs photo on wrong password (Event 4625)
│   └── release.rs       ← releases camera on unlock (Event 4801)
│
├── captures/            ← saved intruder photos
└── logs/
    └── log.txt          ← all events logged here
```

---

## Prerequisites

- Windows 10 / 11
- [Rust](https://rustup.rs) — `winget install Rustlang.Rustup`
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++** workload
- A Gmail account with **App Password** enabled

---

## Installation

### 1. Clone the repository

```powershell
git clone https://github.com/yourname/security-cam
cd security-cam
```

### 2. Configure email

Open `src/mailer.rs` and fill in your credentials:

```rust
const SMTP_HOST: &str = "smtp.gmail.com";
const SMTP_PORT: u16  = 587;
const SMTP_USER: &str = "your_email@gmail.com";
const SMTP_PASS: &str = "xxxx xxxx xxxx xxxx";  // Gmail App Password
const MAIL_FROM: &str = "your_email@gmail.com";
const MAIL_TO:   &str = "your_email@gmail.com";
```

> **Gmail App Password setup:**
> 1. Go to [myaccount.google.com](https://myaccount.google.com)
> 2. Security → 2-Step Verification → enable
> 3. Security → App passwords → generate for "Mail"
> 4. Paste the 16-character password into `SMTP_PASS`

### 3. Build

```powershell
cargo build --release
```

Produces three binaries in `target/release/`:
- `warm.exe`
- `capture.exe`
- `release.exe`

### 4. Enable Windows Security Auditing

Required for Event ID 4800 to fire:

```powershell
auditpol /set /subcategory:"Other Logon/Logoff Events" /success:enable /failure:enable
```

### 5. Register Task Scheduler tasks

Run as **Administrator**:

```powershell
$base   = "..\intruder_alert_system\target\release"
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

Register-SecTask "intruder_alert_system-Warm"    "warm.exe"    "4800"
Register-SecTask "intruder_alert_system-Capture" "capture.exe" "4625"
Register-SecTask "intruder_alert_system-Release" "release.exe" "4801"
```

---

## Testing

### Check tasks are registered
```powershell
Get-ScheduledTask | Where-Object { $_.TaskName -like "intruder_alert_system*" }
```

### Test manually
```powershell
# Trigger capture directly
..\intruder_alert_system\target\release\capture.exe

# Check photo was saved
Get-ChildItem "..\intruder_alert_system\captures"

# Check logs
Get-Content "..\intruder_alert_system\logs\log.txt"
```

### Test full flow
```powershell
# Watch logs live
Get-Content "..\intruder_alert_system\logs\log.txt" -Wait
```
1. Lock PC → `warm.exe` fires → camera warms up
2. Enter wrong password → `capture.exe` fires → photo taken → email sent
3. Unlock PC → `release.exe` fires → camera released

---

## Task Management

```powershell
# Stop all tasks
Stop-ScheduledTask  -TaskName "intruder_alert_system-Warm"
Stop-ScheduledTask  -TaskName "intruder_alert_system-Capture"
Stop-ScheduledTask  -TaskName "intruder_alert_system-Release"

# Disable all tasks
Disable-ScheduledTask -TaskName "intruder_alert_system-Warm"
Disable-ScheduledTask -TaskName "intruder_alert_system-Capture"
Disable-ScheduledTask -TaskName "intruder_alert_system-Release"

# Remove all tasks
Unregister-ScheduledTask -TaskName "intruder_alert_system-Warm"    -Confirm:$false
Unregister-ScheduledTask -TaskName "intruder_alert_system-Capture" -Confirm:$false
Unregister-ScheduledTask -TaskName "intruder_alert_system-Release" -Confirm:$false
```

---

## Windows Event IDs Used

| Event ID | Meaning | Binary triggered |
|---|---|---|
| `4800` | Workstation locked | `warm.exe` |
| `4625` | Failed login attempt | `capture.exe` |
| `4801` | Workstation unlocked | `release.exe` |

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `escapi` | 4.0.0 | Windows camera capture |
| `image` | 0.24 | JPEG encoding |
| `lettre` | 0.11 | SMTP email |
| `chrono` | 0.4 | Timestamps |

---

## Troubleshooting

**Camera not opening**
- Check no other app is using the camera
- Verify camera index `0` is correct — change in `warm.rs` if needed

**Email not sending**
- Verify Gmail App Password is correct (not your regular password)
- Make sure 2-Step Verification is enabled on your Google account
- Check `logs/log.txt` for the exact error

**Task not firing**
- Run `auditpol /get /subcategory:"Other Logon/Logoff Events"` — must not say `No Auditing`
- Re-run the task registration script as Administrator
- Check `DisallowStartIfOnBatteries` is set to `false`

**Console window flashing**
- Make sure `#![windows_subsystem = "windows"]` is at the top of each `.rs` file
- Rebuild with `cargo build --release`

---

## License

MIT
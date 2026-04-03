
# intruder_alert_system

A modular, multi-process Windows security system written in Rust that silently captures intruder images on failed login attempts, stores them, and sends email alerts — all with robust process isolation and scalable architecture.

---

##  System Architecture

This project follows a **decoupled multi-process pipeline**, where each component is isolated and responsible for a single task.

```

capture.exe
↓
(temp storage)
↓
db_writer.exe
↓
MongoDB
↓
mailer.exe
↓
Email sent

```

This design ensures:
- Better fault tolerance
- Independent execution of critical components
- Easier debugging and scalability

---

##  How It Works

```

Lock screen appears 
↓
warm.exe → camera initializes silently
↓
Wrong password entered 
↓
capture.exe → captures image → saves to temp
↓
db_writer.exe → stores image + metadata in MongoDB
↓
mailer.exe → fetches from DB → sends email alert
↓
PC unlocked 
↓
release.exe → camera released

```

---

##  Features

- Multi-process architecture (robust & scalable)
- Silent background execution (no visible UI)
- Near-instant capture using pre-warmed camera
- MongoDB-based storage for images and metadata
- Email alerts with image + timestamp
- Decoupled pipeline (capture → DB → mail)
- Centralized logging system (fixed absolute path)
- Async execution using process spawning
- Fault-tolerant design (supports retries/extensions)
- Tray controller and dashboard UI support

---

##  Project Structure

```

intruder_alert_system/
│
├── Cargo.toml
│
├── src/
│   ├── logger.rs         ← centralized logging (absolute path)
│   ├── config.rs         ← configuration (paths, constants)
│   ├── db.rs             ← MongoDB connection + schema
│   │
│   ├── warm.rs           ← initializes camera on lock
│   ├── capture.rs        ← captures image + saves temp file
│   ├── db_writer.rs      ← writes image + metadata to MongoDB
│   ├── mailer.rs         ← fetches from DB and sends email
│   ├── release.rs        ← releases camera on unlock
│   │
│   ├── controller.rs     ← system tray controller (toggle system)
│   └── dashboard.rs      ← GUI to view intrusion attempts
│
├── temp/                 ← temporary captured images
├── logs/
│   └── log.txt           ← centralized logs (absolute path)
│
├── trigger.ps1           ← registers scheduled tasks
└── README.md

````

---

##  Prerequisites

- Windows 10 / 11
- [Rust](https://rustup.rs)
- Visual Studio Build Tools (C++ workload)
- MongoDB (local or remote)
- Gmail account with App Password enabled

---

##  Installation

### 1. Clone repository

```powershell
git clone https://github.com/yourname/intruder_alert_system
cd intruder_alert_system
````

---

### 2. Configure environment

####  Email (in `mailer.rs`)

```rust
const SMTP_HOST: &str = "smtp.gmail.com";
const SMTP_PORT: u16  = 587;
const SMTP_USER: &str = "your_email@gmail.com";
const SMTP_PASS: &str = "xxxx xxxx xxxx xxxx";
const MAIL_FROM: &str = "your_email@gmail.com";
const MAIL_TO:   &str = "your_email@gmail.com";
```

---

####  MongoDB (in `config.rs` or `db.rs`)

```rust
const DB_URI: &str = "mongodb://localhost:27017";
const DB_NAME: &str = "intruder_alert";
const COLL_NAME: &str = "attempts";
```

---

### 3. Build

```powershell
cargo build --release
```

Generated binaries:

* `warm.exe`
* `capture.exe`
* `db_writer.exe`
* `mailer.exe`
* `release.exe`
* `controller.exe`
* `dashboard.exe`

---

### 4. Enable Windows Auditing

```powershell
auditpol /set /subcategory:"Other Logon/Logoff Events" /success:enable /failure:enable
```

---

### 5. Register Scheduled Tasks

```powershell
./trigger.ps1
```

---

##  Testing

### Manual pipeline test

```powershell
# Step 1: Capture image
capture.exe

# Step 2: Write to DB
db_writer.exe

# Step 3: Send email
mailer.exe
```

---

### Live logs

```powershell
Get-Content "E:\security-cam_rust\logs\log.txt" -Wait
```

---

### Full system test

1. Lock PC → `warm.exe`
2. Enter wrong password → `capture.exe` → `db_writer.exe` → `mailer.exe`
3. Unlock PC → `release.exe`

---

##  UI Components

###  Controller (Tray App)

* Runs in background
* Enables/disables system
* Starts on system boot (via registry)

###  Dashboard

* Displays intrusion attempts
* Fetches data from MongoDB
* Shows captured images and metadata

---

##  MongoDB Schema (Example)

```json
{
  "_id": ObjectId,
  "timestamp": "2026-04-03T18:30:00",
  "image_path": "temp/img_123.jpg",
  "emailed": true
}
```

---

##  Logging System

* Uses **absolute path** to avoid multi-process conflicts
* All executables write to:

```
...\intruder_alert_system\logs\log.txt
```

---

##  Key Design Decisions

* Decoupled processes instead of monolithic flow
* Temporary storage before DB write
* DB-driven email system (not direct send)
* Independent lifecycle for critical components

---

##  Troubleshooting

### Email not sending

* Ensure Gmail App Password is correct
* Check logs for SMTP errors
* Ensure mailer process is not terminated early

---

### Logs missing

* Ensure absolute log path is used
* Verify all processes point to same file

---

### Camera issues

* Ensure no other app is using camera
* Verify correct camera index

---

### Tasks not triggering

* Re-run `trigger.ps1` as Administrator
* Verify auditing is enabled

---

### UI not working

* Ensure correct working directory
* Check if process is running in Task Manager

---

##  Future Enhancements

* Retry queue for failed emails
* Live log streaming in dashboard
* CV-based intruder detection (AI integration)
* Remote MongoDB (Atlas)
* Windows service support
* Installer package

---

##  License

MIT

```

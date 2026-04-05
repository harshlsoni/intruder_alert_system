
# intruder_alert_system

A modular, multi-process Windows security system written in Rust that silently captures intruder images on failed login attempts, stores them, and sends email alerts with a toggle control panel.

---

##  System Architecture



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


##  How It Works

```

Lock screen 
↓
warm.exe → camera initialized
↓
Failed login 
↓
capture.exe → capture → temp file
↓
db_writer.exe → MongoDB
↓
mailer.exe → email alert
↓
Unlock 
↓
release.exe → camera released

```

---

##  Features

- Multi-process architecture (robust & scalable)
- Silent background execution
- Near-instant capture (pre-warmed camera)
- MongoDB storage (images + metadata)
- Email alerts with captured image
- Decoupled pipeline (capture → DB → mail)
- Centralized logging system
- Async execution via process spawning
- Fault-tolerant design
- Tray controller + dashboard UI

---

##  Project Structure

```

intruder_alert_system/
│
├── Cargo.toml
├── build.rs                
│
├── src/
│   ├── lib.rs              
│   ├── env.rs              
│   ├── config.rs           
│   ├── logger.rs           
│   ├── db.rs               
│   │
│   ├── warm.rs
│   ├── capture.rs
│   ├── db_writer.rs
│   ├── mailer.rs
│   ├── release.rs
│   │
│   ├── controller.rs
│   └── dashboard.rs
│
├── assets/                 
├── logs/
│   └── log.txt
├── temp/                   
│
├── .env.example            
├── trigger.ps1             
└── README.md

````

---

##  Configuration (IMPORTANT)

All configuration is handled via `.env`

###  Step 1: Create `.env`

```powershell
copy .env.example .env
````

---

###  Step 2: Fill values

```env
root_dir=

SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password

MAIL_FROM=your_email@gmail.com
MAIL_TO=receiver@gmail.com

MONGO_URI=mongodb://localhost:27017
DB_NAME=intruder_alert
COLL_NAME=attempts
```

---

###  Notes

* No hardcoding anywhere in code
* Shared across all executables
* Loaded at runtime using `load_env()`
* Automatically copied to `target/` via `build.rs`

---

##  Prerequisites

* Windows 10 / 11
* [Rust](https://rustup.rs)
* Visual Studio Build Tools (C++)
* MongoDB (local or Atlas)
* Gmail App Password enabled

---

##  Installation

### 1. Clone

```powershell
git clone https://github.com/yourname/intruder_alert_system
cd intruder_alert_system
```

---



### 3. Build

```powershell
cargo build --release
```

---

### 4. Enable Windows Auditing

```powershell
auditpol /set /subcategory:"Other Logon/Logoff Events" /success:enable /failure:enable
```

---

### 5. Register Triggers

```powershell
./trigger.ps1
```

---

##  Testing

### Manual pipeline

```powershell
capture.exe
db_writer.exe
mailer.exe
```

---

### Logs (live)

```powershell
Get-Content ".\logs\log.txt" -Wait
```

---

### Full flow

1. Lock PC → warm.exe
2. Wrong password → capture → db_writer → mailer
3. Unlock → release.exe

---

##  UI Components

###  Controller (Tray App)

* Runs in background
* Enables/disables system
* Starts at login

---

###  Dashboard

* Displays intrusion attempts
* Fetches from MongoDB
* Shows images + metadata

---

##  MongoDB Schema

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

* Centralized log file
* Shared across all processes
* Uses runtime-configured path

```
<root_dir>/logs/log.txt
```

---

##  Key Design Decisions

* Multi-process over monolithic design
* `.env` over hardcoded config
* Temp → DB → Mail pipeline
* Absolute path handling
* Executable-relative resource loading

---

##  Troubleshooting

### Email not sending

* Check SMTP credentials
* Verify Gmail App Password
* Check logs

---

### `.env` not loading

* Ensure `.env` exists in `target/release`
* Rebuild project (build.rs copies it)

---

### Logs missing

* Verify `root_dir`
* Ensure all processes use same config

---

### Tasks not triggering

* Run `trigger.ps1` as Admin
* Check Windows Event Viewer

---

### Camera issues

* Close other apps using camera
* Check device index

---


## 📜 License

MIT

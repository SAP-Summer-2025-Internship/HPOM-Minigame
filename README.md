# HPOM Minigame: Low-Level HTTP Survey App

Live site: https://hpom-minigame.fly.dev/

This project is a low-level, manual HTTP website request handler written in Rust. It collects and stores user responses about SAP's HPOM architecture, using a custom session and button flow. The app is designed for educational/demo purposes and is hosted externally on Fly.io, leveraging persistent volumes for data storage.

## Features
- Manual HTTP request parsing and response (no frameworks)
- Multi-page button-driven survey about HPOM roles and architecture
- Session management via cookies
- User responses are stored as CSV in a Fly.io volume (`/data/data.csv`)
- Pretty HTML view of all collected data at [`/view-data`](https://hpom-minigame.fly.dev/view-data)
- Designed for deployment on Fly.io with persistent storage
- Limits the number of concurrent connections/threads to prevent server overload

## How It Works
- Users interact with a series of HTML pages, making choices about HPOM roles and questions.
- Each session's responses are summarized and written to `/data/data.csv` on the attached Fly.io volume (if present).
- If the volume is not attached, the app logs a debug message and skips writing.
- Visit `/view-data` to see all collected responses in a formatted table.

## Deployment
1. **Create a Fly.io app and volume:**
   ```sh
   fly launch
   fly volume create mydata -a <app-name> -r <region>
   ```
2. **Configure `fly.toml`:**
   Ensure you have:
   ```toml
   [[mounts]]
     source = "mydata"
     destination = "/data"
   ```
3. **Deploy:**
   ```sh
   fly deploy -a <app-name>
   ```

## Downloading Collected Data
To download the `data.csv` file from your Fly.io volume:

1. Open an SFTP shell to your app:
   ```sh
   flyctl ssh sftp shell -a <app-name>
   ```
2. In the SFTP prompt, run:
   ```sftp
   get /data/data.csv
   ```
3. The file will be downloaded to your current local directory.

## Notes
- The app is intentionally low-level: all HTTP parsing, session, and file I/O are manual.
- The CSV file is only written if the `/data/data.csv` file exists (i.e., the volume is attached).
- For multiple machines, create a volume per machine with the same name in the same region.

---

© 2025 SAP Summer Internship Demo

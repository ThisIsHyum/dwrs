# 📦 dwrs — Parallel Downloader with Progress Bars

dwrs is a lightweight Rust-powered CLI utility for downloading files from the internet with parallelism and stylish progress bars.
🚀 Features

    📥 Download one or multiple URLs in parallel

    📁 Support for custom output file names via --output

    🧵 Control the number of simultaneous downloads with --jobs

    🗂 Batch download from a plain text file (url [output] per line)

    📊 Clean, informative progress bars using indicatif

    🧾 Logging to console with env_logger

    🐧 Easily build .deb and .rpm packages for distribution

🔧 Example usage

# Download a single file
dwrs https://example.com/file.zip

# Download multiple files
dwrs https://a.com/a.zip https://b.com/b.zip

# With custom output filenames
dwrs https://a.com/a.zip https://b.com/b.zip --output one.zip two.zip

# From a list file
dwrs --file downloads.txt

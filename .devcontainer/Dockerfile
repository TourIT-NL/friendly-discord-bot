ARG VARIANT="bookworm"
FROM mcr.microsoft.com/vscode/devcontainers/base:0-${VARIANT}

# Install Tauri dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive 
    && apt-get -y install --no-install-recommends 
    libgtk-3-dev 
    libwebkit2gtk-4.1-dev 
    libayatana-appindicator3-dev 
    librsvg2-dev 
    libssl-dev 
    build-essential 
    curl 
    wget 
    pkg-config 
    libasound2-dev 
    libdbus-1-dev 
    libpulse-dev 
    libudev-1-dev 
    libsecret-1-dev 
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Pre-configure cargo
USER vscode
RUN mkdir -p /home/vscode/.cargo && 
    echo '[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]' > /home/vscode/.cargo/config.toml

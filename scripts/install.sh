#!/bin/bash

PROGRAM_NAME="racky"
REPOSITORY="DervexDev/racky"

set -eo pipefail

dependencies=(
    curl
    unzip
    uname
    tr
)

for dep in "${dependencies[@]}"; do
    if ! command -v "$dep" >/dev/null 2>&1; then
        echo "ERROR: '$dep' is not installed or available." >&2
        exit 1
    fi
done

if [ -z "$BASH_VERSION" ] && [ -z "$ZSH_VERSION" ]; then
    echo "WARNING: You are using an unsupported shell. Automatic installation may not work correctly." >&2
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS" in
darwin) OS="macos" ;;
linux) OS="linux" ;;
cygwin* | mingw* | msys*) OS="windows" ;;
*)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
x86_64) ARCH="x86_64" ;;
x86-64) ARCH="x86_64" ;;
arm64) ARCH="aarch64" ;;
aarch64) ARCH="aarch64" ;;
*)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

VERSION_PATTERN="[0-9]*\\.[0-9]*\\.[0-9]*"
API_URL="https://api.github.com/repos/$REPOSITORY/releases/latest"
FILE_PATTERN="${PROGRAM_NAME}-${VERSION_PATTERN}-${OS}-${ARCH}.zip"

echo "[1/3] Looking for latest $PROGRAM_NAME release"

if [ ! -z "$GITHUB_TOKEN" ]; then
    RELEASE_JSON_DATA=$(curl --proto '=https' --tlsv1.2 -sSf "$API_URL" \
        -H "X-GitHub-Api-Version: 2022-11-28" -H "Authorization: token $GITHUB_TOKEN")
else
    RELEASE_JSON_DATA=$(curl --proto '=https' --tlsv1.2 -sSf "$API_URL" \
        -H "X-GitHub-Api-Version: 2022-11-28")
fi

if [ -z "$RELEASE_JSON_DATA" ] || [[ "$RELEASE_JSON_DATA" == *"Not Found"* ]]; then
    echo "ERROR: Latest release was not found. Please check your network connection." >&2
    exit 1
fi

RELEASE_ASSET_ID=""
RELEASE_ASSET_NAME=""

while IFS= read -r current_line; do
    if [[ "$current_line" == *'"url":'* && "$current_line" == *"https://api.github.com/repos/$REPOSITORY/releases/assets/"* ]]; then
        RELEASE_ASSET_ID="${current_line##*/releases/assets/}"
        RELEASE_ASSET_ID="${RELEASE_ASSET_ID%%\"*}"
    elif [[ "$current_line" == *'"name":'* ]]; then
        current_name="${current_line#*: \"}"
        current_name="${current_name%%\"*}"
        if [[ "$current_name" =~ $FILE_PATTERN ]]; then
            if [ -n "$RELEASE_ASSET_ID" ]; then
                RELEASE_ASSET_ID="$RELEASE_ASSET_ID"
                RELEASE_ASSET_NAME="$current_name"
                break
            else
                RELEASE_ASSET_ID=""
            fi
        else
            RELEASE_ASSET_ID=""
        fi
    fi
done <<<"$RELEASE_JSON_DATA"

if [ -z "$RELEASE_ASSET_ID" ] || [ -z "$RELEASE_ASSET_NAME" ]; then
    echo "ERROR: Failed to find asset that matches the pattern \"$FILE_PATTERN\" in the latest release." >&2
    exit 1
fi

echo "[2/3] Downloading '$RELEASE_ASSET_NAME'"

RELEASE_DOWNLOAD_URL="https://api.github.com/repos/$REPOSITORY/releases/assets/$RELEASE_ASSET_ID"
ZIP_FILE="${RELEASE_ASSET_NAME%.*}.zip"

if [ ! -z "$GITHUB_TOKEN" ]; then
    curl --proto '=https' --tlsv1.2 -L -o "$ZIP_FILE" -sSf "$RELEASE_DOWNLOAD_URL" \
        -H "X-GitHub-Api-Version: 2022-11-28" -H "Accept: application/octet-stream" -H "Authorization: token $GITHUB_TOKEN"
else
    curl --proto '=https' --tlsv1.2 -L -o "$ZIP_FILE" -sSf "$RELEASE_DOWNLOAD_URL" \
        -H "X-GitHub-Api-Version: 2022-11-28" -H "Accept: application/octet-stream"
fi

if [ ! -f "$ZIP_FILE" ]; then
    echo "ERROR: Failed to download the release archive '$ZIP_FILE'." >&2
    exit 1
fi

BINARY_NAME="$PROGRAM_NAME"

if [ "$OS" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

unzip -o -q "$ZIP_FILE" "$BINARY_NAME" -d .
rm "$ZIP_FILE"

if [ ! -f "$BINARY_NAME" ]; then
    echo "ERROR: The file '$BINARY_NAME' does not exist in the downloaded archive." >&2
    exit 1
fi

echo "[3/3] Running $PROGRAM_NAME installer"

if [ "$OS" != "windows" ]; then
    chmod +x "$BINARY_NAME"
fi

./"$BINARY_NAME" install $1
rm "$BINARY_NAME"

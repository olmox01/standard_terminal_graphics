#!/bin/sh
# Installer automatico per STG su Alpine Linux

set -e

echo "🏔️  Installer STG per Alpine Linux"
echo "=================================="

# Verifica che siamo su Alpine
if [ ! -f /etc/alpine-release ]; then
    echo "❌ Errore: questo script è per Alpine Linux"
    exit 1
fi

# Installa dipendenze
echo "📦 Installazione dipendenze..."
apk add --no-cache rust cargo musl-dev

# Rileva architettura
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) TARGET="x86_64-unknown-linux-musl" ;;
    i386|i686) TARGET="i686-unknown-linux-musl" ;;
    aarch64) TARGET="aarch64-unknown-linux-musl" ;;
    armv7l) TARGET="armv7-unknown-linux-musleabihf" ;;
    *) TARGET="$ARCH-unknown-linux-musl" ;;
esac

echo "🎯 Target rilevato: $TARGET"

# Compila per il target specifico
echo "🔨 Compilazione per $TARGET..."
rustup target add $TARGET 2>/dev/null || true
cargo build --release --target $TARGET --bin stg-demo

# Installa
echo "📥 Installazione..."
sudo install -Dm755 target/$TARGET/release/stg-demo /usr/bin/stg-demo
sudo install -dm755 /etc/stg
sudo install -dm755 /usr/share/doc/stg

# Configura
if [ ! -f /etc/stg/config.toml ]; then
    sudo sh -c 'cat > /etc/stg/config.toml << "EOF"
version = "1.0"
default_width = 80
default_height = 24
color_support = true
unicode_support = true
EOF'
fi

echo "✅ STG installato con successo!"
echo "   Esegui 'stg-demo' per testare"

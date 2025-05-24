#!/bin/bash
# filepath: /home/sh/standard_terminal_graphics/uninstall.sh

set -e

echo "🗑️ Standard Terminal Graphics - Uninstaller"
echo "==========================================="

# Colori per output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Verifica se è installato via .deb
if dpkg -l | grep -q standard-terminal-graphics; then
    print_status "Rimozione pacchetto .deb..."
    sudo apt remove standard-terminal-graphics -y
    print_success "Pacchetto rimosso"
else
    # Rimozione manuale
    print_status "Rimozione installazione manuale..."
    
    if [ -f "/usr/local/bin/stg-demo" ]; then
        sudo rm /usr/local/bin/stg-demo
        print_success "Binario rimosso"
    fi
    
    if [ -d "/usr/local/share/doc/standard-terminal-graphics" ]; then
        sudo rm -rf /usr/local/share/doc/standard-terminal-graphics
        print_success "Documentazione rimossa"
    fi
    
    if [ -f "/usr/share/applications/stg-demo.desktop" ]; then
        sudo rm /usr/share/applications/stg-demo.desktop
        print_success "Desktop entry rimosso"
    fi
fi

print_success "🎉 Disinstallazione completata!"
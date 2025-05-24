#!/bin/bash
# filepath: /home/sh/standard_terminal_graphics/install.sh

set -e

echo "🚀 Standard Terminal Graphics - Installer"
echo "=========================================="

# Colori per output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Funzione per stampare messaggi colorati
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Verifica se Rust è installato
if ! command -v cargo &> /dev/null; then
    print_error "Cargo/Rust non trovato!"
    echo "Installa Rust da: https://rustup.rs/"
    exit 1
fi

print_success "Rust/Cargo trovato: $(cargo --version)"

# Verifica se cargo-deb è installato
if ! command -v cargo-deb &> /dev/null; then
    print_status "Installazione cargo-deb..."
    cargo install cargo-deb
    print_success "cargo-deb installato"
else
    print_success "cargo-deb già installato"
fi

# Compila in modalità release
print_status "Compilazione in modalità release..."
cargo build --release --bin stg-demo

if [ $? -eq 0 ]; then
    print_success "Compilazione completata"
else
    print_error "Errore durante la compilazione"
    exit 1
fi

# Verifica se siamo su un sistema che supporta .deb
if command -v dpkg &> /dev/null; then
    print_status "Sistema Debian/Ubuntu rilevato, creazione pacchetto .deb..."
    
    # Crea il pacchetto .deb
    cargo deb --no-build
    
    if [ $? -eq 0 ]; then
        print_success "Pacchetto .deb creato in target/debian/"
        
        # Trova il file .deb creato
        DEB_FILE=$(find target/debian -name "*.deb" | head -n 1)
        
        if [ -n "$DEB_FILE" ]; then
            print_status "Installazione del pacchetto: $DEB_FILE"
            
            # Chiedi conferma per l'installazione
            read -p "Installare il pacchetto? (y/N): " -n 1 -r
            echo
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                sudo dpkg -i "$DEB_FILE"
                
                # Risolvi eventuali dipendenze mancanti
                sudo apt-get install -f -y
                
                print_success "Installazione completata!"
                print_status "Puoi ora eseguire: stg-demo"
            else
                print_status "Installazione saltata. Puoi installare manualmente con:"
                echo "  sudo dpkg -i $DEB_FILE"
            fi
        fi
    else
        print_error "Errore durante la creazione del pacchetto .deb"
        exit 1
    fi
else
    # Installazione manuale per sistemi non-Debian
    print_status "Sistema non-Debian rilevato, installazione manuale..."
    
    # Copia il binario
    sudo cp target/release/stg-demo /usr/local/bin/stg-demo
    sudo chmod +x /usr/local/bin/stg-demo
    
    # Crea directory documentazione
    sudo mkdir -p /usr/local/share/doc/standard-terminal-graphics
    sudo cp README.md LICENSE /usr/local/share/doc/standard-terminal-graphics/
    
    # Desktop entry (opzionale)
    if [ -d "/usr/share/applications" ]; then
        sudo cp stg-demo.desktop /usr/share/applications/
        print_success "Desktop entry installato"
    fi
    
    print_success "Installazione manuale completata!"
    print_status "Puoi ora eseguire: stg-demo"
fi

echo
print_success "🎉 Installazione completata!"
echo
echo "Comandi disponibili:"
echo "  stg-demo          - Avvia la demo del desktop environment"
echo "  stg-demo --help   - Mostra l'aiuto"
echo
echo "Per disinstallare:"
if command -v dpkg &> /dev/null; then
    echo "  sudo apt remove standard-terminal-graphics"
else
    echo "  sudo rm /usr/local/bin/stg-demo"
    echo "  sudo rm -rf /usr/local/share/doc/standard-terminal-graphics"
fi
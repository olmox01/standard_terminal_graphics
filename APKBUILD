# Maintainer: STG Team <team@stg.dev>
pkgname=stg
pkgver=1.0.0
pkgrel=0
pkgdesc="Standard Terminal Graphics - Libreria grafica per terminale"
url="https://github.com/yourrepo/stg"
arch="all"
license="MIT"
depends=""
makedepends="rust cargo"
install="$pkgname.post-install $pkgname.pre-deinstall"
subpackages="$pkgname-doc"
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir/$pkgname-$pkgver"

# Auto-detect target architecture
case "$CARCH" in
    x86_64) _target="x86_64-unknown-linux-musl" ;;
    x86) _target="i686-unknown-linux-musl" ;;
    aarch64) _target="aarch64-unknown-linux-musl" ;;
    armv7) _target="armv7-unknown-linux-musleabihf" ;;
    *) _target="$CARCH-unknown-linux-musl" ;;
esac

prepare() {
    default_prepare
    cargo fetch --target $_target --locked
}

build() {
    cargo build --release --target $_target --bin stg-demo --locked
}

check() {
    cargo test --release --target $_target --locked
}

package() {
    install -Dm755 target/$_target/release/stg-demo "$pkgdir"/usr/bin/stg-demo
    install -Dm644 LICENSE "$pkgdir"/usr/share/licenses/$pkgname/LICENSE
    install -Dm644 README.md "$pkgdir"/usr/share/doc/$pkgname/README.md
    
    # Crea directory di configurazione
    install -dm755 "$pkgdir"/etc/stg
}

doc() {
    default_doc
    install -Dm644 "$builddir"/docs/* "$subpkgdir"/usr/share/doc/$pkgname/ 2>/dev/null || true
}

sha512sums=""

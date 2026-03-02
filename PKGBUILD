# Maintainer: Faynot <faynotdev@gmail.com>
pkgname=dumbshot
pkgver=0.2.0
pkgrel=2
pkgdesc="An elegant, painless one-click screenshot utility for Wayland (grim + slurp)"
arch=('x86_64')
url="https://github.com/Faynot/dumbshot"
license=('MIT')
depends=(
  'grim'
  'slurp'
  'wl-clipboard'
  'libnotify'
  'eww'
  'hyprland'
  'xdg-utils'
)
makedepends=(
  'rust'
  'cargo'
)
optdepends=(
  'satty: Screenshot annotation editor'
)
source=("git+https://github.com/Faynot/$pkgname.git#branch=eww-transition")
sha256sums=('SKIP')

prepare() {
  cd "$pkgname"
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "$pkgname"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --all-features
}

check() {
  cd "$pkgname"
  cargo test --frozen --release
}

package() {
  cd "$pkgname"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

  if [ -f LICENSE ]; then
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  fi
}

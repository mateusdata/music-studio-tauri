#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SYSROOT="$PROJECT_ROOT/.sysroot"
DEBS_DIR="$SYSROOT/debs"

mkdir -p "$DEBS_DIR"

ROOT_PKGS=(
  libwebkit2gtk-4.1-dev
  libgtk-3-dev
  libayatana-appindicator3-dev
  librsvg2-dev
  libsoup-3.0-dev
  libjavascriptcoregtk-4.1-dev
)

echo "[sysroot] Resolving missing -dev packages..."
ALL_DEV_PKGS=$(apt-cache depends --recurse --no-recommends --no-suggests --no-conflicts --no-breaks --no-replaces --no-enhances "${ROOT_PKGS[@]}" \
  | grep -E '^[a-zA-Z0-9]' | grep -v ':i386' | grep -E '(-dev$|multidev$|protocols$|gir1\.2-)' | sort -u)

MISSING_PKGS=()
for pkg in $ALL_DEV_PKGS; do
  if ! dpkg -s "$pkg" >/dev/null 2>&1; then
    MISSING_PKGS+=("$pkg")
  fi
done

if [ ${#MISSING_PKGS[@]} -eq 0 ]; then
  echo "[sysroot] All required -dev packages are already installed system-wide."
else
  echo "[sysroot] Downloading ${#MISSING_PKGS[@]} missing packages into $DEBS_DIR..."
  pushd "$DEBS_DIR" >/dev/null
  for pkg in "${MISSING_PKGS[@]}"; do
    if ! ls "${pkg}_"*.deb >/dev/null 2>&1; then
      apt-get download "$pkg"
    fi
  done
  popd >/dev/null

  echo "[sysroot] Extracting .deb packages into $SYSROOT..."
  for deb in "$DEBS_DIR"/*.deb; do
    dpkg-deb -x "$deb" "$SYSROOT"
  done
fi

echo "[sysroot] Patching pkg-config prefixes..."
for pcdir in "$SYSROOT/usr/lib/x86_64-linux-gnu/pkgconfig" "$SYSROOT/usr/share/pkgconfig"; do
  if [ -d "$pcdir" ]; then
    for pc in "$pcdir"/*.pc; do
      [ -e "$pc" ] || continue
      sed -i "s|^prefix=/usr$|prefix=$SYSROOT/usr|g" "$pc"
      sed -i "s|^exec_prefix=/usr$|exec_prefix=$SYSROOT/usr|g" "$pc"
      sed -i "s|^includedir=/usr/include|includedir=$SYSROOT/usr/include|g" "$pc"
      sed -i "s|^libdir=/usr/lib|libdir=$SYSROOT/usr/lib|g" "$pc"
    done
  fi
done

echo "[sysroot] Fixing broken relative .so symlinks against /usr/lib/x86_64-linux-gnu..."
LIBDIR="$SYSROOT/usr/lib/x86_64-linux-gnu"
if [ -d "$LIBDIR" ]; then
  while IFS= read -r -d '' link; do
    target="$(readlink "$link")"
    if [[ "$target" != /* ]]; then
      if [ ! -e "$LIBDIR/$target" ] && [ -e "/usr/lib/x86_64-linux-gnu/$target" ]; then
        ln -snf "/usr/lib/x86_64-linux-gnu/$target" "$link"
      elif [ ! -e "$LIBDIR/$target" ] && [[ "$target" == /lib/* ]]; then
        :
      fi
    elif [[ "$target" == /lib/x86_64-linux-gnu/* ]]; then
      base="$(basename "$target")"
      if [ -e "/usr/lib/x86_64-linux-gnu/$base" ]; then
        ln -snf "/usr/lib/x86_64-linux-gnu/$base" "$link"
      fi
    fi
  done < <(find "$LIBDIR" -maxdepth 2 -type l -print0)
fi

echo "[sysroot] Done. Verifying pkg-config..."
"$SCRIPT_DIR/with-sysroot.sh" pkg-config --libs --cflags webkit2gtk-4.1 gtk+-3.0 alsa jack >/dev/null
echo "[sysroot] Verification OK!"

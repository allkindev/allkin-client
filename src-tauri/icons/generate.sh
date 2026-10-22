#!/bin/sh
# Régénère les icônes de l'application depuis celles d'Allkin, sans les
# redessiner ni les recolorer :
#
#   src-tauri/icons/generate.sh <dossier web/ du projet allkin>
#
# Windows et Linux : icon-512.png d'Allkin telle quelle (et son icon-32.png,
# dont la marque est agrandie pour rester lisible). macOS : la même, posée
# dans la grille des icônes du système (carré arrondi de 824 px sur 1024,
# marges transparentes), sans quoi elle paraît plus grosse que les autres.
set -e
web=${1:?"usage: $0 <dossier web/ du projet allkin>"}
cd "$(dirname "$0")"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
icon() { npx --yes @tauri-apps/cli@2 icon "$1" -o "$2" > "$tmp/log" 2>&1 || { cat "$tmp/log"; exit 1; }; }

icon "$web/icon-512.png" "$tmp/all"
cp "$tmp/all/icon.png" "$tmp/all/128x128.png" "$tmp/all/128x128@2x.png" "$tmp/all/icon.ico" .
cp "$web/icon-32.png" 32x32.png

png=$(base64 < "$web/icon-512.png" | tr -d '\n')
cat > "$tmp/macos.svg" << EOF
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 1024 1024">
  <clipPath id="tile"><rect x="100" y="100" width="824" height="824" rx="185"/></clipPath>
  <image x="100" y="100" width="824" height="824" clip-path="url(#tile)" xlink:href="data:image/png;base64,$png"/>
</svg>
EOF
icon "$tmp/macos.svg" "$tmp/macos"
cp "$tmp/macos/icon.icns" .
echo "Icônes régénérées dans $(pwd)"

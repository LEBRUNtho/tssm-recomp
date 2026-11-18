#!/bin/zsh
# Builds the iPhone and iPad app from your own disc image, on a Mac with Xcode and Rust.
#
# usage: tools/ios.sh <disc image or extracted disc folder> [app|ipa|install|sim|simrun]
#   app      build and sign build/ios/<name>.app (default)
#   ipa      and pack it as build/ios/<name>.ipa, to install with a sideloading tool
#   install  and install it on the iPhone plugged in (devicectl)
#   sim      build for the simulator and install it on the simulator named $SIMNAME
#   simrun   and start it there
#
# The app carries the game's files (the extracted disc), so it is for your own devices only:
# never share an app or an .ipa built this way.
#
# Optional, as environment variables:
#   PACK=<zip>   a texture pack in Dolphin's format to package with the app (such as the Xbox
#                version's textures, https://gamebanana.com/mods/648590); packs can also be
#                put in the app's textures folder afterwards, in the Files app
#   ICON=<png>   a square picture for the app's icon, instead of the badge from the disc
#   APP_NAME=... the name under the icon
#
# Signing: without settings the app is signed ad hoc, for a sideloading tool that signs it
# again (AltStore, Sideloadly, Feather...). For your own certificate, put these in
# ~/.config/tssm-ios/signature.sh (kept out of the repository):
#   SIGN_IDENTITY="Apple Development: ..."   SIGN_TEAM=XXXXXXXXXX
#   SIGN_PROFILE=/path/to/profile.mobileprovision   BUNDLE_PREFIX=com.example
set -e
[ -f "$HOME/.config/tssm-ios/signature.sh" ] && source "$HOME/.config/tssm-ios/signature.sh"
BUNDLE_PREFIX=${BUNDLE_PREFIX:-com.example}
ROOT=${0:A:h:h}
SOURCE=${1:?disc image (.iso, .gcm) or extracted disc folder}
MODE=${2:-app}
OUT=${OUT:-$ROOT/build/ios}
MIN_IOS=15.0
LEGACY_SDK=26.0
cd "$ROOT"
# The repository pins the Windows toolchain; this build uses the Mac's.
export RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN:-stable-aarch64-apple-darwin}

echo "== Disc"
if [ -d "$SOURCE" ]; then
  DISC=${SOURCE:A}
else
  DISC=$OUT/disc-$(shasum "$SOURCE" | cut -c1-12)
  [ -f "$DISC/sys/main.dol" ] || cargo run -q -p recomp --release --bin extract -- "$SOURCE" "$DISC"
fi
ELF=$DISC/files/sb04gc_nm.elf
[ -f "$ELF" ] || { echo "!! this disc has no sb04gc_nm.elf: not a version the port can build"; exit 1; }
ID=$(head -c 6 "$DISC/sys/boot.bin")
REVISION=$(od -An -tu1 -j7 -N1 "$DISC/sys/boot.bin" | tr -d ' ')
SHA=$(shasum "$ELF" | cut -d' ' -f1)
case "$ID.$REVISION.$SHA" in
  GGVE78.1.2eff21ee1796b124c5200ed8cde09c4d7bc1f45b) VERSION=usa;;
  GGVX78.0.5714b1ba1e67aa9768323adf014598b02431a6c7) VERSION=eur;;
  *) echo "!! $ID revision $REVISION is not a version the port is made for (GGVE78 revision 1, GGVX78 revision 0)"; exit 1;;
esac
echo "   $ID revision $REVISION ($VERSION)"

echo "== Recompiling the game's code"
GEN=crates/game/src/gen
WANTED="$SHA $(cat config/hle.txt config/addresses.txt | shasum | cut -d' ' -f1)"
if [ "$(cat $GEN/.source 2>/dev/null)" != "$WANTED" ]; then
  mkdir -p $GEN
  cargo run -q -p recomp --release --bin recomp -- "$ELF" config/hle.txt $GEN $ID
  echo "$WANTED" > $GEN/.source
else
  echo "   up to date"
fi

if [[ $MODE = sim* ]]; then TARGET=aarch64-apple-ios-sim; else TARGET=aarch64-apple-ios; fi
echo "== Building ($TARGET)"
rustup target list --installed | grep -qx $TARGET || rustup target add $TARGET
IPHONEOS_DEPLOYMENT_TARGET=$MIN_IOS cargo build -p tssm --release --target $TARGET
BIN=${CARGO_TARGET_DIR:-$ROOT/target}/$TARGET/release/tssm

NAME=SpongeBobMovie
BUNDLE_ID=$BUNDLE_PREFIX.tssm.$VERSION
echo "== Package $NAME.app"
APP=$OUT/$TARGET/$NAME.app
rm -rf "$APP" && mkdir -p "$APP"
cp "$BIN" "$APP/tssm"
xcrun strip -x "$APP/tssm"
# The window library (winit 0.30) starts the app the way iOS did before scenes, which iOS 27
# refuses from a program built with its own SDK. Marked as built with the SDK before it, the
# app is started as older apps are.
if [[ $MODE = sim* ]]; then PLATFORM=iossim; else PLATFORM=ios; fi
xcrun vtool -set-build-version $PLATFORM $MIN_IOS $LEGACY_SDK -replace -output "$APP/tssm" "$APP/tssm" 2>/dev/null
sed -e "s/@BUNDLE_ID@/$BUNDLE_ID/" -e "s/@MIN_IOS@/$MIN_IOS/" tools/ios/Info.plist > "$APP/Info.plist"
# The name under the icon, unless given. iOS shows about a dozen characters whole.
[ -n "$APP_NAME" ] && plutil -replace CFBundleDisplayName -string "$APP_NAME" "$APP/Info.plist"
# The icon is the game's own badge, taken from the disc, unless one is given.
if [ -n "$ICON" ]; then cp "$ICON" "$OUT/icon-$VERSION.png"; else python3 tools/ios/icon.py "$DISC" "$OUT/icon-$VERSION.png"; fi
if [ -f "$OUT/icon-$VERSION.png" ]; then
  for icon in "AppIcon60x60@2x 120" "AppIcon60x60@3x 180" "AppIcon76x76@2x~ipad 152" "AppIcon83.5x83.5@2x~ipad 167"; do
    sips -z ${icon#* } ${icon#* } "$OUT/icon-$VERSION.png" --out "$APP/${icon% *}.png" >/dev/null
  done
else
  echo "   (no icon)"
fi
plutil -convert binary1 "$APP/Info.plist"
# The game's files, without the copy of its code that only the build reads.
rsync -a --delete --exclude '/files/sb04gc_nm.elf' "$DISC/" "$APP/disc/"
if [ -n "$PACK" ]; then
  mkdir -p "$APP/textures" && cp "$PACK" "$APP/textures/"
  echo "   with the texture pack ${PACK:t}"
fi

echo "== Signing"
if [[ $MODE = sim* ]]; then
  codesign --force --sign - "$APP" >/dev/null 2>&1
  SIMNAME=${SIMNAME:-iPhone TSSM}
  SIMTYPE=${SIMTYPE:-com.apple.CoreSimulator.SimDeviceType.iPhone-17-Pro}
  UDID=$(xcrun simctl list devices | awk -F'[()]' -v n="$SIMNAME" 'index($0,n" (") {print $2; exit}')
  if [ -z "$UDID" ]; then
    RUNTIME=$(xcrun simctl list runtimes | awk '/iOS/ {id=$NF} END {print id}')
    UDID=$(xcrun simctl create "$SIMNAME" $SIMTYPE $RUNTIME)
  fi
  xcrun simctl boot $UDID 2>/dev/null || true
  xcrun simctl install $UDID "$APP"
  echo "Simulator $UDID ready ($BUNDLE_ID)"
  [ "$MODE" = simrun ] && xcrun simctl launch $UDID $BUNDLE_ID
  exit 0
fi
if [ -n "$SIGN_IDENTITY" ]; then
  security find-identity -v -p codesigning | grep -q "$SIGN_IDENTITY" || { echo "!! the certificate is not in the keychain"; exit 1; }
  cp "$SIGN_PROFILE" "$APP/embedded.mobileprovision"
  cat > "$OUT/entitlements.plist" <<EOP
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>application-identifier</key><string>$SIGN_TEAM.$BUNDLE_ID</string>
  <key>com.apple.developer.team-identifier</key><string>$SIGN_TEAM</string>
  <key>get-task-allow</key><true/>
</dict></plist>
EOP
  codesign --force --sign "$SIGN_IDENTITY" --timestamp=none --entitlements "$OUT/entitlements.plist" "$APP"
else
  echo "   (no certificate set: signed ad hoc, for a sideloading tool to sign again)"
  codesign --force --sign - --timestamp=none "$APP"
fi
codesign --verify --verbose=1 "$APP"
echo "== App ready: $APP ($(du -sh "$APP" | cut -f1))"

if [ "$MODE" = ipa ] || [ "$MODE" = install ]; then
  STAGE=$(mktemp -d); mkdir -p $STAGE/Payload
  ditto --norsrc --noextattr "$APP" $STAGE/Payload/$NAME.app
  IPA=$OUT/$NAME-$VERSION.ipa; rm -f "$IPA"
  # The disc's files are already compressed: stored, the package is made much faster.
  (cd $STAGE && zip -qry -X -1 "$IPA" Payload); rm -rf $STAGE
  echo "== IPA: $IPA ($(du -h "$IPA" | cut -f1))"
fi
if [ "$MODE" = install ]; then
  DEVICE=$(xcrun devicectl list devices 2>/dev/null | awk '/connected/ {print $3; exit}')
  [ -n "$DEVICE" ] || { echo "!! no iPhone connected"; exit 1; }
  xcrun devicectl device install app --device $DEVICE "$APP"
fi

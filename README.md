# Taskify
Taskify is a FOSS product for tracking and creating tasks with a click of a button

## Web App
Supported platforms (not all have been tested):
 - Firefox
 - Chromium
Run `build.sh` (Ubuntu only)
Build command:
`cargo run --release --bin taskify_web`

## Browser Extension
Supported platforms (not all have been tested):
 - Firefox
 - Chromium
Run `build.sh` (Ubuntu only)

## Desktop
Supported platforms (not all have been tested):
 - Mac x86_64
 - Mac arm64
 - Windows x86_64
 - Windows arm64
 - Linux x86_64
 - Linux arm64
Build command:
`cargo run --release --bin taskify_desktop`

## Android
taskify_android (Ubuntu script):
```
# install sdk ndk tools
sudo apt update
sudo apt install -y openjdk-17-jdk adb fastboot android-sdk \
  google-android-cmdline-tools-13.0-installer \
  google-android-ndk-r26c-installer \
  android-sdk-platform-tools

# shell env
export ANDROID_HOME=/usr/lib/android-sdk
export ANDROID_NDK_HOME="$(find "$ANDROID_HOME/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n1)"
export ANDROID_CMDLINE_TOOLS="$(find "$ANDROID_HOME/cmdline-tools" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n1)"
export PATH=$PATH:$ANDROID_HOME/platform-tools:$ANDROID_CMDLINE_TOOLS/bin
unset ANDROID_SDK_ROOT

# install platform packages
yes | sdkmanager --licenses
sudo --preserve-env=ANDROID_HOME,ANDROID_NDK_HOME,ANDROID_CMDLINE_TOOLS,PATH \
  sdkmanager "platform-tools" "platforms;android-35" "build-tools;35.0.0"

# install rust target
rustup target add armv7-linux-androideabi aarch64-linux-android

# build apk
cargo apk build -p taskify_android --lib

# install apk on connected device
adb devices
adb install -r target/debug/apk/taskify_android.apk

# or run locally
cargo apk run -p taskify_android --lib
```
Enable USB debugging on phone
```
Settings -> About phone -> tap Build number 7 times
Settings -> Developer options -> enable USB debugging
Connect phone via USB and accept RSA prompt
adb devices
```

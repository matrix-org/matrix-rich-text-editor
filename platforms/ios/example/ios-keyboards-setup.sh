#!/bin/bash

# Get the list of available simulators
simulators=$(xcrun simctl list devices 17.2)

# Find the current booted simulator
test_simulator=$(echo "$simulators" | grep "iPhone 15 (" | awk -F '[()]' '{print $2}')

echo "Found simulator: $test_simulator"

# Define the path to the preferences plist file for the booted simulator
plist_path=~/Library/Developer/CoreSimulator/Devices/$test_simulator/data/Library/Preferences/.GlobalPreferences.plist
echo "Path: $plist_path"

# Check if the plist file exists
if [ ! -f "$plist_path" ]; then
  echo "Preferences plist file not found: $plist_path"
  exit 1
fi

# Clear the current keyboard layouts
/usr/libexec/PlistBuddy -c "Delete :AppleKeyboards" "$plist_path" 2>/dev/null
/usr/libexec/PlistBuddy -c "Add :AppleKeyboards array" "$plist_path"
/usr/libexec/PlistBuddy -c "Add :AppleKeyboards:0 string 'en_US@sw=QWERTY;hw=Automatic'" "$plist_path"
/usr/libexec/PlistBuddy -c "Add :AppleKeyboards:1 string 'ja_JP-Kana@sw=Kana;hw=Automatic'" "$plist_path"

/usr/libexec/PlistBuddy -c "Print :AppleKeyboards array" "$plist_path" 2>/dev/null

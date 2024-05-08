#!/usr/bin/env bash

APP_NAME="DTB Kampfrichtereinsatzpläne.app"
DMG_FILE_NAME="DTB Kampfrichtereinsatzpläne.dmg"
VOLUME_NAME="DTB Kampfrichtereinsatzpläne"
SOURCE_FOLDER=$(find ./target -maxdepth 5 -name "macos" | head -n 1)
BACKGROUND_IMAGE="../installer/dmgImage.tiff"
WINDOW_W=720
WINDOW_H=590
WINDOW_X=0
WINDOW_Y=0
EULA_PATH="../installer/LICENSE.rtf"
ICON_SIZE=128
VOLUME_ICON="../installer/installerIcon.icns"
APPLICATION_FOLDER_ICON_X=525
APPLICATION_FOLDER_ICON_Y=270
TEXT_SIZE=10

if [[ -e ../externals/create-dmg/create-dmg ]]; then
  CREATE_DMG=../externals/create-dmg/create-dmg
else
   printf "%s\n Are you sure you checked out all submodules?" "create-dmg was not found, but expected at: $(readlink -f ../externals/create-dmg/create-dmg)"
fi

$CREATE_DMG \
  --volname "${VOLUME_NAME}" \
  --volicon "${VOLUME_ICON}" \
  --background "${BACKGROUND_IMAGE}" \
  --window-pos "${WINDOW_X}" "${WINDOW_Y}" \
  --window-size "${WINDOW_W}" "${WINDOW_H}" \
  --icon-size "${ICON_SIZE}" \
  --app-drop-link "${APPLICATION_FOLDER_ICON_X}" "${APPLICATION_FOLDER_ICON_Y}" \
  --text-size "${TEXT_SIZE}" \
  --eula "${EULA_PATH}" \
  --icon "${APP_NAME}" 205 270 \
  --hide-extension "${APP_NAME}" \
  "${DMG_FILE_NAME}" \
  "${SOURCE_FOLDER}"

first_dmg=$(find ./target -maxdepth 5 -type f -name "*.dmg" | head -n 1)

mv "./${DMG_FILE_NAME}" "${first_dmg}"
rm -f "./${DMG_FILE_NAME}"
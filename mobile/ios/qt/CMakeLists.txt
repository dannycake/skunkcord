cmake_minimum_required(VERSION 3.21)
project(DiscordQt LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_AUTOMOC ON)

# iOS-specific
set(CMAKE_OSX_ARCHITECTURES "arm64" CACHE STRING "")
set(CMAKE_OSX_DEPLOYMENT_TARGET "16.0" CACHE STRING "")
if(DEFINED APPLE_TEAM_ID AND NOT "${APPLE_TEAM_ID}" STREQUAL "")
    set(CMAKE_XCODE_ATTRIBUTE_CODE_SIGN_IDENTITY "Apple Distribution" CACHE STRING "")
    set(CMAKE_XCODE_ATTRIBUTE_CODE_SIGN_STYLE "Manual" CACHE STRING "")
    set(CMAKE_XCODE_ATTRIBUTE_DEVELOPMENT_TEAM "${APPLE_TEAM_ID}" CACHE STRING "")
    if(DEFINED PROVISIONING_PROFILE_SPECIFIER)
        set(CMAKE_XCODE_ATTRIBUTE_PROVISIONING_PROFILE_SPECIFIER
            "${PROVISIONING_PROFILE_SPECIFIER}" CACHE STRING "")
    endif()
else()
    # No signing identity — allow unsigned builds (CI without secrets)
    set(CMAKE_XCODE_ATTRIBUTE_CODE_SIGNING_ALLOWED "NO" CACHE STRING "")
    set(CMAKE_XCODE_ATTRIBUTE_CODE_SIGN_IDENTITY "" CACHE STRING "")
endif()

# Rust static library (built separately)
if(NOT DEFINED RUST_LIB)
    message(FATAL_ERROR "RUST_LIB must be set to path of libdiscord_qt.a")
endif()

# Qt for iOS
find_package(Qt6 REQUIRED COMPONENTS Core Quick Gui Qml)

# Repo root (for include/ and src/qml); shared C++ (main.cpp, AppController)
set(DISCORD_QT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../..")
set(SHARED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../shared")

# Executable (iOS app bundle)
qt_add_executable(DiscordQt
    ${SHARED_DIR}/main.cpp
    ${SHARED_DIR}/AppController.cpp
    ${SHARED_DIR}/AppController.h
)

target_include_directories(DiscordQt PRIVATE "${DISCORD_QT_ROOT}/include" "${SHARED_DIR}")

target_link_libraries(DiscordQt PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::Gui
    Qt6::Qml
    "${RUST_LIB}"
)

# Link system libraries required by Rust on iOS
target_link_libraries(DiscordQt PRIVATE
    "-framework Security"
    "-framework Foundation"
    "-framework SystemConfiguration"
    "-lc++"
)

# Bundle QML files as resources
# Qt requires QT_RESOURCE_ALIAS for files outside the source directory
set(QML_SOURCE_DIR "${DISCORD_QT_ROOT}/src/qml")

set(MAIN_QML "${QML_SOURCE_DIR}/main.qml")
set_source_files_properties(${MAIN_QML} PROPERTIES QT_RESOURCE_ALIAS "main.qml")
set(MOBILE_QML "${QML_SOURCE_DIR}/mobile.qml")
set_source_files_properties(${MOBILE_QML} PROPERTIES QT_RESOURCE_ALIAS "mobile.qml")
file(GLOB QML_JS "${QML_SOURCE_DIR}/*.js")
foreach(_js IN LISTS QML_JS)
    get_filename_component(_name "${_js}" NAME)
    set_source_files_properties("${_js}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
endforeach()
qt_add_resources(DiscordQt "qml_resources"
    PREFIX "/qml"
    FILES ${MAIN_QML} ${MOBILE_QML} ${QML_JS}
)

# Add component QML files so main.qml can load them
file(GLOB QML_COMPONENTS "${QML_SOURCE_DIR}/components/*.qml")
foreach(_qml IN LISTS QML_COMPONENTS)
    get_filename_component(_name "${_qml}" NAME)
    set_source_files_properties("${_qml}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
endforeach()
qt_add_resources(DiscordQt "qml_components"
    PREFIX "/qml/components"
    FILES ${QML_COMPONENTS}
)

# Info.plist and bundle identifier (must match App ID in provisioning profile)
set_target_properties(DiscordQt PROPERTIES
    MACOSX_BUNDLE TRUE
    MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/Info.plist"
    XCODE_ATTRIBUTE_PRODUCT_BUNDLE_IDENTIFIER "ink.danny.discordqt"
)

# QML import path so "import components" works
set_target_properties(DiscordQt PROPERTIES
    QT_QML_IMPORT_PATH "${QML_SOURCE_DIR}"
)

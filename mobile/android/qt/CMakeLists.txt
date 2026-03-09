cmake_minimum_required(VERSION 3.21)
project(Skunkcord LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_AUTOMOC ON)

# Rust shared library (built separately for Android ABI)
if(NOT DEFINED RUST_LIB)
    message(FATAL_ERROR "RUST_LIB must be set to path of libskunkcord.so for the target ABI")
endif()

# Qt for Android
find_package(Qt6 REQUIRED COMPONENTS Core Quick Gui Qml)

# Repo root (for include/ and src/qml); shared C++ (main.cpp, AppController)
set(SKUNKCORD_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../..")
set(SHARED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../shared")

# Executable (on Android this becomes a shared library that Qt loads)
qt_add_executable(Skunkcord
    ${SHARED_DIR}/main.cpp
    ${SHARED_DIR}/AppController.cpp
    ${SHARED_DIR}/AppController.h
)

target_include_directories(Skunkcord PRIVATE "${SKUNKCORD_ROOT}/include" "${SHARED_DIR}")

target_link_libraries(Skunkcord PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::Gui
    Qt6::Qml
    "${RUST_LIB}"
)

# Android package source (manifest, etc.)
set_target_properties(Skunkcord PROPERTIES
    QT_ANDROID_PACKAGE_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/android-sources"
    QT_ANDROID_PACKAGE_NAME "ink.danny.skunkcord"
    QT_ANDROID_APP_NAME "Skunkcord"
)

# Bundle QML files as resources (same pattern as iOS)
set(QML_SOURCE_DIR "${SKUNKCORD_ROOT}/src/qml")

set(MAIN_QML "${QML_SOURCE_DIR}/main.qml")
set_source_files_properties(${MAIN_QML} PROPERTIES QT_RESOURCE_ALIAS "main.qml")
set(MOBILE_QML "${QML_SOURCE_DIR}/mobile.qml")
set_source_files_properties(${MOBILE_QML} PROPERTIES QT_RESOURCE_ALIAS "mobile.qml")
file(GLOB QML_JS "${QML_SOURCE_DIR}/*.js")
foreach(_js IN LISTS QML_JS)
    get_filename_component(_name "${_js}" NAME)
    set_source_files_properties("${_js}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
endforeach()
qt_add_resources(Skunkcord "qml_resources"
    PREFIX "/qml"
    FILES ${MAIN_QML} ${MOBILE_QML} ${QML_JS}
)

file(GLOB QML_COMPONENTS "${QML_SOURCE_DIR}/components/*.qml")
foreach(_qml IN LISTS QML_COMPONENTS)
    get_filename_component(_name "${_qml}" NAME)
    set_source_files_properties("${_qml}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
endforeach()
qt_add_resources(Skunkcord "qml_components"
    PREFIX "/qml/components"
    FILES ${QML_COMPONENTS}
)

# QML import path so "import components" works
set_target_properties(Skunkcord PROPERTIES
    QT_QML_IMPORT_PATH "${QML_SOURCE_DIR}"
)

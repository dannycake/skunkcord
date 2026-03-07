cmake_minimum_required(VERSION 3.21)
project(DiscordQtMobileTest LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_AUTOMOC ON)

# Rust shared library (built separately: cargo build --release --lib --no-default-features --features mobile)
if(NOT DEFINED RUST_LIB)
    message(FATAL_ERROR "RUST_LIB must be set to path of libdiscord_qt.so (e.g. ../../target/release/libdiscord_qt.so)")
endif()

# Use Qt6 if available, fall back to Qt5 (desktop app uses Qt5 successfully)
find_package(Qt6 COMPONENTS Core Quick Gui Qml QuickControls2 Widgets WebEngineWidgets QUIET)
if(NOT Qt6_FOUND)
    message(STATUS "Qt6 not found or incomplete, using Qt5")
    find_package(Qt5 REQUIRED COMPONENTS Core Quick Gui Qml)
    find_package(Qt5 COMPONENTS Widgets WebEngine QUIET)
    set(QT_VERSION_MAJOR 5)
else()
    set(QT_VERSION_MAJOR 6)
endif()

# Repo root (discord-qt); shared C++ is in ../shared
set(DISCORD_QT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(SHARED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../shared")

# Create executable (Qt5 and Qt6 differ in how resources are added)
if(QT_VERSION_MAJOR EQUAL 6)
    qt_add_executable(discord_qt_mobile_test
        ${SHARED_DIR}/main.cpp
        ${SHARED_DIR}/AppController.cpp
        ${SHARED_DIR}/AppController.h
    )
else()
    add_executable(discord_qt_mobile_test
        ${SHARED_DIR}/main.cpp
        ${SHARED_DIR}/AppController.cpp
        ${SHARED_DIR}/AppController.h
    )
endif()

target_include_directories(discord_qt_mobile_test PRIVATE "${DISCORD_QT_ROOT}/include" "${SHARED_DIR}")

if(QT_VERSION_MAJOR EQUAL 6)
    target_link_libraries(discord_qt_mobile_test PRIVATE
        Qt6::Core Qt6::Quick Qt6::Gui Qt6::Qml Qt6::QuickControls2
        "${RUST_LIB}"
    )
    if(TARGET Qt6::Widgets)
        target_link_libraries(discord_qt_mobile_test PRIVATE Qt6::Widgets)
    endif()
    if(TARGET Qt6::WebEngineWidgets)
        target_link_libraries(discord_qt_mobile_test PRIVATE Qt6::WebEngineWidgets)
    endif()
else()
    # Qt5
    target_link_libraries(discord_qt_mobile_test PRIVATE
        Qt5::Core Qt5::Quick Qt5::Gui Qt5::Qml
        "${RUST_LIB}"
    )
    if(TARGET Qt5::Widgets)
        target_link_libraries(discord_qt_mobile_test PRIVATE Qt5::Widgets)
    endif()
    if(TARGET Qt5::WebEngine)
        target_link_libraries(discord_qt_mobile_test PRIVATE Qt5::WebEngine)
    endif()
endif()

# Bundle QML as resources (same as iOS/Android)
set(QML_SOURCE_DIR "${DISCORD_QT_ROOT}/src/qml")

if(QT_VERSION_MAJOR EQUAL 6)
    # Qt6: use qt_add_resources with QT_RESOURCE_ALIAS
    set(MAIN_QML "${QML_SOURCE_DIR}/main.qml")
    set_source_files_properties(${MAIN_QML} PROPERTIES QT_RESOURCE_ALIAS "main.qml")
    set(MOBILE_QML "${QML_SOURCE_DIR}/mobile.qml")
    set_source_files_properties(${MOBILE_QML} PROPERTIES QT_RESOURCE_ALIAS "mobile.qml")
    file(GLOB QML_JS "${QML_SOURCE_DIR}/*.js")
    foreach(_js IN LISTS QML_JS)
        get_filename_component(_name "${_js}" NAME)
        set_source_files_properties("${_js}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
    endforeach()
    qt_add_resources(discord_qt_mobile_test "qml_resources"
        PREFIX "/qml"
        FILES ${MAIN_QML} ${MOBILE_QML} ${QML_JS}
    )
    file(GLOB QML_COMPONENTS "${QML_SOURCE_DIR}/components/*.qml")
    foreach(_qml IN LISTS QML_COMPONENTS)
        get_filename_component(_name "${_qml}" NAME)
        set_source_files_properties("${_qml}" PROPERTIES QT_RESOURCE_ALIAS "${_name}")
    endforeach()
    qt_add_resources(discord_qt_mobile_test "qml_components"
        PREFIX "/qml/components"
        FILES ${QML_COMPONENTS}
    )
else()
    # Qt5: use qt5_add_resources
    set(MAIN_QML "${QML_SOURCE_DIR}/main.qml")
    file(GLOB QML_COMPONENTS "${QML_SOURCE_DIR}/components/*.qml")
    file(GLOB QML_JS "${QML_SOURCE_DIR}/*.js")
    # Create a qrc file programmatically
    set(QRC_FILE "${CMAKE_CURRENT_BINARY_DIR}/qml_resources.qrc")
    set(MOBILE_QML "${QML_SOURCE_DIR}/mobile.qml")
    file(WRITE ${QRC_FILE} "<RCC>\n  <qresource prefix=\"/qml\">\n")
    file(APPEND ${QRC_FILE} "    <file alias=\"main.qml\">${MAIN_QML}</file>\n")
    file(APPEND ${QRC_FILE} "    <file alias=\"mobile.qml\">${MOBILE_QML}</file>\n")
    foreach(_js IN LISTS QML_JS)
        get_filename_component(_name "${_js}" NAME)
        file(APPEND ${QRC_FILE} "    <file alias=\"${_name}\">${_js}</file>\n")
    endforeach()
    file(APPEND ${QRC_FILE} "  </qresource>\n")
    file(APPEND ${QRC_FILE} "  <qresource prefix=\"/qml/components\">\n")
    foreach(_qml IN LISTS QML_COMPONENTS)
        get_filename_component(_name "${_qml}" NAME)
        file(APPEND ${QRC_FILE} "    <file alias=\"${_name}\">${_qml}</file>\n")
    endforeach()
    file(APPEND ${QRC_FILE} "  </qresource>\n</RCC>\n")
    qt5_add_resources(QML_RCC ${QRC_FILE})
    target_sources(discord_qt_mobile_test PRIVATE ${QML_RCC})
endif()

set_target_properties(discord_qt_mobile_test PROPERTIES
    QT_QML_IMPORT_PATH "${QML_SOURCE_DIR}"
)

DEFINES += HELLOWORLD_LIBRARY

SOURCES += "$$PWD/src/*.cpp"
HEADERS += "$$PWD/include/*.h"

INCLUDEPATH += "$$PWD/include"

IDE_SOURCE_TREE = "$$PWD/../deps/qt-creator"
IDE_BUILD_TREE  = "$$PWD/../deps/qt-creator/build"

QTC_PLUGIN_NAME = HelloWorldPlugin
QTC_LIB_DEPENDS += \
    # nothing here at this time

QTC_PLUGIN_DEPENDS += \
    coreplugin

QTC_PLUGIN_RECOMMENDS += \
    # optional plugin dependencies. nothing here at this time

include($$IDE_SOURCE_TREE/src/qtcreatorplugin.pri)

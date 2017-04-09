#pragma once

#include <QtGlobal>

#if defined(HELLOWORLD_LIBRARY)
#  define HELLOWORLDPLUGINSHARED_EXPORT Q_DECL_EXPORT
#else
#  define HELLOWORLDSHARED_EXPORT Q_DECL_IMPORT
#endif

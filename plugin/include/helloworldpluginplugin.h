#pragma once

#include "helloworldplugin_global.h"

#include <extensionsystem/iplugin.h>

namespace HelloWorldPlugin {
namespace Internal {

class HelloWorldPluginPlugin : public ExtensionSystem::IPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID "org.qt-project.Qt.QtCreatorPlugin" FILE "HelloWorldPlugin.json")

public:
    HelloWorldPluginPlugin();
    ~HelloWorldPluginPlugin();

    bool initialize(const QStringList &arguments, QString *errorString);
    void extensionsInitialized();
    ShutdownFlag aboutToShutdown();

private:
    void triggerAction();
};

} // namespace Internal
} // namespace HelloWorldPlugin

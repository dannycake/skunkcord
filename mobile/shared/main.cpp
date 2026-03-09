#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include "AppController.h"

extern "C" int skunkcord_init(void);

int main(int argc, char *argv[])
{
    if (skunkcord_init() != 0)
        return -1;

    QGuiApplication app(argc, argv);

    QQmlApplicationEngine engine;
    AppController controller;
    engine.rootContext()->setContextProperty("app", &controller);

    const QUrl qmlUrl(QStringLiteral("qrc:/qml/mobile.qml"));
#if QT_VERSION >= QT_VERSION_CHECK(6, 0, 0)
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection);
#endif
    engine.load(qmlUrl);

    if (engine.rootObjects().isEmpty())
        return -1;

    return app.exec();
}

// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once

#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickWindow>

// A bar docked at the left or right edge of its screen. On Windows it
// registers as an application desktop toolbar (an "app bar", like the
// taskbar), so maximized windows end where it begins. Expanding it widens
// the window over the desktop while the reserved strip stays `thickness`.
class DockWindow : public QQuickWindow
{
    Q_OBJECT
    QML_ELEMENT

    // "left" or "right".
    Q_PROPERTY(QString edge READ edge WRITE setEdge NOTIFY edgeChanged)
    // Width of the reserved strip, in logical pixels.
    Q_PROPERTY(int thickness READ thickness WRITE setThickness NOTIFY thicknessChanged)
    // Width while expanded, in logical pixels.
    Q_PROPERTY(int expandedWidth READ expandedWidth WRITE setExpandedWidth NOTIFY expandedWidthChanged)
    Q_PROPERTY(bool expanded READ expanded WRITE setExpanded NOTIFY expandedChanged)
    // Hide in a thin strip at the edge until the pointer comes (`revealed`),
    // without reserving space.
    Q_PROPERTY(bool autoHide READ autoHide WRITE setAutoHide NOTIFY autoHideChanged)
    Q_PROPERTY(bool revealed READ revealed WRITE setRevealed NOTIFY revealedChanged)
    // Width of the strip while hidden, in logical pixels.
    Q_PROPERTY(int peek READ peek CONSTANT)
    // Height of the window, centered on the edge, in logical pixels; 0 or
    // more than the screen has fills the edge.
    Q_PROPERTY(int contentHeight READ contentHeight WRITE setContentHeight NOTIFY contentHeightChanged)
    // Whether another app is full screen on the dock's monitor (the dock
    // steps back behind it meanwhile).
    Q_PROPERTY(bool fullScreenApp READ fullScreenApp NOTIFY fullScreenAppChanged)

public:
    explicit DockWindow(QWindow *parent = nullptr);
    ~DockWindow() override;

    QString edge() const { return m_edge; }
    void setEdge(const QString &edge);
    int thickness() const { return m_thickness; }
    void setThickness(int thickness);
    int expandedWidth() const { return m_expandedWidth; }
    void setExpandedWidth(int width);
    bool expanded() const { return m_expanded; }
    void setExpanded(bool expanded);
    bool fullScreenApp() const { return m_fullScreenApp; }
    bool autoHide() const { return m_autoHide; }
    void setAutoHide(bool on);
    bool revealed() const { return m_revealed; }
    void setRevealed(bool on);
    int peek() const { return 6; }
    int contentHeight() const { return m_contentHeight; }
    void setContentHeight(int height);

    // Places the window again (after a screen or work area change).
    Q_INVOKABLE void relayout();

signals:
    void edgeChanged();
    void thicknessChanged();
    void expandedWidthChanged();
    void expandedChanged();
    void fullScreenAppChanged();
    void autoHideChanged();
    void revealedChanged();
    void contentHeightChanged();

protected:
    bool event(QEvent *event) override;
    bool nativeEvent(const QByteArray &eventType, void *message, qintptr *result) override;

private:
    void registerBar();
    void unregisterBar();
    bool leftEdge() const { return m_edge == QLatin1String("left"); }

    QString m_edge = QStringLiteral("right");
    int m_thickness = 60;
    int m_expandedWidth = 380;
    bool m_expanded = false;
    bool m_fullScreenApp = false;
    bool m_registered = false;
    bool m_autoHide = false;
    bool m_revealed = false;
    int m_contentHeight = 0;
};

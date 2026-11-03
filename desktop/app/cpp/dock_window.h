// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once

#include <QtCore/QTimer>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickWindow>

// A compact bar floating over other windows at the left or right edge of its
// screen, centered vertically, without taking space from them. The window
// keeps its size while it hides (resizing a translucent window is slow):
// tucked away, only a thin strip at the edge takes the pointer and clicks
// elsewhere go through to the windows below.
class DockWindow : public QQuickWindow
{
    Q_OBJECT
    QML_ELEMENT

    // "left" or "right".
    Q_PROPERTY(QString edge READ edge WRITE setEdge NOTIFY edgeChanged)
    // Width of the bar, in logical pixels.
    Q_PROPERTY(int thickness READ thickness WRITE setThickness NOTIFY thicknessChanged)
    // Width while expanded, in logical pixels.
    Q_PROPERTY(int expandedWidth READ expandedWidth WRITE setExpandedWidth NOTIFY expandedWidthChanged)
    Q_PROPERTY(bool expanded READ expanded WRITE setExpanded NOTIFY expandedChanged)
    // Height of the window, centered on the edge, in logical pixels; 0 or
    // more than the screen has fills the edge.
    Q_PROPERTY(int contentHeight READ contentHeight WRITE setContentHeight NOTIFY contentHeightChanged)
    // Tucked away at the edge until the pointer comes (`revealed`).
    Q_PROPERTY(bool autoHide READ autoHide WRITE setAutoHide NOTIFY autoHideChanged)
    Q_PROPERTY(bool revealed READ revealed WRITE setRevealed NOTIFY revealedChanged)
    // Width of the strip that takes the pointer while tucked away.
    Q_PROPERTY(int peek READ peek CONSTANT)
    // Whether a full-screen app, game or presentation is in front (the dock
    // steps back behind it meanwhile).
    Q_PROPERTY(bool fullScreenApp READ fullScreenApp NOTIFY fullScreenAppChanged)

public:
    explicit DockWindow(QWindow *parent = nullptr);

    QString edge() const { return m_edge; }
    void setEdge(const QString &edge);
    int thickness() const { return m_thickness; }
    void setThickness(int thickness);
    int expandedWidth() const { return m_expandedWidth; }
    void setExpandedWidth(int width);
    bool expanded() const { return m_expanded; }
    void setExpanded(bool expanded);
    int contentHeight() const { return m_contentHeight; }
    void setContentHeight(int height);
    bool autoHide() const { return m_autoHide; }
    void setAutoHide(bool on);
    bool revealed() const { return m_revealed; }
    void setRevealed(bool on);
    int peek() const { return 6; }
    bool fullScreenApp() const { return m_fullScreenApp; }

    // Places the window again (after a screen or work area change).
    Q_INVOKABLE void relayout();

signals:
    void edgeChanged();
    void thicknessChanged();
    void expandedWidthChanged();
    void expandedChanged();
    void contentHeightChanged();
    void autoHideChanged();
    void revealedChanged();
    void fullScreenAppChanged();

private:
    void updateMask();
    void checkFullScreen();
    bool leftEdge() const { return m_edge == QLatin1String("left"); }
    bool tucked() const { return m_autoHide && !m_revealed && !m_expanded; }

    QString m_edge = QStringLiteral("right");
    int m_thickness = 64;
    int m_expandedWidth = 420;
    bool m_expanded = false;
    int m_contentHeight = 0;
    bool m_autoHide = false;
    bool m_revealed = false;
    bool m_fullScreenApp = false;
    QTimer m_fullScreenTimer;
};

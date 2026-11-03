// SPDX-License-Identifier: GPL-3.0-or-later
#include "dock_window.h"

#include <QtGui/QRegion>
#include <QtGui/QScreen>

#include <algorithm>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <shellapi.h>
#endif

DockWindow::DockWindow(QWindow *parent)
    : QQuickWindow(parent)
{
    setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint);
    connect(this, &QWindow::screenChanged, this, [this](QScreen *) { relayout(); });
    connect(this, &QWindow::visibleChanged, this, [this](bool visible) {
        if (visible)
            relayout();
    });
    // Windows tells whether something runs full screen only when asked.
    m_fullScreenTimer.setInterval(2000);
    connect(&m_fullScreenTimer, &QTimer::timeout, this, &DockWindow::checkFullScreen);
    m_fullScreenTimer.start();
}

void DockWindow::setEdge(const QString &edge)
{
    const QString value = edge == QLatin1String("left") ? QStringLiteral("left") : QStringLiteral("right");
    if (value == m_edge)
        return;
    m_edge = value;
    emit edgeChanged();
    relayout();
}

void DockWindow::setThickness(int thickness)
{
    thickness = std::max(40, thickness);
    if (thickness == m_thickness)
        return;
    m_thickness = thickness;
    emit thicknessChanged();
    relayout();
}

void DockWindow::setExpandedWidth(int width)
{
    if (width == m_expandedWidth)
        return;
    m_expandedWidth = width;
    emit expandedWidthChanged();
    if (m_expanded)
        relayout();
}

void DockWindow::setExpanded(bool expanded)
{
    if (expanded == m_expanded)
        return;
    m_expanded = expanded;
    emit expandedChanged();
    relayout();
}

void DockWindow::setContentHeight(int height)
{
    height = std::max(0, height);
    if (height == m_contentHeight)
        return;
    m_contentHeight = height;
    emit contentHeightChanged();
    relayout();
}

void DockWindow::setAutoHide(bool on)
{
    if (on == m_autoHide)
        return;
    m_autoHide = on;
    emit autoHideChanged();
    updateMask();
}

void DockWindow::setRevealed(bool on)
{
    if (on == m_revealed)
        return;
    m_revealed = on;
    emit revealedChanged();
    // Only what takes the pointer changes, not the window.
    updateMask();
}

void DockWindow::relayout()
{
    QScreen *s = screen();
    if (!s || !isVisible())
        return;
    const int wide = m_expanded ? std::max(m_thickness, m_expandedWidth) : m_thickness;
    const QRect area = s->availableGeometry();
    const int tall = m_contentHeight > 0 ? std::min(m_contentHeight, area.height()) : area.height();
    const int x = leftEdge() ? area.left() : area.right() + 1 - wide;
    setGeometry(x, area.top() + (area.height() - tall) / 2, wide, tall);
    updateMask();
}

void DockWindow::updateMask()
{
    if (tucked()) {
        const int x = leftEdge() ? 0 : width() - peek();
        setMask(QRegion(x, 0, peek(), height()));
    } else {
        setMask(QRegion());
    }
}

void DockWindow::checkFullScreen()
{
    bool full = false;
#ifdef _WIN32
    QUERY_USER_NOTIFICATION_STATE state {};
    if (SUCCEEDED(SHQueryUserNotificationState(&state)))
        full = state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_PRESENTATION_MODE;
#endif
    if (full == m_fullScreenApp)
        return;
    m_fullScreenApp = full;
    emit fullScreenAppChanged();
}

// SPDX-License-Identifier: GPL-3.0-or-later
#include "dock_window.h"

#include <QtGui/QPlatformSurfaceEvent>
#include <QtGui/QScreen>

#include <algorithm>
#include <cmath>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <shellapi.h>
#endif

namespace {

#ifdef _WIN32
// The message Windows sends the app bar about position changes and full
// screen apps.
UINT callbackMessage()
{
    static const UINT message = RegisterWindowMessageW(L"NectarLinkDockAppBar");
    return message;
}

HWND hwndOf(const QWindow *window)
{
    return reinterpret_cast<HWND>(window->winId());
}
#endif

}  // namespace

DockWindow::DockWindow(QWindow *parent)
    : QQuickWindow(parent)
{
    setFlags(Qt::Tool | Qt::FramelessWindowHint | Qt::WindowStaysOnTopHint);
    connect(this, &QWindow::screenChanged, this, [this](QScreen *) { relayout(); });
    connect(this, &QWindow::visibleChanged, this, [this](bool visible) {
        if (visible) {
            if (!m_autoHide)
                registerBar();
            relayout();
        } else {
            unregisterBar();
        }
    });
}

DockWindow::~DockWindow()
{
    unregisterBar();
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

void DockWindow::setAutoHide(bool on)
{
    if (on == m_autoHide)
        return;
    m_autoHide = on;
    emit autoHideChanged();
    if (isVisible()) {
        if (on)
            unregisterBar();
        else
            registerBar();
    }
    relayout();
}

void DockWindow::setRevealed(bool on)
{
    if (on == m_revealed)
        return;
    m_revealed = on;
    emit revealedChanged();
    if (m_autoHide)
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

void DockWindow::registerBar()
{
#ifdef _WIN32
    if (m_registered)
        return;
    APPBARDATA abd { sizeof(abd) };
    abd.hWnd = hwndOf(this);
    abd.uCallbackMessage = callbackMessage();
    m_registered = SHAppBarMessage(ABM_NEW, &abd) != 0;
#endif
}

void DockWindow::unregisterBar()
{
#ifdef _WIN32
    if (!m_registered)
        return;
    APPBARDATA abd { sizeof(abd) };
    abd.hWnd = hwndOf(this);
    SHAppBarMessage(ABM_REMOVE, &abd);
    m_registered = false;
#endif
}

void DockWindow::relayout()
{
    QScreen *s = screen();
    if (!s || !isVisible())
        return;
    const int open = m_expanded ? std::max(m_thickness, m_expandedWidth) : m_thickness;
    const int shown = m_autoHide && !m_revealed && !m_expanded ? peek() : open;
#ifdef _WIN32
    if (m_registered) {
        const qreal dpr = s->devicePixelRatio();
        // The monitor in physical pixels, as Windows counts it.
        HMONITOR monitor = MonitorFromWindow(hwndOf(this), MONITOR_DEFAULTTOPRIMARY);
        MONITORINFO info { sizeof(info) };
        if (!GetMonitorInfoW(monitor, &info))
            return;
        const int strip = int(std::lround(m_thickness * dpr));
        APPBARDATA abd { sizeof(abd) };
        abd.hWnd = hwndOf(this);
        abd.uEdge = leftEdge() ? ABE_LEFT : ABE_RIGHT;
        abd.rc = info.rcMonitor;
        if (leftEdge())
            abd.rc.right = abd.rc.left + strip;
        else
            abd.rc.left = abd.rc.right - strip;
        // Windows moves the strip off the taskbar and other app bars.
        SHAppBarMessage(ABM_QUERYPOS, &abd);
        if (leftEdge())
            abd.rc.right = abd.rc.left + strip;
        else
            abd.rc.left = abd.rc.right - strip;
        SHAppBarMessage(ABM_SETPOS, &abd);

        RECT rc = abd.rc;
        const int wide = int(std::lround(shown * dpr));
        // Centered on the edge, as tall as its content.
        const int tall = int(std::lround(m_contentHeight * dpr));
        if (tall > 0 && tall < rc.bottom - rc.top) {
            const int top = rc.top + (rc.bottom - rc.top - tall) / 2;
            rc.top = top;
            rc.bottom = top + tall;
        }
        if (leftEdge())
            rc.right = rc.left + wide;
        else
            rc.left = rc.right - wide;
        SetWindowPos(hwndOf(this), m_fullScreenApp ? HWND_BOTTOM : HWND_TOPMOST, rc.left, rc.top,
                     rc.right - rc.left, rc.bottom - rc.top, SWP_NOACTIVATE);
        return;
    }
#endif
    // Without an app bar (auto-hide, or not Windows): cover the edge of the
    // work area.
    const QRect area = s->availableGeometry();
    const int x = leftEdge() ? area.left() : area.right() + 1 - shown;
    const int tall = m_contentHeight > 0 ? std::min(m_contentHeight, area.height()) : area.height();
    setGeometry(x, area.top() + (area.height() - tall) / 2, shown, tall);
}

bool DockWindow::event(QEvent *event)
{
    if (event->type() == QEvent::PlatformSurface
        && static_cast<QPlatformSurfaceEvent *>(event)->surfaceEventType()
            == QPlatformSurfaceEvent::SurfaceAboutToBeDestroyed) {
        unregisterBar();
    }
    return QQuickWindow::event(event);
}

bool DockWindow::nativeEvent(const QByteArray &eventType, void *message, qintptr *result)
{
#ifdef _WIN32
    if (eventType == "windows_generic_MSG" && m_registered) {
        MSG *msg = static_cast<MSG *>(message);
        if (msg->message == callbackMessage()) {
            switch (msg->wParam) {
            case ABN_POSCHANGED:
                relayout();
                break;
            case ABN_FULLSCREENAPP: {
                const bool full = msg->lParam != 0;
                if (full != m_fullScreenApp) {
                    m_fullScreenApp = full;
                    emit fullScreenAppChanged();
                    SetWindowPos(hwndOf(this), full ? HWND_BOTTOM : HWND_TOPMOST, 0, 0, 0, 0,
                                 SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                }
                break;
            }
            default:
                break;
            }
            *result = 0;
            return true;
        }
        if (msg->message == WM_ACTIVATE || msg->message == WM_WINDOWPOSCHANGED) {
            APPBARDATA abd { sizeof(abd) };
            abd.hWnd = msg->hwnd;
            SHAppBarMessage(msg->message == WM_ACTIVATE ? ABM_ACTIVATE : ABM_WINDOWPOSCHANGED, &abd);
        }
        if (msg->message == WM_DISPLAYCHANGE || msg->message == WM_DPICHANGED)
            QMetaObject::invokeMethod(this, &DockWindow::relayout, Qt::QueuedConnection);
    }
#else
    Q_UNUSED(eventType);
    Q_UNUSED(message);
    Q_UNUSED(result);
#endif
    return QQuickWindow::nativeEvent(eventType, message, result);
}

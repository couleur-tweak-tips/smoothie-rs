#include <windows.h>

BOOL SetConsoleWindowTitle(char *wndTitle) { return SetConsoleTitle(wndTitle); }

BOOL SetConsoleParams(BOOL borderless, BOOL alwaysontop, int wnd_position, int wnd_cx, int wnd_cy)
{
    int wndX, wndY;
    HWND hWnd = GetConsoleWindow(),
            hWndPos = HWND_TOP;
    DWORD wndStyle = WS_VISIBLE | WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_VSCROLL,
            wndExStyle = WS_EX_ACCEPTFILES | WS_EX_WINDOWEDGE | WS_EX_APPWINDOW;
    float scale = GetDpiForWindow(hWnd) / 96.0;
    HMONITOR hMonitor = MonitorFromWindow(hWnd, MONITOR_DEFAULTTONEAREST);
    UINT uFlags;
    MONITORINFO mi = {.cbSize = sizeof(MONITORINFO)};
    if (!hWnd ||
        !hMonitor ||
        !GetMonitorInfo(hMonitor, &mi) ||
        !scale ||
        !ShowWindowAsync(hWnd, SW_NORMAL) ||
        !SetWindowPos(hWnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE))
        return FALSE;
    if (!wnd_cx && !wnd_cy)
        uFlags = SWP_NOSIZE;
    if (alwaysontop)
        hWndPos = HWND_TOPMOST;
    if (borderless)
    {
        wndStyle = WS_VISIBLE | WS_POPUP;
        wndExStyle = WS_EX_APPWINDOW;
    };

    switch (wnd_position)
    {
        // Top Left
        case 0:
            wndX = mi.rcWork.left;
            wndY = mi.rcWork.top;
            break;

            // Top Right
        case 1:
            wndX = mi.rcWork.right - wnd_cx;
            wndY = mi.rcWork.top;
            break;

            // Bottom Left
        case 2:
            wndX = mi.rcWork.left;
            wndY = mi.rcWork.bottom - wnd_cy;
            break;

            // Bottom Right
        case 3:
            wndX = mi.rcWork.right - wnd_cx;
            wndY = mi.rcWork.bottom - wnd_cy;
            break;

        default:
            return FALSE;
    }

    SetWindowLong(hWnd, GWL_STYLE, wndStyle);
    SetWindowLong(hWnd, GWL_EXSTYLE, wndExStyle);

    SetWindowPos(hWnd, hWndPos, wndX, wndY, wnd_cx * scale, wnd_cy * scale, uFlags);
    return TRUE;
}
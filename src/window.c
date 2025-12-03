#include <windows.h>

BOOL SetConsoleWindowTitle(char *wndTitle) { return SetConsoleTitle(wndTitle); }

BOOL IsConsoleHost(BOOL bRelaunch)
{
    // Reference: https://stackoverflow.com/a/72575526
    BOOL bConsoleHost = !GetWindow(GetConsoleWindow(), GW_OWNER);
    if (!bConsoleHost && bRelaunch)
    {
        ShellExecute(0, "open", "conhost.exe", GetCommandLine(), ".", SW_SHOW);
        exit(0);
    };
    return bConsoleHost;
}

BOOL SetConsoleParams(BOOL bBorderless, BOOL bAlwaysOnTop, int wndPos, int wndCX, int wndCY)
{
    if (!IsConsoleHost(FALSE))
        return FALSE;
    int wndX = 0,
        wndY = 0;
    UINT uFlags = 0;
    HWND hWnd = GetConsoleWindow(),
         hWndPos = HWND_TOP;
    DWORD wndStyle = WS_VISIBLE | WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_VSCROLL,
          wndExStyle = WS_EX_ACCEPTFILES | WS_EX_WINDOWEDGE | WS_EX_APPWINDOW;
    HMONITOR hMonitor = MonitorFromWindow(hWnd, MONITOR_DEFAULTTONEAREST);

    MONITORINFO mi = {.cbSize = sizeof(MONITORINFO)};

    if (!hWnd ||
        !hMonitor ||
        !GetMonitorInfo(hMonitor, &mi) ||
        !ShowWindowAsync(hWnd, SW_NORMAL) ||
        !SetWindowPos(hWnd,
                      HWND_NOTOPMOST,
                      0, 0, 0, 0,
                      SWP_NOMOVE | SWP_NOSIZE))
        return FALSE;

    if (!wndCX || !wndCY)
        uFlags = SWP_NOSIZE;
    if (bAlwaysOnTop)
        hWndPos = HWND_TOPMOST;
    if (bBorderless)
    {
        wndStyle = WS_VISIBLE | WS_POPUP;
        wndExStyle = WS_EX_APPWINDOW;
    };

    switch (wndPos)
    {
    // Top Left
    case 1:
        wndX = mi.rcWork.left;
        wndY = mi.rcWork.top;
        break;

        // Top Right
    case 2:
        wndX = mi.rcWork.right - wndCX;
        wndY = mi.rcWork.top;
        break;

        // Bottom Left
    case 3:
        wndX = mi.rcWork.left;
        wndY = mi.rcWork.bottom - wndCY;
        break;

        // Bottom Right
    case 4:
        wndX = mi.rcWork.right - wndCX;
        wndY = mi.rcWork.bottom - wndCY;
        break;

    default:
        return FALSE;
    }

    SetWindowLong(hWnd, GWL_STYLE, wndStyle);
    SetWindowLong(hWnd, GWL_EXSTYLE, wndExStyle);
    SetWindowPos(hWnd, hWndPos, wndX, wndY, wndCX, wndCY, uFlags);

    return TRUE;
}
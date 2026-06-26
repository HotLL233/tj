"""System tray application — tray icon + background server."""
import os
import sys
import io
import time
import socket
import threading
import webbrowser
import traceback

LOG_FILE = os.path.join(os.path.dirname(sys.executable), "server.log")


def log(msg):
    try:
        with open(LOG_FILE, "a", encoding="utf-8") as f:
            f.write(f"{time.strftime('%H:%M:%S')} {msg}\n")
    except Exception:
        pass


def create_icon_image():
    from PIL import Image, ImageDraw
    size = 64
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.ellipse([4, 4, size - 4, size - 4], fill="#1976D2")
    bar_w, gap = size // 6, size // 10
    for i, h_pct in enumerate([0.3, 0.55, 0.75, 0.5, 0.9]):
        x0 = gap + i * (bar_w + gap)
        x1 = x0 + bar_w
        bar_h = int(h_pct * size * 0.6)
        draw.rectangle([x0, size - gap - bar_h, x1, size - gap], fill="white")
    return img


def on_open(_icon, _item):
    webbrowser.open("http://localhost:8000")


def on_exit(icon, _item):
    icon.stop()
    os._exit(0)


def is_running(port=8000):
    """Return True if another instance is already listening on port."""
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(0.5)
        s.connect(("127.0.0.1", port))
        s.close()
        return True
    except (OSError, ConnectionRefusedError):
        return False


def run_tray(app):
    """Start server in thread, show tray icon."""
    log("Starting...")

    # Fix stdout/stderr for GUI mode (console=False makes them None)
    null_stream = io.StringIO()
    if sys.stdout is None or getattr(sys.stdout, "closed", False):
        sys.stdout = null_stream
    if sys.stderr is None or getattr(sys.stderr, "closed", False):
        sys.stderr = null_stream

    # Check existing instance
    if is_running():
        log("Already running — opening browser")
        try:
            webbrowser.open("http://localhost:8000")
        except Exception:
            pass
        return

    # Start uvicorn in thread
    import uvicorn

    def _run():
        try:
            log("Server thread started")
            uvicorn.run(app, host="0.0.0.0", port=8000, log_level="error")
        except Exception as e:
            log(f"Server crash: {e}\n{traceback.format_exc()}")

    server_thread = threading.Thread(target=_run, daemon=True)
    server_thread.start()

    # Wait for server to be ready
    log("Waiting for server...")
    ready = False
    for i in range(40):
        time.sleep(0.5)
        if is_running():
            ready = True
            log(f"Server ready after {(i+1)*0.5:.1f}s")
            break
    if not ready:
        log("Server failed to start within 20s")
        return

    # Show tray
    try:
        import pystray
        image = create_icon_image()
        menu = pystray.Menu(
            pystray.MenuItem("打开页面", on_open, default=True),
            pystray.Menu.SEPARATOR,
            pystray.MenuItem("退出", on_exit),
        )
        icon = pystray.Icon("workload_tool", image, "工作量统计工具 v0.2.5", menu)
        log("Tray icon shown")
        icon.run()
    except Exception as e:
        log(f"Tray error: {e}\n{traceback.format_exc()}")

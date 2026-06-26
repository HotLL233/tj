"""工作量统计工具 — FastAPI entry point."""
import os
import sys
from contextlib import asynccontextmanager
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse, Response

from database import init_db
from api import api_router


@asynccontextmanager
async def lifespan(app: FastAPI):
    init_db()
    yield


app = FastAPI(
    title="工作量统计 API",
    description="Workload statistics management system",
    version="0.2.5",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(api_router)

# 静态文件路径
if getattr(sys, "frozen", False):
    static_dir = os.path.join(sys._MEIPASS, "static")
else:
    static_dir = os.path.join(os.path.dirname(__file__), "static")

has_static = os.path.isdir(static_dir) and os.path.isfile(os.path.join(static_dir, "index.html"))
index_path = os.path.join(static_dir, "index.html") if has_static else None
assets_dir = os.path.join(static_dir, "assets") if has_static else None

if has_static and os.path.isdir(assets_dir):
    app.mount("/assets", StaticFiles(directory=assets_dir), name="assets")


@app.middleware("http")
async def spa_fallback_middleware(request: Request, call_next):
    """SPA fallback: 非 API 路径返回 index.html."""
    response = await call_next(request)

    # 只有静态目录存在且有 index.html 时才做 fallback
    if not index_path:
        return response

    # 404 + 非 API + non-asset → 返回 index.html
    if response.status_code == 404 and not request.url.path.startswith("/api"):
        return FileResponse(index_path)

    return response


@app.get("/")
async def serve_root():
    """根路径返回 index.html."""
    if index_path:
        return FileResponse(index_path)
    return Response('{"message":"API running","docs":"/docs"}', media_type="application/json")


if __name__ == "__main__":
    import uvicorn
    if getattr(sys, "frozen", False):
        from tray_app import run_tray
        run_tray(app)
    else:
        uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)

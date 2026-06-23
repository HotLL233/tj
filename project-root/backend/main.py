"""FastAPI application entry point."""
import os
import sys
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
from starlette.routing import Route

from database import init_db
from api import api_router


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan: initialize database on startup."""
    init_db()
    yield


app = FastAPI(
    title="工作量统计 API",
    description="Workload statistics management system",
    version="1.7.0",
    lifespan=lifespan,
)

# CORS - allow all origins
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register API routes
app.include_router(api_router)

# Production mode: serve static files + SPA fallback
if getattr(sys, "frozen", False):
    static_dir = os.path.join(sys._MEIPASS, "static")
else:
    static_dir = os.path.join(os.path.dirname(__file__), "static")

if os.path.isdir(static_dir) and os.path.isfile(os.path.join(static_dir, "index.html")):
    # Serve assets directly
    app.mount("/assets", StaticFiles(directory=os.path.join(static_dir, "assets")), name="assets")

    # SPA fallback: serve index.html for all non-API routes
    @app.get("/{full_path:path}")
    async def serve_spa(full_path: str):
        file_path = os.path.join(static_dir, full_path)
        if os.path.isfile(file_path):
            return FileResponse(file_path)
        return FileResponse(os.path.join(static_dir, "index.html"))

    @app.get("/")
    async def serve_root():
        return FileResponse(os.path.join(static_dir, "index.html"))


if __name__ == "__main__":
    import uvicorn
    if getattr(sys, "frozen", False):
        from tray_app import run_tray
        run_tray(app)
    else:
        uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)

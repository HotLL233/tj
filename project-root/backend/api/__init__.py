"""API router aggregation."""
from fastapi import APIRouter

from .groups import router as groups_router
from .projects import router as projects_router
from .records import router as records_router
from .stats import router as stats_router
from .export_v4 import router as export_router
from .audit_logs import router as audit_logs_router

api_router = APIRouter(prefix="/api")

api_router.include_router(groups_router)
api_router.include_router(projects_router)
api_router.include_router(records_router)
api_router.include_router(stats_router)
api_router.include_router(export_router)
api_router.include_router(audit_logs_router, prefix="/audit-logs")

"""Pydantic request/response models (schemas)."""
from typing import Optional, List, Any
from pydantic import BaseModel, Field


# --- Group ---
class GroupCreate(BaseModel):
    name: str = Field(..., min_length=1, max_length=50)
    sort_order: Optional[int] = 0


class GroupUpdate(BaseModel):
    name: Optional[str] = Field(None, min_length=1, max_length=50)
    sort_order: Optional[int] = None


class GroupResponse(BaseModel):
    id: int
    name: str
    sort_order: int
    project_count: int = 0
    created_at: str


# --- Project ---
class ProjectCreate(BaseModel):
    group_id: int
    name: str = Field(..., min_length=1, max_length=100)
    sort_order: Optional[int] = 0


class ProjectUpdate(BaseModel):
    name: Optional[str] = Field(None, min_length=1, max_length=100)
    full_name: Optional[str] = None
    notes: Optional[str] = None
    sort_order: Optional[int] = None
    is_active: Optional[int] = None


class ProjectResponse(BaseModel):
    id: int
    group_id: int
    group_name: str = ""
    name: str
    full_name: str = ""
    notes: str = ""
    sort_order: int
    is_active: int
    created_at: str


# --- Record ---
class RecordCreate(BaseModel):
    project_id: int
    user_name: str = Field(..., min_length=1, max_length=50)
    quantity: int = Field(..., ge=1)
    recorded_at: str  # ISO 8601: "2025-07-17T14:30:00"


class RecordResponse(BaseModel):
    id: int
    project_id: int
    project_name: str = ""
    group_name: str = ""
    user_name: str
    quantity: int
    recorded_at: str
    created_at: str
    deleted_at: Optional[str] = None


class RecordUpdate(BaseModel):
    """Model for updating an existing work record (user correction)."""
    user_name: Optional[str] = Field(None, min_length=1, max_length=50)
    quantity: Optional[int] = Field(None, ge=1)
    recorded_at: Optional[str] = None


# --- Stats ---
class StatsSummary(BaseModel):
    total_quantity: int
    total_records: int
    user_count: int
    project_count: int
    details: List[Any] = []


class UserStats(BaseModel):
    user_name: str
    total_quantity: int
    record_count: int


class ProjectStats(BaseModel):
    project_id: int
    project_name: str
    group_name: str
    total_quantity: int
    record_count: int


class TypeStats(BaseModel):
    """Statistics grouped by instrument type (液相/气相/其他)."""
    instrument_type: str
    total_quantity: int
    record_count: int


class InstrumentStats(BaseModel):
    """Statistics grouped by individual instrument (LC-01, GC-02, etc.)."""
    instrument: str
    instrument_type: str = ""
    total_quantity: int
    record_count: int
    user_count: int


# --- Common ---
class ApiResponse(BaseModel):
    code: int = 0
    data: Optional[Any] = None
    message: str = "ok"


class PaginatedResponse(BaseModel):
    items: List[Any]
    total: int
    page: int
    page_size: int

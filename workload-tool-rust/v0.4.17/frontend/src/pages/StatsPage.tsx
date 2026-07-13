import React, { useEffect, useState, useCallback } from "react";
import {
  Box,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Button,
  CircularProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Chip,
  Snackbar,
  useMediaQuery,
  useTheme,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import EditIcon from "@mui/icons-material/Edit";
import DeleteForeverIcon from "@mui/icons-material/DeleteForever";
import DeleteIcon from "@mui/icons-material/Delete";
import ViewWeekIcon from "@mui/icons-material/ViewWeek";
import CalendarMonthIcon from "@mui/icons-material/CalendarMonth";
import PeopleIcon from "@mui/icons-material/People";
import FolderIcon from "@mui/icons-material/Folder";
import ScienceIcon from "@mui/icons-material/Science";
import PrecisionManufacturingIcon from "@mui/icons-material/PrecisionManufacturing";
import HistoryIcon from "@mui/icons-material/History";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import TableChartIcon from "@mui/icons-material/TableChart";
import BiotechIcon from "@mui/icons-material/Biotech";
import BusinessIcon from "@mui/icons-material/Business";
import AssessmentIcon from "@mui/icons-material/Assessment";
import WaterDropIcon from "@mui/icons-material/WaterDrop";
import MemoryIcon from "@mui/icons-material/Memory";
import dayjs from "dayjs";
import isoWeek from "dayjs/plugin/isoWeek";
import StatsCards from "../components/StatsCards";
import DateRangePicker from "../components/DateRangePicker";
import ConfirmDialog from "../components/ConfirmDialog";
import PreviewTable from "../components/PreviewTable";
import {
  getStatsSummary,
  getStatsByType,
  getStatsByProject,
  getStatsByInstrument,
  exportExcel,
  getGroups,
  getRecords,
  updateRecord,
  deleteRecord,
  deleteRecordsByUser,
  getPreviewSheet1,
  getPreviewSheet2,
  getPreviewSheet3,
  getPreviewSheet4,
  getPreviewSheet5,
  getPreviewSheet6,
  getPreviewSheet7,
  getPreviewSheet8,
  getPreviewSheet9,
  getPreviewSheet10,
  getRecordUsers,
} from "../api/client";
import type {
  StatsSummary,
  ProjectGroup,
  StatsDetail,
  WorkRecord,
  TypeStats,
  ProjectStats,
  InstrumentStats,
  Sheet1Data,
  Sheet2Row,
  Sheet3Row,
  Sheet4Row,
  Sheet5Row,
  Sheet6Row,
  Sheet7Row,
  Sheet8Row,
  Sheet9Row,
  Sheet10Row,
} from "../types";

dayjs.extend(isoWeek);

// 从方法名中提取仪器标签（@符号后的[...]内容）
const extractInstrumentFromMethodName = (methodName: string): string | null => {
  if (!methodName) return null;
  const atIndex = methodName.indexOf("@");
  if (atIndex === -1) return null;
  const afterAt = methodName.substring(atIndex + 1);
  const bracketStart = afterAt.indexOf("[");
  if (bracketStart === -1) return null;
  const bracketEnd = afterAt.indexOf("]", bracketStart);
  if (bracketEnd === -1) return null;
  return afterAt.substring(bracketStart + 1, bracketEnd);
};

export type TabValue =
  | "week"
  | "month"
  | "user-log"
  | "sheet1"
  | "sheet2"
  | "sheet3"
  | "sheet4"
  | "sheet5"
  | "sheet6"
  | "sheet7"
  | "sheet8"
  | "sheet9"
  | "sheet10";
interface StatCardDef {
  key: TabValue;
  label: string;
  icon: React.ReactNode;
  color: string;
  desc: string;
}

const R = "2px";
const cardSx = {
  p: 2.5,
  borderRadius: R,
  cursor: "pointer",
  background: "linear-gradient(145deg, #ffffff, #f5f5f5)",
  border: "1px solid rgba(0,0,0,0.06)",
  boxShadow: "0 4px 20px rgba(0,0,0,0.06)",
  transition: "all 0.3s cubic-bezier(0.4,0,0.2,1)",
  "&:hover": {
    transform: "translateY(-4px)",
    boxShadow: "0 8px 30px rgba(0,0,0,0.1)",
  },
};
const STAT_CARDS: StatCardDef[] = [
  {
    key: "week",
    label: "按周统计",
    icon: <ViewWeekIcon />,
    color: "#1976d2",
    desc: "每月第几周汇总",
  },
  {
    key: "month",
    label: "按月统计",
    icon: <CalendarMonthIcon />,
    color: "#43a047",
    desc: "每月汇总数据",
  },
  {
    key: "user-log",
    label: "检测人记录",
    icon: <HistoryIcon />,
    color: "#5d4037",
    desc: "逐条记录明细",
  },
  {
    key: "sheet1",
    label: "实验室-项目-方法",
    icon: <BiotechIcon />,
    color: "#1976d2",
    desc: "Sheet 1 各实验室项目方法对应表",
  },
  {
    key: "sheet2",
    label: "仪器-汇总",
    icon: <PrecisionManufacturingIcon />,
    color: "#43A047",
    desc: "Sheet 2 仪器每日汇总",
  },
  {
    key: "sheet3",
    label: "项目-汇总（含金额）",
    icon: <FolderIcon />,
    color: "#FF9800",
    desc: "Sheet 3 项目金额汇总",
  },
  {
    key: "sheet4",
    label: "实验室-汇总（含金额）",
    icon: <BusinessIcon />,
    color: "#9C27B0",
    desc: "Sheet 4 实验室金额汇总",
  },
  {
    key: "sheet5",
    label: "检测人-汇总（原始记录）",
    icon: <HistoryIcon />,
    color: "#E91E63",
    desc: "Sheet 5 检测人原始记录",
  },
  {
    key: "sheet6",
    label: "检测人汇总表（含系数）",
    icon: <PeopleIcon />,
    color: "#00BCD4",
    desc: "Sheet 6 检测人系数汇总",
  },
  {
    key: "sheet7",
    label: "实验室总表",
    icon: <ScienceIcon />,
    color: "#4CAF50",
    desc: "Sheet 7 实验室分类汇总",
  },
  {
    key: "sheet8",
    label: "项目总表",
    icon: <AssessmentIcon />,
    color: "#FFC107",
    desc: "Sheet 8 项目分类汇总",
  },
  {
    key: "sheet9",
    label: "仪器类型汇总",
    icon: <MemoryIcon />,
    color: "#9E9E9E",
    desc: "Sheet 9 仪器类型汇总",
  },
  {
    key: "sheet10",
    label: "理化汇总",
    icon: <WaterDropIcon />,
    color: "#795548",
    desc: "Sheet 10 理化方法汇总",
  },
];
const tablePaperSx = {
  borderRadius: R,
  background: "linear-gradient(145deg, #ffffff, #fafafa)",
  border: "1px solid rgba(0,0,0,0.05)",
  boxShadow: "0 2px 16px rgba(0,0,0,0.04)",
};
const chipSx = {
  bgcolor: "rgba(25,118,210,0.08)",
  color: "#1976d2",
  fontWeight: 600,
  borderRadius: R,
};

const StatsPage: React.FC = () => {
  const t = useTheme();
  const m = useMediaQuery(t.breakpoints.down("sm"));
  const [ac, setAc] = useState<TabValue | null>(null);
  const [s, setS] = useState(() =>
    dayjs().startOf("isoWeek").format("YYYY-MM-DD"),
  );
  const [e, setE] = useState(() =>
    dayjs().endOf("isoWeek").format("YYYY-MM-DD"),
  );
  const [ld, setLd] = useState(false);
  const [er, setEr] = useState("");
  const [sm, setSm] = useState<StatsSummary | null>(null);
  const [dt, setDt] = useState<StatsDetail[]>([]);
  const [gs, setGs] = useState<ProjectGroup[]>([]);
  const [gf, setGf] = useState(0);
  const [byType, setByType] = useState<TypeStats[]>([]);
  const [byProject, setByProject] = useState<ProjectStats[]>([]);
  const [byInstrument, setByInstrument] = useState<InstrumentStats[]>([]);
  const [s1d, setS1d] = useState<Sheet1Data>([]);
  const [s2d, setS2d] = useState<Sheet2Row[]>([]);
  const [s3d, setS3d] = useState<Sheet3Row[]>([]);
  const [s4d, setS4d] = useState<Sheet4Row[]>([]);
  const [s5d, setS5d] = useState<Sheet5Row[]>([]);
  const [s6d, setS6d] = useState<Sheet6Row[]>([]);
  const [s7d, setS7d] = useState<Sheet7Row[]>([]);
  const [s8d, setS8d] = useState<Sheet8Row[]>([]);
  const [s9d, setS9d] = useState<Sheet9Row[]>([]);
  const [s10d, setS10d] = useState<Sheet10Row[]>([]);
  const [ul, setUl] = useState<WorkRecord[]>([]);
  const [ull, setUll] = useState(false);
  const [ulp, setUlp] = useState(1);
  const [ult, setUlt] = useState(0);
  const [uf, setUf] = useState("");
  const [userNames, setUserNames] = useState<string[]>([]);
  const PS = 100;
  const [edo, setEdo] = useState(false);
  const [ef, setEf] = useState({
    id: 0,
    user_name: "",
    quantity: 0,
    recorded_at: "",
  });
  const [ee, setEe] = useState("");
  const [duo, setDuo] = useState(false);
  const [dun, setDun] = useState("");
  const [dul, setDul] = useState(false);
  const [sdr, setSdr] = useState<WorkRecord | null>(null);
  // 各 Sheet 筛选状态
  const [fUser, setFUser] = useState(""); // sheet5/6 按人员筛选
  const [fLab, setFLab] = useState(""); // sheet1/4/7 按实验室筛选
  const [fProject, setFProject] = useState(""); // sheet1/3/8 按项目筛选
  const [fInstrument, setFInstrument] = useState(""); // sheet2/9 按仪器筛选
  const [fMethod, setFMethod] = useState(""); // sheet10 按方法筛选


  const lg = async () => {
    try {
      const r = await getGroups();
      if (r.code === 0) setGs(r.data as ProjectGroup[]);
    } catch {}
  };
  useEffect(() => {
    lg();
  }, []);
  const si = dayjs(s).format("YYYY-MM-DDTHH:mm:ss");
  const ei = dayjs(e).endOf("day").format("YYYY-MM-DDTHH:mm:ss");
  // Load user names from backend for filter dropdown
  useEffect(() => {
    (async () => {
      try {
        const r = await getRecordUsers({ start: si, end: ei });
        if (r.code === 0 && r.data) setUserNames(r.data);
      } catch {}
    })();
  }, [si, ei]);
  const lul = useCallback(
    async (pg = 1, userName = uf) => {
      setUll(true);
      try {
        const p: any = { start: si, end: ei, page: pg, page_size: PS };
        if (userName) p.user_name = userName;
        const r = await getRecords(p);
        if (r.code === 0) {
          const d = r.data as { items: WorkRecord[]; total: number };
          setUl(d.items || []);
          setUlt(d.total || 0);
          setUlp(pg);
        }
      } catch {
      } finally {
        setUll(false);
      }
    },
    [si, ei, uf],
  );
  const ld2 = useCallback(async () => {
    if (!ac) return;
    setLd(true);
    setEr("");
    try {
      if (ac === "week" || ac === "month") {
        const r = await getStatsSummary({
          start: si,
          end: ei,
          group_by: ac === "week" ? "week" : "month",
        });
        if (r.code === 0) {
          setSm(r.data as StatsSummary);
          setDt((r.data as any).details || []);
        } else setEr(r.message);
      } else if (ac === "user-log") await lul(1);
      else if (ac === "sheet1") {
        const r = await getPreviewSheet1({
          start: si,
          end: ei,
          group_id: gf || undefined,
        });
        if (r.code === 0) setS1d((r.data || []) as Sheet1Data);
      } else if (ac === "sheet2") {
        const r = await getPreviewSheet2({ start: si, end: ei });
        if (r.code === 0) setS2d(r.data || []);
      } else if (ac === "sheet3") {
        const r = await getPreviewSheet3({ start: si, end: ei });
        if (r.code === 0) setS3d(r.data || []);
      } else if (ac === "sheet4") {
        const r = await getPreviewSheet4({ start: si, end: ei });
        if (r.code === 0) setS4d(r.data || []);
      } else if (ac === "sheet5") {
        const r = await getPreviewSheet5({ start: si, end: ei });
        if (r.code === 0) setS5d(r.data || []);
      } else if (ac === "sheet6") {
        const r = await getPreviewSheet6({ start: si, end: ei });
        if (r.code === 0) setS6d(r.data || []);
      } else if (ac === "sheet7") {
        const r = await getPreviewSheet7({ start: si, end: ei });
        if (r.code === 0) setS7d(r.data || []);
      } else if (ac === "sheet8") {
        const r = await getPreviewSheet8({ start: si, end: ei });
        if (r.code === 0) setS8d(r.data || []);
      } else if (ac === "sheet9") {
        const r = await getPreviewSheet9({ start: si, end: ei });
        if (r.code === 0) setS9d(r.data || []);
      } else if (ac === "sheet10") {
        const r = await getPreviewSheet10({ start: si, end: ei });
        if (r.code === 0) setS10d(r.data || []);
      }
    } catch {
      setEr("加载失败");
    } finally {
      setLd(false);
    }
  }, [ac, si, ei, gf]);
  useEffect(() => {
    ld2();
  }, [ld2]);
  useEffect(() => {
    (async () => {
      try {
        const r = await getStatsSummary({
          start: si,
          end: ei,
          group_by: "week",
        });
        if (r.code === 0) setSm(r.data as StatsSummary);
      } catch {}
    })();
  }, [si, ei]);
  useEffect(() => {
    (async () => {
      try {
        const [rt, rp, ri] = await Promise.all([
          getStatsByType({ start: si, end: ei }),
          getStatsByProject({ start: si, end: ei }),
          getStatsByInstrument({ start: si, end: ei }),
        ]);
        if (rt.code === 0) setByType(rt.data || []);
        if (rp.code === 0) setByProject(rp.data || []);
        if (ri.code === 0) setByInstrument(ri.data || []);
      } catch {}
    })();
  }, [si, ei]);
  useEffect(() => {
    const handleFocus = () => {
      if (!ac) return;
      ld2();
    };
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        if (!ac) return;
        ld2();
      }
    };
    window.addEventListener("focus", handleFocus);
    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      window.removeEventListener("focus", handleFocus);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [ac, ld2]);
  const hx = async () => {
    try {
      const b = await exportExcel({
        start: si,
        end: ei,
        group_id: gf || undefined,
      });
      const u = URL.createObjectURL(b);
      const a = document.createElement("a");
      a.href = u;
      a.download = `工作量统计_${s}_${e}.xlsx`;
      a.click();
      URL.revokeObjectURL(u);
    } catch {
      setEr("导出失败");
    }
  };
  const oed = (r: WorkRecord) => {
    setEf({
      id: r.id,
      user_name: r.user_name,
      quantity: r.quantity,
      recorded_at: dayjs(r.recorded_at).format("YYYY-MM-DDTHH:mm"),
    });
    setEe("");
    setEdo(true);
  };
  const hes = async () => {
    if (!ef.user_name.trim()) {
      setEe("请输入检测人");
      return;
    }
    if (ef.quantity < 1) {
      setEe("数量必须大于0");
      return;
    }
    try {
      const r = await updateRecord(ef.id, {
        user_name: ef.user_name,
        quantity: ef.quantity,
        recorded_at: dayjs(ef.recorded_at).format("YYYY-MM-DDTHH:mm:ss"),
      });
      if (r.code === 0) {
        setEdo(false);
        lul(ulp);
      } else setEe(r.message);
    } catch {
      setEe("保存失败");
    }
  };
  const odu = (n: string) => {
    setDun(n);
    setDuo(true);
  };
  const hdu = async () => {
    setDul(true);
    try {
      const r = await deleteRecordsByUser(dun, { start: si, end: ei });
      if (r.code === 0) {
        setDuo(false);
        lul(ulp);
        const r2 = await getStatsSummary({
          start: si,
          end: ei,
          group_by: "week",
        });
        if (r2.code === 0) setSm(r2.data as StatsSummary);
      } else {
        setEr(r.message);
        setDuo(false);
      }
    } catch {
      setEr("删除失败");
      setDuo(false);
    } finally {
      setDul(false);
    }
  };
  const hsr = async () => {
    if (!sdr) return;
    try {
      const r = await deleteRecord(sdr.id);
      if (r.code === 0) {
        setSdr(null);
        lul(ulp);
      } else {
        setEr(r.message);
        setSdr(null);
      }
    } catch {
      setEr("单条删除失败");
      setSdr(null);
    }
  };

  const cg = () => (
    <Grid container spacing={2}>
      {STAT_CARDS.map((c) => (
        <Grid item xs={12} sm={6} md={3} key={c.key}>
          <Paper
            onClick={() => {
              setAc(c.key);
              setEr("");
            }}
            sx={{
              ...cardSx,
              "&:hover": {
                ...cardSx["&:hover"],
                borderColor: `${c.color}50`,
                boxShadow: `0 12px 30px ${c.color}20`,
              },
            }}
          >
            <Box
              sx={{ display: "flex", alignItems: "center", gap: 1.5, mb: 1 }}
            >
              <Box
                sx={{
                  width: 40,
                  height: 40,
                  borderRadius: R,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  bgcolor: `${c.color}16`,
                  color: c.color,
                }}
              >
                {c.icon}
              </Box>
              <Typography variant="subtitle1" fontWeight={700}>
                {c.label}
              </Typography>
            </Box>
            <Typography variant="body2" color="text.secondary">
              {c.desc}
            </Typography>
          </Paper>
        </Grid>
      ))}
    </Grid>
  );

  const cct = () => {
    if (!ac) return null;
    return (
      <Box>
        <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 2 }}>
          <IconButton
            onClick={() => {
              setAc(null);
              setEr("");
            }}
            size="small"
            sx={{ bgcolor: "rgba(0,0,0,0.04)", borderRadius: R }}
          >
            <ArrowBackIcon />
          </IconButton>
          <Typography variant="h6" fontWeight={700}>
            {STAT_CARDS.find((x) => x.key === ac)?.label ?? ""}
          </Typography>
        </Box>
        {er && (
          <Alert severity="error" sx={{ mb: 2, borderRadius: R }}>
            {er}
          </Alert>
        )}
        {ld ? (
          <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
            <CircularProgress />
          </Box>
        ) : (
          <Box>
            {(ac === "week" || ac === "month") && (
              <TableContainer
                component={Paper}
                className="table-responsive"
                sx={tablePaperSx}
              >
                <Table size="small">
                  <TableHead>
                    <TableRow>
                      <TableCell sx={{ fontWeight: 600 }}>时间</TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>
                        总数量
                      </TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>
                        记录数
                      </TableCell>
                      <TableCell align="right" sx={{ fontWeight: 600 }}>
                        管理分值
                      </TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {dt.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={4} align="center">
                          暂无数据
                        </TableCell>
                      </TableRow>
                    ) : (
                      dt.map((d, i) => (
                        <TableRow key={i} hover>
                          <TableCell>{d.period}</TableCell>
                          <TableCell align="right">
                            <Chip
                              label={d.total_quantity}
                              size="small"
                              sx={chipSx}
                            />
                          </TableCell>
                          <TableCell align="right">{d.record_count}</TableCell>
                          <TableCell align="right">
                            <Chip
                              label={(d.coefficient_score ?? 0).toFixed(1)}
                              size="small"
                              color="secondary"
                              variant="outlined"
                              sx={{ borderRadius: R }}
                            />
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </TableContainer>
            )}
            {ac === "user-log" && (
              <Box>
                <Box
                  sx={{
                    mb: 2,
                    display: "flex",
                    gap: 2,
                    alignItems: "center",
                    flexWrap: "wrap",
                  }}
                >
                  <FormControl size="small" sx={{ minWidth: 150 }}>
                    <InputLabel>筛选检测人</InputLabel>
                    <Select
                      value={uf}
                      label="筛选检测人"
                      onChange={(x) => {
                        const val = x.target.value;
                        setUf(val);
                        setUlp(1);
                        lul(1, val);
                      }}
                      sx={{ borderRadius: R }}
                    >
                      <MenuItem value="">全部检测人</MenuItem>
                      {userNames.map((n) => (
                        <MenuItem key={n} value={n}>
                          {n}
                        </MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                  <Typography variant="body2" color="text.secondary">
                    共 {ult} 条
                  </Typography>
                </Box>
                {ull ? (
                  <Box
                    sx={{
                      display: "flex",
                      justifyContent: "center",
                      py: 4,
                    }}
                  >
                    <CircularProgress />
                  </Box>
                ) : (
                  <>
                    <TableContainer
                      component={Paper}
                      className="table-responsive"
                      sx={tablePaperSx}
                    >
                      <Table size="small">
                        <TableHead>
                          <TableRow>
                            <TableCell sx={{ fontWeight: 600 }}>序号</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>
                              日期时间
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>
                              检测人
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>
                              实验室
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>项目</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>方法</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>类型</TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>仪器</TableCell>
                            <TableCell align="right" sx={{ fontWeight: 600 }}>
                              数量
                            </TableCell>
                            <TableCell align="center" sx={{ fontWeight: 600 }}>
                              单价倍率
                            </TableCell>
                            <TableCell sx={{ fontWeight: 600 }}>操作</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {ul.length === 0 ? (
                            <TableRow>
                              <TableCell colSpan={11} align="center">
                                暂无数据
                              </TableCell>
                            </TableRow>
                          ) : (
                            ul.map((r, i) => (
                              <TableRow key={r.id} hover>
                                <TableCell>{(ulp - 1) * PS + i + 1}</TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>
                                  {dayjs(r.recorded_at).format(
                                    "YYYY-MM-DD HH:mm:ss",
                                  )}
                                </TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>{r.user_name}</TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>
                                  {(r.group_name || "").split(",")[0].trim() ||
                                    "-"}
                                </TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>{r.project_name}</TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>{r.method_name || "-"}</TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>{r.method_type || "-"}</TableCell>
                                <TableCell sx={{ whiteSpace: "nowrap" }}>
                                  {extractInstrumentFromMethodName(
                                    r.method_name || "",
                                  ) || "-"}
                                </TableCell>
                                <TableCell align="right">
                                  <Chip
                                    label={r.quantity}
                                    size="small"
                                    sx={chipSx}
                                  />
                                </TableCell>
                                <TableCell align="center">
                                  <TextField
                                    size="small"
                                    type="number"
                                    defaultValue={r.multiplier ?? 1.0}
                                    onBlur={async (e) => {
                                      const val = e.target.value === '' ? 1.0 : Number(e.target.value);
                                      if (val !== (r.multiplier ?? 1.0)) {
                                        try {
                                          await updateRecord(r.id, { multiplier: val });
                                          setUl(prev => prev.map(item =>
                                            item.id === r.id ? { ...item, multiplier: val } : item
                                          ));
                                        } catch {}
                                      }
                                    }}
                                    sx={{ width: 70, '& .MuiOutlinedInput-root': { borderRadius: R, fontSize: '0.75rem' } }}
                                    inputProps={{ min: 0, step: 0.1, style: { padding: '2px 4px', textAlign: 'center' } }}
                                  />
                                </TableCell>
                                <TableCell>
                                  <IconButton
                                    size="small"
                                    onClick={() => oed(r)}
                                    title="编辑"
                                    sx={{ color: "#1976d2" }}
                                  >
                                    <EditIcon fontSize="small" />
                                  </IconButton>
                                  <IconButton
                                    size="small"
                                    color="error"
                                    onClick={() => setSdr(r)}
                                    title="删除本条记录"
                                  >
                                    <DeleteIcon fontSize="small" />
                                  </IconButton>
                                </TableCell>
                              </TableRow>
                            ))
                          )}
                        </TableBody>
                      </Table>
                    </TableContainer>
                    {ult > PS && (
                      <Box
                        sx={{
                          display: "flex",
                          justifyContent: "center",
                          mt: 2,
                          gap: 1,
                        }}
                      >
                        <Button
                          size="small"
                          disabled={ulp <= 1}
                          onClick={() => lul(ulp - 1)}
                        >
                          上一页
                        </Button>
                        <Typography
                          variant="body2"
                          sx={{ alignSelf: "center" }}
                        >
                          {ulp} / {Math.max(1, Math.ceil(ult / PS))}
                        </Typography>
                        <Button
                          size="small"
                          disabled={ulp * PS >= ult}
                          onClick={() => lul(ulp + 1)}
                        >
                          下一页
                        </Button>
                      </Box>
                    )}
                  </>
                )}
              </Box>
            )}
            {ac === "sheet1" &&
              (() => {
                const labs = [
                  ...new Set((s1d || []).map((r) => r[0]).filter(Boolean)),
                ] as string[];
                const projects = [
                  ...new Set((s1d || []).map((r) => r[1]).filter(Boolean)),
                ] as string[];
                const fd =
                  fLab || fProject
                    ? (s1d || []).filter(
                        (r) =>
                          (!fLab || r[0] === fLab) &&
                          (!fProject || r[1] === fProject),
                      )
                    : s1d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选实验室</InputLabel>
                        <Select
                          value={fLab}
                          label="筛选实验室"
                          onChange={(x) => setFLab(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部实验室</MenuItem>
                          {labs.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选项目</InputLabel>
                        <Select
                          value={fProject}
                          label="筛选项目"
                          onChange={(x) => setFProject(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部项目</MenuItem>
                          {projects.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable
                      title="各实验室项目方法对应表"
                      columns={[
                        { field: "lab", headerName: "使用实验室", width: 140 },
                        {
                          field: "project",
                          headerName: "项目代号",
                          width: 180,
                        },
                        {
                          field: "instrument",
                          headerName: "液相仪器",
                          width: 180,
                        },
                        {
                          field: "method",
                          headerName: "检测方法",
                          width: 300,
                        },
                        {
                          field: "quantity",
                          headerName: "检测数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "lcTotal",
                          headerName: "液相检测量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "gcTotal",
                          headerName: "气相检测量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "projectTotal",
                          headerName: "项目检测总量",
                          width: 150,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        switch (field) {
                          case "lab":
                            return row[0] ?? "-";
                          case "project":
                            return row[1] ?? "-";
                          case "instrument":
                            return row[2] ?? "-";
                          case "method":
                            return row[3] ?? "-";
                          case "quantity":
                            return row[4] ?? "-";
                          case "lcTotal": {
                            let s = 0;
                            (s1d || []).forEach((r) => {
                              if (r[0] === row[0] && r[1] === row[1]) {
                                s += r[4] || 0;
                              }
                            });
                            return s;
                          }
                          case "gcTotal":
                            return "-";
                          case "projectTotal":
                            return row[6] ?? "-";
                          default:
                            return "-";
                        }
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet2" &&
              (() => {
                const instruments = [
                  ...new Set(
                    (s2d || []).map((r) => r.instrument).filter(Boolean),
                  ),
                ] as string[];
                const fd = fInstrument
                  ? (s2d || []).filter((r) => r.instrument === fInstrument)
                  : s2d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选仪器</InputLabel>
                        <Select
                          value={fInstrument}
                          label="筛选仪器"
                          onChange={(x) => setFInstrument(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部仪器</MenuItem>
                          {instruments.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet2Row>
                      title="仪器-汇总"
                      columns={[
                        { field: "date", headerName: "日期", width: 120 },
                        {
                          field: "instrument",
                          headerName: "仪器",
                          width: 140,
                        },
                        {
                          field: "lab",
                          headerName: "实验室",
                          width: 140,
                        },
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "method",
                          headerName: "方法",
                          width: 300,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet3" &&
              (() => {
                const projects = [
                  ...new Set((s3d || []).map((r) => r.project).filter(Boolean)),
                ] as string[];
                const labs = [
                  ...new Set((s3d || []).map((r) => r.lab).filter(Boolean)),
                ] as string[];
                const fd =
                  fProject || fLab
                    ? (s3d || []).filter(
                        (r) =>
                          (!fProject || r.project === fProject) &&
                          (!fLab || r.lab === fLab),
                      )
                    : s3d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选项目</InputLabel>
                        <Select
                          value={fProject}
                          label="筛选项目"
                          onChange={(x) => setFProject(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部项目</MenuItem>
                          {projects.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选实验室</InputLabel>
                        <Select
                          value={fLab}
                          label="筛选实验室"
                          onChange={(x) => setFLab(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部实验室</MenuItem>
                          {labs.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet3Row>
                      title="项目-汇总（含金额）"
                      columns={[
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "lab",
                          headerName: "实验室",
                          width: 140,
                        },
                        {
                          field: "instrument",
                          headerName: "仪器",
                          width: 140,
                        },
                        {
                          field: "method",
                          headerName: "方法",
                          width: 300,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "unit_price",
                          headerName: "单价",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "total_amount",
                          headerName: "金额",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (field === "total_amount") {
                          return (
                            (
                              (row.quantity || 0) * (row.unit_price || 0)
                            ).toFixed(2) ?? "-"
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet4" &&
              (() => {
                const labs = [
                  ...new Set((s4d || []).map((r) => r.lab).filter(Boolean)),
                ] as string[];
                const projects = [
                  ...new Set((s4d || []).map((r) => r.project).filter(Boolean)),
                ] as string[];
                const fd =
                  fLab || fProject
                    ? (s4d || []).filter(
                        (r) =>
                          (!fLab || r.lab === fLab) &&
                          (!fProject || r.project === fProject),
                      )
                    : s4d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选实验室</InputLabel>
                        <Select
                          value={fLab}
                          label="筛选实验室"
                          onChange={(x) => setFLab(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部实验室</MenuItem>
                          {labs.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选项目</InputLabel>
                        <Select
                          value={fProject}
                          label="筛选项目"
                          onChange={(x) => setFProject(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部项目</MenuItem>
                          {projects.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet4Row>
                      title="实验室-汇总（含金额）"
                      columns={[
                        {
                          field: "lab",
                          headerName: "实验室",
                          width: 140,
                        },
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "instrument",
                          headerName: "仪器",
                          width: 140,
                        },
                        {
                          field: "method",
                          headerName: "方法",
                          width: 300,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "unit_price",
                          headerName: "单价",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "total_amount",
                          headerName: "金额",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (field === "total_amount") {
                          return (
                            (
                              (row.quantity || 0) * (row.unit_price || 0)
                            ).toFixed(2) ?? "-"
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet5" &&
              (() => {
                const users = [
                  ...new Set(
                    (s5d || []).map((r) => r.user_name).filter(Boolean),
                  ),
                ] as string[];
                const fd = fUser
                  ? (s5d || []).filter((r) => r.user_name === fUser)
                  : s5d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选检测人</InputLabel>
                        <Select
                          value={fUser}
                          label="筛选检测人"
                          onChange={(x) => setFUser(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部检测人</MenuItem>
                          {users.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet5Row>
                      title="检测人-汇总（原始记录）"
                      columns={[
                        {
                          field: "recorded_at",
                          headerName: "日期时间",
                          width: 140,
                        },
                        {
                          field: "lab",
                          headerName: "实验室",
                          width: 140,
                        },
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "method",
                          headerName: "方法",
                          width: 250,
                        },
                        {
                          field: "method_type",
                          headerName: "方法类型",
                          width: 100,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 100,
                          align: "right",
                        },
                        {
                          field: "user_name",
                          headerName: "检测人",
                          width: 120,
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (
                          field === "recorded_at" &&
                          (row as any).recorded_at
                        ) {
                          return dayjs((row as any).recorded_at).format(
                            "YYYY-MM-DD HH:mm:ss",
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet6" &&
              (() => {
                const users = [
                  ...new Set(
                    (s6d || []).map((r) => r.user_name).filter(Boolean),
                  ),
                ] as string[];
                const fd = fUser
                  ? (s6d || []).filter((r) => r.user_name === fUser)
                  : s6d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选检测人</InputLabel>
                        <Select
                          value={fUser}
                          label="筛选检测人"
                          onChange={(x) => setFUser(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部检测人</MenuItem>
                          {users.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet6Row>
                      title="检测人汇总表（含系数）"
                      columns={[
                        {
                          field: "user_name",
                          headerName: "检测人",
                          width: 120,
                        },
                        {
                          field: "method_type",
                          headerName: "检测类型",
                          width: 100,
                        },
                        {
                          field: "coefficient",
                          headerName: "系数",
                          width: 80,
                          align: "right",
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 100,
                          align: "right",
                        },
                        {
                          field: "subtotal",
                          headerName: "汇总",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (field === "subtotal") {
                          return (
                            (
                              (row.quantity || 0) * (row.coefficient || 0)
                            ).toFixed(1) ?? "-"
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet7" &&
              (() => {
                const labs = [
                  ...new Set((s7d || []).map((r) => r.lab).filter(Boolean)),
                ] as string[];
                const fd = fLab
                  ? (s7d || []).filter((r) => r.lab === fLab)
                  : s7d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选实验室</InputLabel>
                        <Select
                          value={fLab}
                          label="筛选实验室"
                          onChange={(x) => setFLab(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部实验室</MenuItem>
                          {labs.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet7Row>
                      title="实验室总表"
                      columns={[
                        {
                          field: "lab",
                          headerName: "实验室",
                          width: 140,
                        },
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "method_type",
                          headerName: "方法类型",
                          width: 100,
                        },
                        {
                          field: "unit_price",
                          headerName: "单价",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "total_amount",
                          headerName: "金额",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (field === "total_amount") {
                          return (
                            (
                              (row.quantity || 0) * (row.unit_price || 0)
                            ).toFixed(2) ?? "-"
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet8" &&
              (() => {
                const projects = [
                  ...new Set((s8d || []).map((r) => r.project).filter(Boolean)),
                ] as string[];
                const fd = fProject
                  ? (s8d || []).filter((r) => r.project === fProject)
                  : s8d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选项目</InputLabel>
                        <Select
                          value={fProject}
                          label="筛选项目"
                          onChange={(x) => setFProject(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部项目</MenuItem>
                          {projects.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet8Row>
                      title="项目总表"
                      columns={[
                        {
                          field: "project",
                          headerName: "项目",
                          width: 200,
                        },
                        {
                          field: "method_type",
                          headerName: "方法类型",
                          width: 100,
                        },
                        {
                          field: "unit_price",
                          headerName: "单价",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "total_amount",
                          headerName: "金额",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        if (field === "total_amount") {
                          return (
                            (
                              (row.quantity || 0) * (row.unit_price || 0)
                            ).toFixed(2) ?? "-"
                          );
                        }
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet9" &&
              (() => {
                const instruments = [
                  ...new Set(
                    (s9d || []).map((r) => r.instrument).filter(Boolean),
                  ),
                ] as string[];
                const fd = fInstrument
                  ? (s9d || []).filter((r) => r.instrument === fInstrument)
                  : s9d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选仪器</InputLabel>
                        <Select
                          value={fInstrument}
                          label="筛选仪器"
                          onChange={(x) => setFInstrument(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部仪器</MenuItem>
                          {instruments.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet9Row>
                      title="仪器类型汇总"
                      columns={[
                        {
                          field: "instrument",
                          headerName: "仪器",
                          width: 160,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                        {
                          field: "instrument_type",
                          headerName: "仪器类型",
                          width: 100,
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
            {ac === "sheet10" &&
              (() => {
                const methods = [
                  ...new Set((s10d || []).map((r) => r.method).filter(Boolean)),
                ] as string[];
                const fd = fMethod
                  ? (s10d || []).filter((r) => r.method === fMethod)
                  : s10d || [];
                return (
                  <Box>
                    <Box
                      sx={{
                        mb: 2,
                        display: "flex",
                        gap: 2,
                        alignItems: "center",
                        flexWrap: "wrap",
                      }}
                    >
                      <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>筛选方法</InputLabel>
                        <Select
                          value={fMethod}
                          label="筛选方法"
                          onChange={(x) => setFMethod(x.target.value)}
                          sx={{ borderRadius: R }}
                        >
                          <MenuItem value="">全部方法</MenuItem>
                          {methods.map((n) => (
                            <MenuItem key={n} value={n}>
                              {n}
                            </MenuItem>
                          ))}
                        </Select>
                      </FormControl>
                      <Typography variant="body2" color="text.secondary">
                        共 {fd.length} 条
                      </Typography>
                    </Box>
                    <PreviewTable<Sheet10Row>
                      title="理化汇总"
                      columns={[
                        {
                          field: "method",
                          headerName: "方法",
                          width: 300,
                        },
                        {
                          field: "quantity",
                          headerName: "数量",
                          width: 120,
                          align: "right",
                        },
                      ]}
                      data={fd}
                      loading={ld}
                      getRowKey={(_, i) => i}
                      renderCell={(row, field) => {
                        return (row as any)[field] ?? "-";
                      }}
                    />
                  </Box>
                );
              })()}
          </Box>
        )}
      </Box>
    );
  };

  return (
    <Box>
      <Typography
        variant="h5"
        fontWeight={700}
        sx={{
          mb: 3,
          px: 1,
          background: "linear-gradient(135deg, #00897b, #43a047)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          backgroundClip: "text",
        }}
      >
        统计分析
      </Typography>
      {sm && (
        <StatsCards
          summary={sm}
          byType={byType}
          byProject={byProject}
          byInstrument={byInstrument}
          themeColor="#1976d2"
          onCardClick={(x) => {
            setAc(x);
            setEr("");
          }}
        />
      )}
      {/* 筛选区 — 移动端折叠为 Accordion */}
      {m ? (
        <Accordion
          defaultExpanded={false}
          sx={{ mb: 2, borderRadius: R, "&:before": { display: "none" } }}
        >
          <AccordionSummary
            expandIcon={<ExpandMoreIcon />}
            sx={{ borderRadius: R }}
          >
            <Typography variant="body2" fontWeight={600}>
              筛选条件 · {dayjs(s).format("MM/DD")} ~ {dayjs(e).format("MM/DD")}
            </Typography>
          </AccordionSummary>
          <AccordionDetails sx={{ pt: 0 }}>
            <DateRangePicker
              startDate={s}
              endDate={e}
              onStartChange={setS}
              onEndChange={setE}
            />
            <Box
              sx={{
                display: "flex",
                justifyContent: "flex-end",
                mt: 1,
                gap: 1,
              }}
            >
              <Button
                variant="contained"
                startIcon={<DownloadIcon />}
                onClick={hx}
                size="small"
                sx={{
                  borderRadius: R,
                  background: "linear-gradient(135deg, #00897b, #43a047)",
                  boxShadow: "0 4px 14px rgba(0,137,123,0.3)",
                  "&:hover": {
                    background: "linear-gradient(135deg, #00796b, #388e3c)",
                  },
                }}
              >
                导出 Excel
              </Button>
            </Box>
          </AccordionDetails>
        </Accordion>
      ) : (
        <Box sx={{ mb: 3, px: 1 }}>
          <DateRangePicker
            startDate={s}
            endDate={e}
            onStartChange={setS}
            onEndChange={setE}
          />
          <Box sx={{ display: "flex", justifyContent: "flex-end", mt: 1 }}>
            <Button
              variant="contained"
              startIcon={<DownloadIcon />}
              onClick={hx}
              size="small"
              sx={{
                borderRadius: R,
                background: "linear-gradient(135deg, #00897b, #43a047)",
                boxShadow: "0 4px 14px rgba(0,137,123,0.3)",
                "&:hover": {
                  background: "linear-gradient(135deg, #00796b, #388e3c)",
                },
              }}
            >
              导出 Excel
            </Button>
          </Box>
        </Box>
      )}
      {ac ? cct() : cg()}
      <Dialog
        open={edo}
        onClose={() => setEdo(false)}
        maxWidth="sm"
        fullWidth
        PaperProps={{ sx: { borderRadius: R } }}
      >
        <DialogTitle sx={{ fontWeight: 700 }}>编辑记录</DialogTitle>
        <DialogContent>
          {ee && (
            <Alert severity="error" sx={{ mb: 2, borderRadius: R }}>
              {ee}
            </Alert>
          )}
          <TextField
            label="检测人"
            fullWidth
            value={ef.user_name}
            onChange={(x) => setEf({ ...ef, user_name: x.target.value })}
            sx={{ mt: 1, "& .MuiOutlinedInput-root": { borderRadius: R } }}
          />
          <TextField
            label="数量"
            type="number"
            fullWidth
            value={ef.quantity}
            onChange={(x) => setEf({ ...ef, quantity: Number(x.target.value) })}
            sx={{ mt: 2, "& .MuiOutlinedInput-root": { borderRadius: R } }}
            inputProps={{ min: 1 }}
          />
          <TextField
            label="日期时间"
            type="datetime-local"
            fullWidth
            value={ef.recorded_at}
            onChange={(x) => setEf({ ...ef, recorded_at: x.target.value })}
            sx={{ mt: 2, "& .MuiOutlinedInput-root": { borderRadius: R } }}
            InputLabelProps={{ shrink: true }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEdo(false)}>取消</Button>
          <Button onClick={hes} variant="contained">
            保存
          </Button>
        </DialogActions>
      </Dialog>
      <ConfirmDialog
        open={duo}
        title="删除检测人记录"
        message={`确定要删除检测人「${dun}」在所选日期范围内的所有记录吗？`}
        confirmText="删除"
        cancelText="取消"
        onConfirm={hdu}
        onCancel={() => setDuo(false)}
      />
      <ConfirmDialog
        open={!!sdr}
        title="删除记录"
        message={`确定要删除「${sdr?.user_name ?? ""}」的这条记录吗？（数量: ${sdr?.quantity ?? 0}）`}
        confirmText="删除"
        cancelText="取消"
        onConfirm={hsr}
        onCancel={() => setSdr(null)}
      />
    </Box>
  );
};
export default StatsPage;

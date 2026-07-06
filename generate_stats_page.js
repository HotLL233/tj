const fs = require('fs');
const path = require('path');

// Read the clean file
const filePath = 'D:/桌面/工作量统计工具项目/project-root/frontend/src/pages/StatsPage.tsx';
let content = fs.readFileSync(filePath, 'utf8');

// Step 1: Add MUI imports (Accordion, AccordionSummary, AccordionDetails)
content = content.replace(
  /}([^]*?)from "@mui\/material";/,
  `  Accordion,
  AccordionSummary,
  AccordionDetails,
}$1from "@mui/material";`
);

// Step 2: Add icon imports
content = content.replace(
  /import ArrowBackIcon from "@mui\/icons-material\/ArrowBack";/,
  `import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import TableChartIcon from "@mui/icons-material/TableChart";
import BiotechIcon from "@mui/icons-material/Biotech";
import BusinessIcon from "@mui/icons-material/Business";
import AssessmentIcon from "@mui/icons-material/Assessment";
import WaterDropIcon from "@mui/icons-material/WaterDrop";
import MemoryIcon from "@mui/icons-material/Memory";`
);

// Step 3: Add PreviewTable import
content = content.replace(
  /import ConfirmDialog from "..\/components\/ConfirmDialog";/,
  `import ConfirmDialog from "../components/ConfirmDialog";
import PreviewTable from "../components/PreviewTable";`
);

// Step 4: Add sheet API imports
content = content.replace(
  /import {\n  getStatsSummary,\n  getStatsByUser,\n  getStatsByProject,\n  getStatsByType,\n  getStatsByInstrument,\n  exportExcel,\n  getGroups,\n  getRecords,\n  updateRecord,\n  deleteRecord,\n  deleteRecordsByUser,\n} from "..\/api\/client";/,
  `import {
  getStatsSummary,
  getStatsByUser,
  getStatsByProject,
  getStatsByType,
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
} from "../api/client";`
);

// Step 5: Add sheet type imports
content = content.replace(
  /} from "..\/types";/,
  `  Sheet1Data,
  Sheet2Row,
  Sheet3Row,
  Sheet4Row,
  Sheet5Row,
  Sheet6Row,
  Sheet7Row,
  Sheet8Row,
  Sheet9Row,
  Sheet10Row,
} from "../types";`
);

// Step 6: Update TabValue type
content = content.replace(
  /export type TabValue =\n  "week" \| "month" \| "user" \| "project" \| "type" \| "instrument" \| "user-log";/,
  `export type TabValue =
  "week" | "month" | "user" | "project" | "type" | "instrument" | "user-log"
  | "sheet1" | "sheet2" | "sheet3" | "sheet4" | "sheet5"
  | "sheet6" | "sheet7" | "sheet8" | "sheet9" | "sheet10";`
);

// Step 7: Add helper function for extracting instrument from method name
const helperFunc = `
// 从方法名中提取仪器标签（@符号后的[...]内容）
const extractInstrumentFromMethodName = (methodName: string): string | null => {
  if (!methodName) return null;
  const atIndex = methodName.indexOf('@');
  if (atIndex === -1) return null;
  const afterAt = methodName.substring(atIndex + 1);
  const bracketStart = afterAt.indexOf('[');
  if (bracketStart === -1) return null;
  const bracketEnd = afterAt.indexOf(']', bracketStart);
  if (bracketEnd === -1) return null;
  return afterAt.substring(bracketStart + 1, bracketEnd);
};

`;

content = content.replace(
  /export type TabValue[\s\S]*?";/,
  (match) => match + helperFunc
);

// Write the file
fs.writeFileSync(filePath, content, 'utf8');
console.log('Step 1-6 completed: Added imports and types');

export interface AdaptiveColumnConfig<T> {
  key: string;
  header: string;
  min?: number;
  max?: number;
  fixed?: number;
  padding?: number;
  getValue: (row: T) => unknown;
}

const textUnits = (value: unknown): number => {
  const text = String(value ?? '');
  if (!text || text === '-') return 1;
  let units = 0;
  for (const char of text) {
    if (/[\u4e00-\u9fff]/.test(char)) units += 2;
    else if (/[A-Z0-9]/.test(char)) units += 1.05;
    else units += 0.85;
  }
  return Math.max(1, units);
};

const clamp = (value: number, min: number, max: number) => Math.max(min, Math.min(max, value));

export const getAdaptiveColumnWidths = <T,>(
  rows: T[],
  columns: AdaptiveColumnConfig<T>[],
): Record<string, number> => {
  return columns.reduce<Record<string, number>>((acc, col) => {
    if (col.fixed) {
      acc[col.key] = col.fixed;
      return acc;
    }
    const min = col.min ?? 64;
    const max = col.max ?? 180;
    const padding = col.padding ?? 24;
    const headerUnits = textUnits(col.header);
    const bodyUnits = rows.reduce((longest, row) => Math.max(longest, textUnits(col.getValue(row))), 0);
    acc[col.key] = Math.round(clamp(Math.max(headerUnits, bodyUnits) * 8 + padding, min, max));
    return acc;
  }, {});
};

export const adaptiveTableSx = {
  width: 'max-content',
  minWidth: '100%',
  tableLayout: 'fixed',
};

export const adaptiveCellSx = (width: number) => ({
  width,
  maxWidth: width,
  whiteSpace: 'normal',
  overflowWrap: 'anywhere',
  wordBreak: 'break-word',
  verticalAlign: 'top',
});

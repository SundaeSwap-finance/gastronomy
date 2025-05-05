interface IBudget {
  steps: number;
  mem: number;
  stepsDiff: number;
  memDiff: number;
}

interface IEnv {
  name: string;
  value: string;
}

export interface IFrame {
  budget: IBudget;
  context: string[];
  label: string;
  retValue: string | null;
  term: string;
  location: string | null;
  env: IEnv[];
}

export interface ITraceResponse {
  identifiers: string[];
}

export interface ISummaryResponse {
  frameCount: number;
  sourceTokenIndices: number[];
}

export interface IFrameResponse {
  frame: IFrame;
}

export interface ISourceResponse {
  files: Record<string, string>;
}

export interface IBlockfrostSettings {
  key: string;
}

export interface IScriptOverride {
  from: string;
  to: string;
}

export interface ISettings {
  blockfrost?: IBlockfrostSettings;
  blueprintFile?: string;
  scriptOverrides?: IScriptOverride[];
}

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
  env: IEnv[];
}

export interface ITraceResponse {
  identifiers: string[];
}

export interface ISummaryResponse {
  frameCount: number;
}

export interface IFrameResponse {
  frame: IFrame;
}

export interface IBlockfrostSettings {
  key: string;
}

export interface ISettings {
  blockfrost?: IBlockfrostSettings;
}

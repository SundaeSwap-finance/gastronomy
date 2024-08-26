interface IBudget {
  cpu: number;
  mem: number;
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
  identifier: string;
}

export interface ISummaryResponse {
  frameCount: number;
}

export interface IFrameResponse {
  frame: IFrame;
}
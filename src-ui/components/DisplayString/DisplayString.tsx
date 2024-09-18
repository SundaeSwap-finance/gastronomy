import { FC, Fragment } from "react";

interface IDisplayStringProps {
  string?: string | null;
  highlight?: { line: number; column: number } | null;
}

const DisplayString: FC<IDisplayStringProps> = ({ string, highlight }) => {
  if (!string) return null;

  const formattedCode = string.split("\n").map((line, index) => {
    if (index + 1 === highlight?.line) {
      const col = highlight.column - 1;
      const before = line.substring(0, col);
      const at = line[col];
      const after = line.substring(col + 1);
      return (
        <div key={index} className="bg-slate-800">
          {before}
          <span className="text-lime-500 animate-caret-blink">{at}</span>
          {after}
          <br />
        </div>
      );
    }
    return (
      <Fragment key={index}>
        {line}
        <br />
      </Fragment>
    );
  });

  return <span style={{ whiteSpace: "pre-wrap" }}>{formattedCode}</span>;
};

export default DisplayString;

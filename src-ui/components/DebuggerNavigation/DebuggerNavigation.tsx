import { FC } from "react";
import cx from "classnames";

interface IDebuggerNavigationProps {
  className?: string;
  multipleTraces: boolean;
  handleNext: () => void;
  handleNextTrace: () => void;
  handlePrevious: () => void;
  handleQuit: () => void;
}

const DebuggerNavigation: FC<IDebuggerNavigationProps> = ({
  className,
  multipleTraces,
  handleNext,
  handleNextTrace,
  handlePrevious,
  handleQuit,
}) => {
  return (
    <div className={cx("px-2 bg-slate-950 flex gap-2 text-sm", className)}>
      <div>
        <button className="hover:underline" onClick={handleNext}>
          Next
        </button>{" "}
        <span className="text-blue-600">{"<N>"}</span>
      </div>
      {multipleTraces && (
        <div>
          <button className="hover:underline" onClick={handleNextTrace}>
            Next Trace
          </button>{" "}
          <span className="text-blue-600">{"<T>"}</span>
        </div>
      )}
      <div>
        <button className="hover:underline" onClick={handlePrevious}>
          Previous
        </button>{" "}
        <span className="text-blue-600">{"<P>"}</span>
      </div>
      <div>
        <button className="hover:underline" onClick={handleQuit}>
          Quit
        </button>{" "}
        <span className="text-blue-600">{"<Q>"}</span>
      </div>
    </div>
  );
};

export default DebuggerNavigation;

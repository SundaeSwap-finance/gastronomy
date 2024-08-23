import { FC } from "react";
import cx from "classnames";

interface DebuggerNavigationProps {
  className?: string;
  handleNext: () => void;
  handlePrevious: () => void;
}

const DebuggerNavigation: FC<DebuggerNavigationProps> = ({
  className,
  handleNext,
  handlePrevious,
}) => {
  return (
    <div className={cx("px-2 bg-slate-950 flex gap-2 text-sm", className)}>
      <div>
        <button className="hover:underline" onClick={handleNext}>
          Next
        </button>{" "}
        <span className="text-blue-600">{"<N>"}</span>
      </div>
      <div>
        <button className="hover:underline" onClick={handlePrevious}>
          Previous
        </button>{" "}
        <span className="text-blue-600">{"<P>"}</span>
      </div>
      <div>
        <button className="hover:underline">Quit</button>{" "}
        <span className="text-blue-600">{"<Q>"}</span>
      </div>
    </div>
  );
};

export default DebuggerNavigation;

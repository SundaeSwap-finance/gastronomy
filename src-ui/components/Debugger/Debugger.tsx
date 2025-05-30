import { FC, useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { message, open } from "@tauri-apps/plugin-dialog";
import cx from "classnames";
import {
  IFrame,
  IFrameResponse,
  ISourceResponse,
  ISummaryResponse,
  ITraceResponse,
} from "../../types";
import DisplayString from "../DisplayString";
import Modal from "../Modal";
import DebuggerNavigation from "../DebuggerNavigation";
import { TbFaceIdError } from "react-icons/tb";
import { Triangle } from "react-loader-spinner";

interface IDebuggerProps {
  file: string;
  fileName: string;
  onQuit: () => void;
  parameters: string[];
}

const Debugger: FC<IDebuggerProps> = ({
  onQuit,
  file,
  parameters,
  fileName,
}) => {
  const [identifiers, setIdentifiers] = useState<string[] | undefined>(
    undefined,
  );
  const [identifier, setIdentifier] = useState<string | undefined>(undefined);
  const [currentFrameIndex, setCurrentFrameIndex] = useState(0);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [error, setError] = useState("");
  const [viewSource, setViewSource] = useState(false);
  const [frameCount, setFrameCount] = useState<number>(0);
  const [sourceTokenIndices, setSourceTokenIndices] = useState<number[]>([]);
  const [sourceFiles, setSourceFiles] = useState<Record<string, string>>({});
  const [currentFrame, setCurrentFrame] = useState<IFrame | undefined>(
    undefined,
  );

  useEffect(() => {
    const fetchFrame = async () => {
      if (!identifier) return;
      const { frame } = await invoke<IFrameResponse>("get_frame", {
        identifier,
        frame: currentFrameIndex,
      });
      setCurrentFrame(frame);
    };
    fetchFrame().catch(setError);
  }, [currentFrameIndex, identifier]);

  const displayLabel = (frameIndex: number) => {
    if (frameIndex === frameCount - 1) return "Done";
    if (frameIndex === frameCount) return "None";
    if (currentFrame?.retValue) return "Return";
    return "Compute";
  };

  const handleQuit = useCallback(() => {
    onQuit();
    setCurrentFrame(undefined);
  }, [onQuit]);

  const handleNext = useCallback(() => {
    setCurrentFrameIndex((prev) => {
      let next;
      if (viewSource) {
        next = sourceTokenIndices.find((i) => i > prev) ?? frameCount - 1;
      } else {
        next = prev + 1;
      }
      return Math.min(next, frameCount - 1);
    });
  }, [viewSource, sourceTokenIndices, frameCount]);

  const handlePrevious = useCallback(() => {
    setCurrentFrameIndex((prev) => {
      let next;
      if (viewSource) {
        next =
          sourceTokenIndices
            .slice()
            .reverse()
            .find((i) => i < prev) ?? 0;
      } else {
        next = prev - 1;
      }
      return Math.max(next, 0);
    });
  }, [viewSource, sourceTokenIndices]);

  const handleNextTrace = useCallback(() => {
    if (!identifiers) return;
    const currentIndex = identifiers.indexOf(identifier ?? "");
    const nextIdentifier = identifiers[currentIndex + 1] || identifiers[0];
    setIdentifier(nextIdentifier);
  }, [identifiers, identifier]);

  const [sourceText, sourcePos] = useMemo(() => {
    const location = currentFrame?.location;
    if (!location) return [null, null];
    const [file, line, column] = location.split(":");
    const sourcePos = {
      line: Number(line),
      column: Number(column),
    };
    return [sourceFiles[file] || null, sourcePos];
  }, [currentFrame, sourceFiles]);

  const handleViewSource = useCallback(async () => {
    if (!identifier) return;
    if (viewSource) {
      setViewSource(false);
      return;
    }
    const location = currentFrame?.location;
    if (!location) return;
    const [file] = location.split(":");
    if (!sourceFiles[file]) {
      await message("Please select the root directory of your app.");
      const sourceRoot = await open({
        title: "Open Aiken source root",
        multiple: false,
        directory: true,
        recursive: true,
      });
      if (!sourceRoot) return;
      try {
        const { files } = await invoke<ISourceResponse>("get_source", {
          identifier,
          sourceRoot,
        });
        console.log({ files });
        setSourceFiles(files);
      } catch (error) {
        setError(error as string);
        return;
      }
    }
    setViewSource(true);
  }, [identifier, viewSource, currentFrame, sourceFiles]);

  const handleKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "n") {
        handleNext();
      } else if (event.key === "p") {
        handlePrevious();
      } else if (event.key === "q") {
        handleQuit();
      } else if (event.key === "t") {
        handleNextTrace();
      } else if (event.key === "v") {
        handleViewSource();
      }
    },
    [handleNext, handlePrevious, handleQuit, handleNextTrace, handleViewSource],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyPress);
    return () => {
      window.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  const fetchIdentifiers = useCallback(
    async (file: string, parameters: string[]) => {
      try {
        const { identifiers } = await invoke<ITraceResponse>("create_traces", {
          file,
          parameters,
        });
        setIdentifiers(identifiers);
        const identifier = identifiers[0];
        setIdentifier(identifier);
      } catch (error) {
        setError(error as string);
      }
    },
    [],
  );

  useEffect(() => {
    fetchIdentifiers(file, parameters);
  }, [fetchIdentifiers, file, parameters]);

  const fetchFrames = useCallback(async (identifier: string) => {
    try {
      const { frameCount, sourceTokenIndices } = await invoke<ISummaryResponse>(
        "get_trace_summary",
        {
          identifier,
        },
      );
      setFrameCount(frameCount);
      setSourceTokenIndices(sourceTokenIndices);
      setSourceFiles({});
      setCurrentFrameIndex(0);
      setIsModalOpen(false);
    } catch (error) {
      setError(error as string);
    }
  }, []);

  useEffect(() => {
    if (identifier) fetchFrames(identifier);
  }, [identifier, fetchFrames]);

  useEffect(() => {
    setIsModalOpen(!!currentFrame?.retValue);
  }, [currentFrame?.retValue]);

  const { stepsDiff = 0, memDiff = 0 } = currentFrame?.budget ?? {};

  if (error) {
    return (
      <div className="h-full flex justify-center items-center">
        <div className="max-w-[30rem] flex flex-col gap-6 items-center">
          <TbFaceIdError size={80} />
          <div className="flex flex-col gap-2 text-center">
            <div className="text-blue-600">An error occurred:</div>
            <div>{error}</div>
          </div>
          <button
            className={cx(
              "py-2 px-6 text-lime-600 border border-lime-600 transition-colors",
              "hover:bg-lime-600 hover:text-slate-950 duration-300 ease-in-out",
            )}
            onClick={onQuit}
          >
            Try again
          </button>
        </div>
      </div>
    );
  }

  if (!identifier) {
    return (
      <div className="h-svh flex items-center justify-center">
        <Triangle
          height="80"
          width="80"
          color="#55960E"
          ariaLabel="triangle-loading"
        />
      </div>
    );
  }

  const traceIndex = identifiers?.indexOf(identifier);
  const multipleTraces = (identifiers?.length ?? 0) > 1;
  const title = multipleTraces ? `${fileName} #${traceIndex}` : fileName;

  return (
    <div className="px-2 pb-3 pt-4 relative h-full">
      <div className="border border-lime-600 h-full pt-3">
        <h1 className="px-2 bg-slate-950 absolute right-1/2 translate-x-1/2 top-1">
          Gastronomy Debugger ({title})
        </h1>
        <div className="grid grid-rows-[max-content_1fr] h-full text-sm">
          <div className="px-2 pt-1 pb-4">
            <div className="overflow-hidden h-4 mb-1 text-xs flex relative">
              <div
                style={{
                  width: `${(currentFrameIndex / (frameCount - 1)) * 100}%`,
                }}
                className="shadow-none flex flex-col text-center whitespace-nowrap text-white justify-center bg-lime-900 overflow-auto"
              />
            </div>
            <div className="text-xs absolute top-[33px] right-1/2 translate-x-1/2 text-lime-600">
              Step {currentFrameIndex}/{frameCount - 1}
            </div>
            <div className="flex justify-between">
              <div className="w-36">
                Current:{" "}
                <span className="text-blue-600">
                  {displayLabel(currentFrameIndex)}
                </span>
              </div>
              <div className="flex gap-2">
                Budget:
                <div className="flex gap-2">
                  <span className="text-blue-600">
                    {currentFrame?.budget.steps} steps
                  </span>{" "}
                  {!!stepsDiff && `+(${stepsDiff})`}
                </div>
                <div className="flex gap-2">
                  <span className="text-blue-600">
                    {currentFrame?.budget.mem} mem
                  </span>
                  {!!memDiff && `+(${memDiff})`}
                </div>
              </div>
              <div className="w-36 text-right">
                Next:{" "}
                <span className="text-blue-600">
                  {displayLabel(currentFrameIndex + 1)}
                </span>
              </div>
            </div>
          </div>
          <div className="grid grid-cols-2 h-full border-t border-lime-600">
            <div className="relative border-r border-lime-600">
              <h2 className="left-2 -top-3 bg-slate-950 text-blue-600 absolute px-2 z-10">
                {viewSource ? "Source" : "Term"}
              </h2>
              <div className="h-full flex flex-col">
                <div className="flex-auto relative">
                  <div className="p-4 overflow-auto absolute inset-0">
                    <DisplayString
                      string={
                        viewSource
                          ? sourceText || "File not found"
                          : currentFrame?.term
                      }
                      highlight={viewSource ? sourcePos : null}
                    />
                  </div>
                </div>
                {currentFrame?.location && (
                  <div className="p-3 border-t border-lime-600 flex-initial relative">
                    <h2 className="left-2 -top-3 bg-slate-950 absolute px-2 z-10">
                      Source Location
                    </h2>
                    <div className="p-4 overflow-auto relative inset-0">
                      <DisplayString string={currentFrame.location} />
                    </div>
                    <div className="left-2 -bottom-2 bg-slate-950 absolute px-2 z-10">
                      <button className="hover:underline">
                        {viewSource ? "View Term" : "View Source"}
                      </button>{" "}
                      <span className="text-blue-600">{"<V>"}</span>
                    </div>
                  </div>
                )}
              </div>
            </div>
            <div className="relative">
              <h2 className="left-2 -top-3 bg-slate-950 absolute px-2 z-10">
                Context
              </h2>
              <div className="h-full grid grid-rows-2">
                <div className="relative">
                  <div className="p-4 overflow-auto absolute inset-0">
                    {currentFrame?.context.map((c, i) => (
                      <div key={i}>
                        {i !== 0 && "->"} {c}
                      </div>
                    ))}
                  </div>
                </div>
                <div className="p-3 border-t border-lime-600 relative">
                  <h2 className="left-2 -top-3 bg-slate-950 absolute px-2 z-10">
                    Env
                  </h2>
                  <div className="relative h-full">
                    <div className="p-4 overflow-auto absolute inset-0">
                      {currentFrame?.env.map(({ name, value }, i) => (
                        <div key={i}>
                          {name}: <DisplayString string={value} />
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <DebuggerNavigation
        className="absolute right-1/2 translate-x-1/2 bottom-1"
        multipleTraces={multipleTraces}
        handleNext={handleNext}
        handlePrevious={handlePrevious}
        handleQuit={handleQuit}
        handleNextTrace={handleNextTrace}
      />
      <Modal className="w-[50rem]" isOpen={isModalOpen}>
        <div className="">
          <h2 className="absolute left-4 -top-3 bg-slate-950 px-2 text-blue-600">
            Return Value
          </h2>
          <div className="px-4 pt-4 pb-6 h-[30rem] overflow-auto">
            <DisplayString string={currentFrame?.retValue} />
          </div>
        </div>
        <DebuggerNavigation
          className="absolute right-1/2 translate-x-1/2 -bottom-2"
          multipleTraces={multipleTraces}
          handleNext={handleNext}
          handleNextTrace={handleNextTrace}
          handlePrevious={handlePrevious}
          handleQuit={handleQuit}
        />
      </Modal>
    </div>
  );
};

export default Debugger;

import { FC, useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api";
import cx from "classnames";
import {
  IFrame,
  IFrameResponse,
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
  const [currentFrameIndex, setCurrentFrameIndex] = useState(0);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState("");
  const [frames, setFrames] = useState<IFrame[]>([]);

  const currentFrame = frames[currentFrameIndex];

  const handleQuite = useCallback(() => {
    onQuit();
    setFrames([]);
  }, [onQuit]);

  const handleNext = useCallback(() => {
    if (currentFrameIndex < frames.length - 1) {
      setCurrentFrameIndex((prev) => prev + 1);
      setIsModalOpen(false);
    }
  }, [currentFrameIndex, frames.length]);

  const displayLabel = (frameIndex: number) => {
    if (frameIndex === frames.length - 1) return "Done";
    if (frameIndex === frames.length) return "None";
    if (frames[frameIndex].retValue) return "Return";
    return "Compute";
  };

  const handlePrevious = useCallback(() => {
    if (currentFrameIndex > 0) {
      setCurrentFrameIndex((prev) => prev - 1);
      setIsModalOpen(false);
    }
  }, [currentFrameIndex]);

  const handleKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "n") {
        handleNext();
      } else if (event.key === "p") {
        handlePrevious();
      } else if (event.key === "q") {
        handleQuite();
      }
    },
    [handleNext, handlePrevious, handleQuite]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyPress);
    return () => {
      window.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  const fetchFrames = useCallback(async () => {
    try {
      const { identifier } = await invoke<ITraceResponse>("create_trace", {
        file,
        parameters,
      });

      const { frameCount } = await invoke<ISummaryResponse>(
        "get_trace_summary",
        {
          identifier,
        }
      );

      const framePromises = Array.from({ length: frameCount }, (_, i) =>
        invoke<IFrameResponse>("get_frame", {
          identifier,
          frame: i,
        })
      );

      const frames: IFrame[] = await Promise.all(
        framePromises.map((p) => p.then((res) => res.frame))
      );
      setFrames(frames);
    } catch (error) {
      setError(error as string);
    } finally {
      setIsLoading(false);
    }
  }, [file, parameters]);

  useEffect(() => {
    fetchFrames();
  }, [fetchFrames, file, parameters]);

  useEffect(() => {
    if (currentFrame?.retValue) {
      setIsModalOpen(true);
    }
  }, [currentFrame?.retValue]);

  const { cpu: prevCpu = 0, mem: prevMem = 0 } =
    frames[currentFrameIndex - 1]?.budget ?? {};

  const { cpu = 0, mem = 0 } = currentFrame?.budget ?? [];

  const { stepsDiff, memDiff } = useMemo(() => {
    return { stepsDiff: cpu - prevCpu, memDiff: mem - prevMem };
  }, [cpu, mem, prevCpu, prevMem]);

  if (isLoading) {
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
              "hover:bg-lime-600 hover:text-slate-950 duration-300 ease-in-out"
            )}
            onClick={onQuit}
          >
            Try again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="px-2 pb-3 pt-4 relative h-full">
      <div className="border border-lime-600 h-full pt-3">
        <h1 className="px-2 bg-slate-950 absolute right-1/2 translate-x-1/2 top-1">
          Gastronomy Debugger ({fileName})
        </h1>
        <div className="grid grid-rows-[max-content_1fr] h-full text-sm">
          <div className="px-2 pt-1 pb-4">
            <div className="overflow-hidden h-4 mb-1 text-xs flex relative">
              <div
                style={{
                  width: `${(currentFrameIndex / (frames.length - 1)) * 100}%`,
                }}
                className="shadow-none flex flex-col text-center whitespace-nowrap text-white justify-center bg-lime-900 overflow-auto"
              />
            </div>
            <div className="text-xs absolute top-[33px] right-1/2 translate-x-1/2 text-lime-600">
              Step {currentFrameIndex}/{frames.length - 1}
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
                    {currentFrame.budget.cpu} steps
                  </span>{" "}
                  {!!stepsDiff && `+(${stepsDiff})`}
                </div>
                <div className="flex gap-2">
                  <span className="text-blue-600">
                    {currentFrame.budget.mem} mem
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
                Term
              </h2>
              <div className="p-4 overflow-auto absolute inset-0">
                <DisplayString string={currentFrame.term} />
              </div>
            </div>
            <div className="relative">
              <h2 className="left-2 -top-3 bg-slate-950 absolute px-2 z-10">
                Context
              </h2>
              <div className="h-full grid grid-rows-2">
                <div className="relative">
                  <div className="p-4 overflow-auto absolute inset-0">
                    {currentFrame.context.map((c, i) => (
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
                      {currentFrame.env.map(({ name, value }, i) => (
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
        handleNext={handleNext}
        handlePrevious={handlePrevious}
        handleQuite={handleQuite}
      />
      <Modal isOpen={isModalOpen}>
        <div className="">
          <h2 className="absolute left-4 -top-3 bg-slate-950 px-2 text-blue-600">
            Return Value
          </h2>
          <div className="px-4 pt-4 pb-6 h-[30rem] overflow-auto">
            <DisplayString string={currentFrame.retValue} />
          </div>
        </div>
        <DebuggerNavigation
          className="absolute right-1/2 translate-x-1/2 -bottom-2"
          handleNext={handleNext}
          handlePrevious={handlePrevious}
          handleQuite={handleQuite}
        />
      </Modal>
    </div>
  );
};

export default Debugger;

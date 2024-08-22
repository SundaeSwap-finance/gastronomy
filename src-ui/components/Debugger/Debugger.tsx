import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";
import {
  IFrame,
  IFrameResponse,
  ISummaryResponse,
  ITraceResponse,
} from "../../types";
import DisplayString from "../DisplayString";

const Debugger = () => {
  const [currentFrameIndex, setCurrentFrameIndex] = useState(0);
  const [frames, setFrames] = useState<IFrame[]>([]);

  const handleNext = useCallback(() => {
    if (currentFrameIndex < frames.length - 1) {
      setCurrentFrameIndex((prev) => prev + 1);
    }
  }, [currentFrameIndex, frames.length]);

  const handlePrevious = useCallback(() => {
    if (currentFrameIndex > 0) {
      setCurrentFrameIndex((prev) => prev - 1);
    }
  }, [currentFrameIndex]);

  const handleKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "n") {
        handleNext();
      } else if (event.key === "p") {
        handlePrevious();
      }
    },
    [handleNext, handlePrevious]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyPress);
    return () => {
      window.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  const fetchFrames = async () => {
    try {
      const { identifier } = await invoke<ITraceResponse>("create_trace", {
        file: "/Users/selvioperez/Documents/Projects/gastronomy/test_data/fibonacci.uplc",
        parameters: ["03"],
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
      console.error("Failed to fetch frames:", error);
      throw error;
    }
  };

  useEffect(() => {
    fetchFrames();
  }, []);

  const currentFrame = frames[currentFrameIndex];

  return (
    <div className="px-2 pb-3 pt-4 h-screen bg-slate-950 text-lime-600 font-['Source_Code_Pro'] relative">
      <div className="border border-lime-600 h-full pt-3">
        <h1 className="px-2 bg-slate-950 absolute right-1/2 translate-x-1/2 top-1">
          Gastronomy Debugger (fibonacci.uplc)
        </h1>
        <div className="grid grid-rows-[max-content_1fr] h-full text-sm">
          <div className="px-2 pt-1 pb-4">
            <div className="overflow-hidden h-4 mb-1 text-xs flex relative">
              <div
                style={{
                  width: `${(currentFrameIndex / frames.length) * 100}%`,
                }}
                className="shadow-none flex flex-col text-center whitespace-nowrap text-white justify-center bg-lime-900 overflow-auto"
              />
            </div>
            <div className="text-xs absolute top-[33px] right-1/2 translate-x-1/2 text-lime-600">
              Step {currentFrameIndex}/{frames.length}
            </div>
            <div className="flex justify-between">
              <div>
                Current: <span className="text-blue-600">Compute</span>
              </div>
              <div>
                Budget: <span className="text-blue-600">{100} steps</span>{" "}
                (+100) <span className="text-blue-600">100 mem</span> (+100)
              </div>
              <div>
                Next: <span className="text-blue-600">Compute</span>
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
      <div className="px-2 bg-slate-950 flex absolute right-1/2 translate-x-1/2 bottom-1 gap-2 text-sm">
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
    </div>
  );
};

export default Debugger;

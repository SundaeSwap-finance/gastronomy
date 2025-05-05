import { ChangeEvent, FC, useCallback, useEffect, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { IScriptOverride, ISettings } from "../../types";
import objectPath from "object-path";
import Modal from "../Modal";

interface ISettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const Settings: FC<ISettingsProps> = ({ isOpen, onClose }) => {
  const [config, setConfig] = useState<ISettings | null>(null);
  const [store, setStore] = useState<Store | null>(null);

  useEffect(() => {
    (async () => {
      setStore(await Store.load("settings.json"));
    })();
  }, []);

  const onSettingChanged = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setConfig((oldConfig) => {
        const newConfig = { ...oldConfig };
        const updated = event.target.name;
        const value = event.target.value;

        if (updated === "blockfrost.key") {
          if (value) {
            objectPath.set(newConfig, "blockfrost.key", value);
          } else {
            objectPath.del(newConfig, "blockfrost");
          }
        }

        return newConfig;
      });
    },
    [],
  );

  const handleBlueprintFileSelect = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
        filters: [{ name: "Blueprint JSON", extensions: ["json"] }],
      });

      if (selectedPath && typeof selectedPath === "string") {
        setConfig((oldConfig) => {
          if (!oldConfig) return null;
          return { ...oldConfig, blueprintFile: selectedPath };
        });
        saveSettings();
      }
    } catch (error) {
      console.error("Error selecting blueprint file:", error);
    }
  };

  const handleScriptOverrideChange = useCallback(
    (index: number, field: keyof IScriptOverride, value: string) => {
      setConfig((oldConfig) => {
        if (!oldConfig) return null;

        const newConfig = { ...oldConfig };
        const scriptOverrides = [...(newConfig.scriptOverrides || [])];

        scriptOverrides[index] = {
          ...scriptOverrides[index],
          [field]: value,
        };

        newConfig.scriptOverrides = scriptOverrides;
        return newConfig;
      });
    },
    [],
  );

  const addScriptOverride = useCallback(() => {
    setConfig((oldConfig) => {
      if (!oldConfig) return null;

      const newConfig = { ...oldConfig };
      const scriptOverrides = [...(newConfig.scriptOverrides || [])];

      scriptOverrides.push({
        from: "",
        to: "",
      });

      newConfig.scriptOverrides = scriptOverrides;
      return newConfig;
    });
  }, []);

  const removeScriptOverride = useCallback((index: number) => {
    setConfig((oldConfig) => {
      if (!oldConfig) return null;

      const newConfig = { ...oldConfig };
      const scriptOverrides = [...(newConfig.scriptOverrides || [])];

      scriptOverrides.splice(index, 1);
      newConfig.scriptOverrides = scriptOverrides;

      return newConfig;
    });
  }, []);

  const saveSettings = useCallback(async () => {
    if (!store || !config) {
      return;
    }

    try {
      await store.set("config", config);
      await store.save();
    } catch (e) {
      console.error(e);
    }
  }, [store, config]);

  useEffect(() => {
    if (!store) {
      return;
    }

    (async () => {
      const data = (await store.get<ISettings>("config")) ?? {};
      if (!data.scriptOverrides) {
        data.scriptOverrides = [];
      }
      setConfig(data);
    })();
  }, [store]);

  const handleKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Enter" || event.key === "Escape") {
        onClose();
      }
    },
    [onClose],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyPress);
    return () => {
      window.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  return (
    <Modal
      className="w-[48rem] px-2 max-h-[80vh]"
      isOpen={isOpen}
      onClose={onClose}
    >
      <h1 className="absolute left-4 -top-3 bg-slate-950 px-2">Settings</h1>
      <div
        className="px-4 pt-4 pb-4 flex flex-col"
        style={{ height: "calc(80vh - 40px)" }}
      >
        <div className="mb-6">
          <label htmlFor="blockfrost.key">Blockfrost API Key</label>
          <input
            id="blockfrost.key"
            name="blockfrost.key"
            type="text"
            className="my-2 w-full"
            placeholder="Blockfrost API Key"
            value={config?.blockfrost?.key || ""}
            onChange={onSettingChanged}
            onBlur={saveSettings}
          />
        </div>

        <div className="mb-6">
          <h2 className="text-xl font-semibold mb-3">
            Blueprint Configuration
          </h2>
          <div className="flex items-center mb-4">
            <input
              type="text"
              value={config?.blueprintFile || ""}
              readOnly
              className="flex-grow mr-2 pl-1"
              placeholder="No blueprint file selected"
            />
            <button
              type="button"
              onClick={handleBlueprintFileSelect}
              className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Select Blueprint
            </button>
          </div>
        </div>

        <div className="flex-1 flex flex-col min-h-0 mb-6">
          <div className="flex justify-between items-center mb-3">
            <h2 className="text-xl font-semibold">Script Overrides</h2>
            <button
              type="button"
              onClick={addScriptOverride}
              disabled={!config?.blueprintFile}
              className={`px-3 py-1 rounded text-white ${
                config?.blueprintFile
                  ? "bg-green-600 hover:bg-green-700"
                  : "bg-gray-500 cursor-not-allowed opacity-60"
              }`}
            >
              Add Script Override
            </button>
          </div>
          <div className="flex-1 overflow-y-auto overflow-x-hidden pr-2 border border-gray-700 rounded-md bg-slate-900">
            {(!config?.scriptOverrides ||
              config.scriptOverrides.length === 0) && (
              <p className="text-gray-400 italic p-4">
                No script overrides configured. Add one to get started.
              </p>
            )}

            {config?.scriptOverrides?.map((pair, index) => (
              <div
                key={index}
                className="p-4 border-b last:border-b-0 border-gray-700"
              >
                <div className="flex justify-between mb-2">
                  <h3 className="text-lg font-medium">
                    Script Override #{index + 1}
                  </h3>
                  <button
                    type="button"
                    onClick={() => removeScriptOverride(index)}
                    className="text-red-500 hover:text-red-400"
                  >
                    Remove
                  </button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label
                      htmlFor={`targetHash-${index}`}
                      className="block mb-1"
                    >
                      Target Script Hash
                    </label>
                    <input
                      id={`targetHash-${index}`}
                      type="text"
                      value={pair.from}
                      onChange={(e) =>
                        handleScriptOverrideChange(
                          index,
                          "from",
                          e.target.value,
                        )
                      }
                      className="w-full pl-1"
                      placeholder="Hash of script to override"
                      onBlur={saveSettings}
                    />
                  </div>

                  <div>
                    <label
                      htmlFor={`overrideHash-${index}`}
                      className="block mb-1"
                    >
                      Override Script Hash
                    </label>
                    <input
                      id={`overrideHash-${index}`}
                      type="text"
                      value={pair.to}
                      onChange={(e) =>
                        handleScriptOverrideChange(index, "to", e.target.value)
                      }
                      className="w-full pl-1"
                      placeholder="Hash of replacement script"
                      onBlur={saveSettings}
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="mt-auto pt-4 border-t border-gray-700">
          <div className="flex justify-end">
            <button
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
              onClick={() => {
                saveSettings();
                onClose();
              }}
            >
              Save & Close
            </button>
          </div>
        </div>
      </div>
    </Modal>
  );
};

export default Settings;

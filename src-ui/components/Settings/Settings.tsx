import { ChangeEvent, FC, useCallback, useEffect, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { ISettings, IScriptOverride } from "../../types";
import objectPath from "object-path";
import Modal from "../Modal";

interface ISettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const ScriptVersionOptions = [1, 2, 3] as const;

const ScriptOverrideItem: FC<{
  override: IScriptOverride;
  index: number;
  onChange: (index: number, updated: IScriptOverride) => void;
  onRemove: (index: number) => void;
}> = ({ override, index, onChange, onRemove }) => {
  const handleFileSelect = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
      });

      if (selectedPath && typeof selectedPath === "string") {
        onChange(index, { ...override, filePath: selectedPath });
      }
    } catch (error) {
      console.error("Error selecting file:", error);
    }
  };

  return (
    <div className="p-4 border border-gray-700 rounded-md mb-4">
      <div className="flex justify-between mb-2">
        <h3 className="text-lg font-medium">Script Override #{index + 1}</h3>
        <button
          type="button"
          onClick={() => onRemove(index)}
          className="text-red-500 hover:text-red-400"
        >
          Remove
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label htmlFor={`fromHash-${index}`} className="block mb-1">
            From Hash
          </label>
          <input
            id={`fromHash-${index}`}
            type="text"
            value={override.fromHash}
            onChange={(e) =>
              onChange(index, { ...override, fromHash: e.target.value })
            }
            className="w-full pl-1"
            placeholder="Script hash to override"
          />
        </div>

        <div>
          <label htmlFor={`scriptVersion-${index}`} className="block mb-1">
            Script Version
          </label>
          <select
            id={`scriptVersion-${index}`}
            value={override.scriptVersion}
            onChange={(e) =>
              onChange(index, {
                ...override,
                scriptVersion: Number(e.target.value) as 1 | 2 | 3,
              })
            }
            className="w-full"
          >
            {ScriptVersionOptions.map((version) => (
              <option key={version} value={version}>
                Version {version}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="mt-4">
        <label htmlFor={`filePath-${index}`} className="block mb-1">
          Script File
        </label>

        <div className="flex items-center">
          <input
            type="text"
            value={override.filePath}
            readOnly
            className="flex-grow mr-2 pl-1"
            placeholder="No file selected"
          />

          <button
            type="button"
            onClick={handleFileSelect}
            className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Browse
          </button>
        </div>
      </div>
    </div>
  );
};

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

  const handleScriptOverrideChange = useCallback(
    (index: number, updatedOverride: IScriptOverride) => {
      setConfig((oldConfig) => {
        if (!oldConfig) return null;

        const newConfig = { ...oldConfig };
        const scriptOverrides = [...(newConfig.scriptOverrides || [])];
        scriptOverrides[index] = updatedOverride;
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
        filePath: "",
        fromHash: "",
        scriptVersion: 1,
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
      <div className="px-4 pt-4 pb-4 flex flex-col h-full">
        <div className="overflow-y-auto flex-grow">
          <form>
            <div className="flex flex-col mb-6">
              <label htmlFor="blockfrost.key">Blockfrost API Key</label>
              <input
                id="blockfrost.key"
                name="blockfrost.key"
                type="text"
                className="my-2"
                placeholder="Blockfrost API Key"
                value={config?.blockfrost?.key || ""}
                onChange={onSettingChanged}
                onBlur={saveSettings}
              />
            </div>

            <div className="mb-6">
              <div className="flex justify-between items-center mb-3">
                <h2 className="text-xl font-semibold">Script Overrides</h2>
                <button
                  type="button"
                  onClick={addScriptOverride}
                  className="px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700"
                >
                  Add Override
                </button>
              </div>
              <div className="max-h-[50vh] overflow-y-auto overflow-x-hidden pr-2">
                {config?.scriptOverrides?.length === 0 && (
                  <p className="text-gray-400 italic mb-4">
                    No script overrides configured. Add one to get started.
                  </p>
                )}

                {config?.scriptOverrides?.map((override, index) => (
                  <ScriptOverrideItem
                    key={index}
                    override={override}
                    index={index}
                    onChange={handleScriptOverrideChange}
                    onRemove={removeScriptOverride}
                  />
                ))}
              </div>
            </div>
          </form>
        </div>

        <div className="flex justify-end pt-2 border-t border-gray-700">
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
    </Modal>
  );
};

export default Settings;
